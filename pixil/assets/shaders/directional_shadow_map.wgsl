@group(0) @binding(0) var<uniform> u_light_view_proj: mat4x4<f32>;
@group(1) @binding(0) var<uniform> transform_matrix: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = transform_matrix * vec4<f32>(in.position, 1.0);
    let flipped_pos = vec4<f32>(world_pos.x, world_pos.y, world_pos.z, world_pos.w);
    out.clip_position = u_light_view_proj * flipped_pos;
    return out;
}