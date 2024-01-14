// vertex input
struct VertexInput {
  // position
  @location(0)
  pos : vec4<f32>,

  // texture position
  @location(1)
  tex_pos : vec2<f32>,

  // face index
  @location(2)
  face : u32,
};

// instance input
struct InstanceInput {
  // position
  @location(3)
  pos : vec4<f32>,

  // texture
  @location(4)
  tex : f32,

  // faces to render
  @location(5)
  faces : u32,
}

// vertex output
struct VertexOutput {
  // clip position
  @builtin(position)
  clip_pos : vec4<f32>,

  // texture position
  @location(0)
  tex_pos : vec2<f32>,

  // face index
  @location(1)
  face : u32,

  // should render
  @location(2)
  render : u32,
};

// camera uniform
struct CameraUniform {
  view_proj : mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera : CameraUniform;

// vertex shader
@vertex
fn vtx_main(in : VertexInput, instance : InstanceInput) -> VertexOutput {
  var out : VertexOutput;

  // alter view based on projection
  out.clip_pos = camera.view_proj * (in.pos + instance.pos);
  out.tex_pos  = vec2<f32>(in.tex_pos.x, in.tex_pos.y + instance.tex);

  // render face if in faces
  out.face   = in.face;
  out.render = (1u << in.face) & instance.faces;

  return out;
}

// texture
@group(0) @binding(0)
var t_diffuse : texture_2d<f32>;

// sampler
@group(0) @binding(1)
var s_diffuse : sampler;

// darken certain faces
fn lighten(face : u32) -> f32 {
  switch face {
    case 1u: {
      return 0.75;
    }

    case 3u: {
      return 0.85;
    }

    case 5u: {
      return 0.65;
    }

    default: {
      return 1.0;
    }
  }
}

// fragment shader
@fragment
fn frag_main(in : VertexOutput) -> @location(0) vec4<f32> {
  if in.render == 0u {
    // discard face
    discard;
  }

  // sample texture
  var texture = textureSample(t_diffuse, s_diffuse, in.tex_pos);

  // lighten based on face
  var amount = lighten(in.face);
  texture *= vec4<f32>(amount, amount, amount, 1.0);

  return texture;
}
