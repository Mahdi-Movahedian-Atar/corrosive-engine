
@vertex
fn vs_main(input: VertexInput) -> @builtin(position) vec4<f32> {
    return vec4<f32>(0.0);
}

// Fragment Shader
@fragment
fn fs_main(position: vec4<f32> ) -> @location(0) vec4<f32> {

    return vec4<f32>(1.0);
}
