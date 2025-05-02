struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) location: vec2<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position : vec4 <f32>,
    @location(0) location : vec2 <f32>
};

@group(0) @binding(0)
var<uniform> transform : mat4x4<f32>;
@group(0) @binding(1)
var<uniform> camera_matrix: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> resolution : vec2<f32>;

@group(3) @binding(0)
var<uniform> color : vec4<f32>;

@group(2) @binding(0)
var diffuse_texture: texture_2d<f32>;
@group(2) @binding(1)
var diffuse_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let world_pos = transform * vec4<f32>(input.position, 1.0);
    output.clip_position = camera_matrix * world_pos;

    output.location = input.location;
    return output;
}

// Fragment Shader
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let tex_color = textureSample(
        diffuse_texture,
        diffuse_sampler,
        input.location
    );

    // Apply color tint
    let final_color = tex_color * color;

    return final_color;
}

