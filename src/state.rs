use crate::vertex::{get_vertices, Vertex, INDICES};
use crate::texture::Texture;
use crate::camera::{Camera, CameraUniform};
use crate::projection::Projection;
use crate::camera_controller::CameraController;
use crate::instance::Instance;
use crate::world::World;

use winit::event::WindowEvent;
use winit::window::Window;

use wgpu::util::DeviceExt;
use anyhow::{Result, Context};

// app state
pub struct State {
    // surface
    surface: wgpu::Surface,

    // device
    device: wgpu::Device,

    // queue
    queue: wgpu::Queue,

    // surface config
    config: wgpu::SurfaceConfiguration,

    // window size
    size: winit::dpi::PhysicalSize<u32>,

    // render pipeline
    pipeline: wgpu::RenderPipeline,

    // buffers
    vtx_buf: wgpu::Buffer,
    idx_buf: wgpu::Buffer,

    // world instances
    world:        World,
    instances:    Vec<Instance>,
    instance_buf: wgpu::Buffer,

    // textures
    depth_texture:   Texture,
    diffuse_bind_group: wgpu::BindGroup,

    // camera
    camera:                Camera,
    projection:            Projection,
    pub camera_controller: CameraController,
    camera_uniform:        CameraUniform,
    camera_buf:            wgpu::Buffer,
    camera_bind_group:     wgpu::BindGroup,
}

impl State {
    // create state
    pub async fn new(window: &Window) -> Result<Self> {
        // window size
        let size = window.inner_size();

        // wasm sizing fix
        #[cfg(target_arch = "wasm32")]
        let size = winit::dpi::PhysicalSize { width: size.width / 2, height: size.height / 2 };

        // create instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // create surface
        let surface = unsafe { instance.create_surface(&window) }?;

        // request adapter
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.context("")?;

        // request device
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label:    None,
                features: wgpu::Features::empty(),

                limits: if cfg!(target_arch = "wasm32") {
                    // limit features
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            None,
        ).await?;

        // get surface capabilities
        let caps = surface.get_capabilities(&adapter);

        // find srgb format
        let format =
            caps.formats
                .iter()
                .copied()
                .find(|format| format.is_srgb())
                .unwrap_or(caps.formats[0]);

        // configure surface
        let config = wgpu::SurfaceConfiguration {
            format,
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            width:        size.width,
            height:       size.height,
            present_mode: caps.present_modes[0],
            alpha_mode:   caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        // load image
        let bytes = include_bytes!("assets/texture.png");

        // create bind group
        let (
            diffuse_bind_group_layout,
            diffuse_bind_group,
        ) = Self::create_texture(bytes, &device, &queue)?;

        // create projection
        let projection = Projection::new(
            config.width,
            config.height,
            cgmath::Deg(45.0).into(),
            0.1, 100.0,
        );

        // create camera
        let (
            camera,
            camera_uniform,
            camera_buf,
            camera_bind_group_layout,
            camera_bind_group,
        ) = Self::create_camera(&projection, &device);

        // create camera controller
        let camera_controller = CameraController::new(12.5, 0.5);

        // create shader
        let shader = device.create_shader_module(
            wgpu::include_wgsl!("shaders/shader.wgsl"),
        );

        // create depth texture
        let depth_texture = Texture::create_depth_texture(
            &device, &config, "depth_texture"
        );

        // create pipeline
        let pipeline = Self::create_pipeline(
            &shader,
            &device,
            &config,
            &[
                &diffuse_bind_group_layout,
                &camera_bind_group_layout,
            ],
        );

        // create vertex buffer
        let vtx_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label:    Some("vtx_buf"),
                contents: bytemuck::cast_slice(&get_vertices()),
                usage:    wgpu::BufferUsages::VERTEX,
            }
        );

