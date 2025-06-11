struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) view_position: vec3<f32>,
    @location(1) view_normal: vec3<f32>,
}

struct PointLight {
    position: vec4<f32>,
    color: vec4<f32>,
    radius: f32,
    intensity: f32,
}

struct Cluster {
    minPoint: vec4<f32>,
    maxPoint: vec4<f32>,
    count: u32,
    lightIndices: array<u32, 100>,
};

// Uniforms
@group(0) @binding(0) var<uniform> resolution: vec2<u32>;
@group(0) @binding(1) var<uniform> view_matrix: mat4x4<f32>;
@group(0) @binding(2) var<uniform> projection_matrix: mat4x4<f32>;
@group(0) @binding(3) var<uniform> cluster_size: vec3<u32>;
@group(0) @binding(4) var<storage, read> clusters: array<Cluster>;
@group(1) @binding(0) var<uniform> transform_matrix: mat4x4<f32>;
@group(2) @binding(1) var<storage, read> lights: array<PointLight>;
@group(2) @binding(0) var<uniform> light_num: u32;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Transform to world and view space
    let world_position = transform_matrix * vec4<f32>(input.position, 1.0);
    let view_position4 = view_matrix * world_position;

    // Project to clip space
    output.clip_position = projection_matrix * view_position4;
    output.view_position = view_position4.xyz;

    // Transform normal using inverse transpose of model-view matrix
    let normal_matrix = mat3x3<f32>(
        view_matrix[0].xyz,
        view_matrix[1].xyz,
        view_matrix[2].xyz
    ) * mat3x3<f32>(
        transform_matrix[0].xyz,
        transform_matrix[1].xyz,
        transform_matrix[2].xyz
    );
    output.view_normal = normalize(normal_matrix * input.normal);

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Reconstruct normalized vectors
    let N = normalize(input.view_normal);
    let V = normalize(-input.view_position); // View direction

    // Initialize lighting
    var total_light = vec3<f32>(0.05); // Ambient
    var cluster_found = true;
    var cluster_index = u32((input.view_position.x + 1)  * f32(cluster_size.x/2)) + u32((input.view_position.y + 1)  *  f32(cluster_size.y/2)) * 12 + u32((input.view_position.z + 1)  *  f32(cluster_size.z/2)) * 144;


    let cluster = clusters[cluster_index];
    let count = min(cluster.count, 100u);

    for (var i: u32 = 0; i < count; i++) {
        let light_idx = cluster.lightIndices[i];
        let light = lights[light_idx];

        // Transform light to view space
        let light_view_pos = (view_matrix * light.position).xyz;
        let L = light_view_pos - input.view_position;
        let dist = length(L);
        let L_dir = normalize(L);

        // Attenuation with smooth cutoff
        let d = dist / light.radius;
        let attenuation = light.intensity * (1.0 - smoothstep(0.8, 1.0, d));

        // Diffuse contribution
        let NdotL = max(dot(N, L_dir), 0.0);
        let diffuse = light.color.rgb * NdotL * attenuation;

        // Specular (Blinn-Phong)
        let H = normalize(L_dir + V);
        let NdotH = max(dot(N, H), 0.0);
        let specular = light.color.rgb * pow(NdotH, 32.0) * attenuation;

        total_light += diffuse + specular;
    }

    return vec4<f32>(total_light.xyz , 1.0);
    //return vec4<f32>(f32(u32((input.view_position.x + 1)  * 6.0)) / 12 ,f32(u32((input.view_position.y + 1) * 6.0)) / 12,f32(u32((input.view_position.z + 1) * 12.0)) / 24 , 1.0);
    //return vec4<f32>(f32(cluster_index) / f32(arrayLength(&clusters)), f32(cluster_index ) / f32( arrayLength(&clusters)), f32(cluster_index ) / f32( arrayLength(&clusters)), 1.0);
}