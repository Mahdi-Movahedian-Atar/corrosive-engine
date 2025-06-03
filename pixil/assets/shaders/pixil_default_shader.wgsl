struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}
struct VertexOutput {
    @builtin(position) clip_position : vec4 <f32>,
    @location(0) uv : vec2 <f32>
};

struct PointLight {
    position: vec4<f32>,
    color: vec4<f32>,
    intensity: f32,
    radius: f32,
};
struct Cluster {
    minPoint: vec4<f32>,
    maxPoint: vec4<f32>,
    count: u32,
    lightIndices: array<u32, 100>,
};

@group(0) @binding(0) var<uniform> resolution : vec2<u32>;
@group(0) @binding(1) var<uniform> view_matrix : mat4x4<f32>;
@group(0) @binding(1) var<uniform> position : vec3<f32>;
@group(0) @binding(1) var<uniform> near_far : vec2<f32>;

@group(1) @binding(0) var<uniform> transform_matrix : mat4x4<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Transform the vertex position
    let world_position = transform_matrix * vec4<f32>(input.position, 1.0);
    let view_position = view_matrix * world_position;

    output.clip_position = view_position;

    // Use normalized screen position as fake UV
    output.uv = (input.position.xy + vec2<f32>(1.0, 1.0)) * 0.5;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    //let color = vec3<f32>(input.uv, 1.0 - input.uv.x); // Gradient color
    return input.clip_position;
}