        // create index buffer
        let idx_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label:    Some("idx_buf"),
                contents: bytemuck::cast_slice(INDICES),
                usage:    wgpu::BufferUsages::INDEX,
            }
        );

        // create world instances
        let mut world = World::new();
        let (instances, instance_buf) = Self::create_instance_buf(&mut world, &camera, &device);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,

            vtx_buf,
            idx_buf,

            world,
            instances,
            instance_buf,

            depth_texture,
            diffuse_bind_group,

            camera,
            projection,
            camera_controller,
            camera_uniform,
            camera_buf,
            camera_bind_group,
        })
    }

    // create texture
    fn create_texture(
        bytes:  &[u8],
        device: &wgpu::Device,
        queue:  &wgpu::Queue,
    ) -> Result<(wgpu::BindGroupLayout, wgpu::BindGroup)> {
        // create from bytes
        let texture = Texture::from_bytes(bytes, device, queue, Some("texture"))?;

        // create bind group layout
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("diffuse_bind_group_layout"),

                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled:   false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type:    wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },

                    wgpu::BindGroupLayoutEntry {
                        binding:    1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty:         wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count:      None,
                    },
                ],
            });

        // create bind group
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label:  Some("diffuse_bind_group"),
                layout: &bind_group_layout,

                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    }
                ],
            }
        );

        Ok((bind_group_layout, bind_group))
    }

    // create camera
    fn create_camera(
        projection: &Projection,
        device:     &wgpu::Device,
    ) -> (Camera, CameraUniform, wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        // create camera
        let camera = Camera::new((0.0, 5.0, 10.0).into(), cgmath::Deg(-90.0).into(), cgmath::Deg(-20.0).into());

        // create camera uniform
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(&camera, projection);

        // create camera buffer
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label:    Some("camera_buf"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // create bind group layout
        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),

                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,

                        // buffer type
                        ty: wgpu::BindingType::Buffer {
                            ty:                 wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size:   None,
                        },

                        count: None,
                    }
                ],
            }
        );

        // create bind group
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label:  Some("camera_bind_group"),
                layout: &bind_group_layout,

                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }
                ],
            }
        );

        (camera, uniform, buffer, bind_group_layout, bind_group)
    }

    // create render pipeline
    fn create_pipeline(
        shader:  &wgpu::ShaderModule,
        device:  &wgpu::Device,
        config:  &wgpu::SurfaceConfiguration,
        layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        // create pipeline layout
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline_layout"),
                bind_group_layouts:   layouts,
                push_constant_ranges: &[],
            });

        // create render pipeline
        device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label:  Some("pipeline"),
                layout: Some(&pipeline_layout),

                // run vertex shader
                vertex: wgpu::VertexState {
                    module:      shader,
                    entry_point: "vtx_main",
                    buffers:     &[Vertex::layout(), Instance::layout()],
                },

                // run fragment shader
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: "frag_main",

                    targets: &[Some(wgpu::ColorTargetState {
                        format:     config.format,
                        blend:      Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),

                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },

                // use depth texture
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil:       wgpu::StencilState::default(),
                    bias:          wgpu::DepthBiasState::default(),
                }),

                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask:  !0,
                    alpha_to_coverage_enabled: false,
                },

                multiview: None,
            }
        )
    }

    // create instance buffer
    fn create_instance_buf(
        world:  &mut World,
        camera: &Camera,
        device: &wgpu::Device,
    ) -> (Vec<Instance>, wgpu::Buffer) {
        // get instances
        let instances = world.instances(camera.pos.x as i32, camera.pos.z as i32);

        // create buffer
        let instance_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label:    Some("instance_buf"),
                contents: bytemuck::cast_slice(&instances),
                usage:    wgpu::BufferUsages::VERTEX,
            }
        );

        (instances, instance_buf)
    }

    // resize by scale factor
    pub fn scale(&mut self, factor: &f64) {
        self.resize(
            winit::dpi::PhysicalSize::new(
                (self.size.width  as f64 * factor) as u32,
                (self.size.height as f64 * factor) as u32,
            )
        );
    }

    // resize window and change state
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        // wasm sizing fix
        #[cfg(target_arch = "wasm32")]
        let new_size = winit::dpi::PhysicalSize { width: new_size.width / 2, height: new_size.height / 2 };

        self.size = new_size;

        self.config.width  = new_size.width;
        self.config.height = new_size.height;

        self.surface.configure(&self.device, &self.config);
        self.projection.resize(self.config.width, self.config.height);

        self.depth_texture = Texture::create_depth_texture(
            &self.device, &self.config, "depth_texture"
        );
    }

    // handle window event
    pub fn event(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.event(event)
    }

    // handle updates
    pub fn update(&mut self, dt: instant::Duration) {
        // update camera
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update_view_proj(&self.camera, &self.projection);

        // write new uniform to buffer
        self.queue.write_buffer(
            &self.camera_buf, 0,
            bytemuck::cast_slice(&[self.camera_uniform])
        );

        // update instances if required
        if self.world.refresh_required(self.camera.pos.x as i32, self.camera.pos.z as i32) {
            (self.instances, self.instance_buf) =
                Self::create_instance_buf(&mut self.world, &self.camera, &self.device);
        }
    }

    // render window
    pub fn render(&mut self) -> Result<()> {
        // wait for provided texture
        let output = self.surface.get_current_texture()?;

        // create texture view
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // create command buffer
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            }
        );

        {
            // begin render pass
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),

                color_attachments: &[Some(
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,

                        // clear screen
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.1,
                                    g: 0.4,
                                    b: 0.7,
                                    a: 1.0,
                                },
                            ),
                            store: wgpu::StoreOp::Store,
                        },
                    }
                )],

                // use depth texture
                depth_stencil_attachment: Some(
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        stencil_ops: None,

                        depth_ops: Some(
                            wgpu::Operations {
                                load:  wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            },
                        ),
                    },
                ),

                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.pipeline);

            // set bind groups
            rpass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            rpass.set_bind_group(1, &self.camera_bind_group,  &[]);

            // set vector buffer
            rpass.set_vertex_buffer(0, self.vtx_buf.slice(..));

            // set instance buffer
            rpass.set_vertex_buffer(1, self.instance_buf.slice(..));

            // set index buffer
            rpass.set_index_buffer(self.idx_buf.slice(..), wgpu::IndexFormat::Uint16);

            // draw instances
            rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..self.instances.len() as u32);
        }

        // submit to queue
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
