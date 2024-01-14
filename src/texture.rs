use image::GenericImageView;
use anyhow::Result;

// texture
pub struct Texture {
    // texture
    pub texture: wgpu::Texture,

    // texture view
    pub view: wgpu::TextureView,

    // sampler
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    // create from bytes
    pub fn from_bytes(
        bytes:  &[u8],
        device: &wgpu::Device,
        queue:  &wgpu::Queue,
        label:  Option<&str>,
    ) -> Result<Self> {
        let image = image::load_from_memory(bytes)?;
        Ok(Self::from_image(&image, device, queue, label))
    }

    // create from image
    pub fn from_image(
        image:  &image::DynamicImage,
        device: &wgpu::Device,
        queue:  &wgpu::Queue,
        label:  Option<&str>,
    ) -> Self {
        let rgba = image.to_rgba8();
        let dims = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width:  dims.0,
            height: dims.1,
            depth_or_array_layers: 1,
        };

        // create texture
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        queue.write_texture(
            // copy texture
            wgpu::ImageCopyTexture {
                texture:   &texture,
                mip_level: 0,
                origin:    wgpu::Origin3d::ZERO,
                aspect:    wgpu::TextureAspect::All,
            },

            &rgba,

            // data layout
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row:  Some(4 * dims.0),
                rows_per_image: Some(dims.1),
            },

            texture_size,
        );

        // create view and sampler
        let view    = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { ..Default::default() }
        );

        Self { texture, view, sampler }
    }

    // create depth texture
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label:  &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width:  config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        // create texture
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size,
                label: Some(label),

                mip_level_count: 1,
                sample_count:    1,

                dimension: wgpu::TextureDimension::D2,
                format:    Self::DEPTH_FORMAT,

                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,

                view_formats: &[],
            },
        );

        // create view and sampler
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                compare: Some(wgpu::CompareFunction::LessEqual),
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }
}
