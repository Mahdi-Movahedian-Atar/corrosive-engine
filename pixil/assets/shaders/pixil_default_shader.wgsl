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
    radius: f32,
    intensity: f32,
    index:u32
}
struct SpotLight {
    position: vec4f,
    direction: vec4f,
    radius: f32,
    index:u32,
    inner_angle: f32,
    outer_angle: f32,
    attenuation: vec4f,
};

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
@group(0) @binding(4) var<uniform> z_params: vec2<f32>;
@group(0) @binding(5) var<storage, read> clusters: array<Cluster>;
@group(1) @binding(0) var<uniform> transform_matrix: mat4x4<f32>;
@group(2) @binding(1) var<storage, read> point_lights: array<PointLight>;
@group(2) @binding(0) var<uniform>  point_lights_num: u32;
@group(2) @binding(3) var<storage, read> spot_lights: array<SpotLight>;
@group(2) @binding(2) var<uniform>  spot_lights_num: u32;
@group(3) @binding(0)
var gradient_texture: texture_2d<f32>;
@group(3) @binding(1)
var gradient_sampler: sampler;
@group(3) @binding(2)
var dither_view: texture_2d<f32>;
@group(3) @binding(3)
var dither_sampler: sampler;

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

fn get_cluster_index(
    frag_coord: vec4<f32>,
    view_z: f32
) -> u32 {
    let tile_size = vec2<f32>(resolution) / vec2<f32>(cluster_size.xy);

    let tile_x = u32(clamp(frag_coord.x / tile_size.x, 0.0, f32(cluster_size.x - 1)));
    let tile_y = u32(clamp(frag_coord.y / tile_size.y, 0.0, f32(cluster_size.y - 1)));

    let z_view = -view_z; // assuming right-handed view space (z negative into screen)
    let z_slice = u32(clamp(
        log(z_view / z_params.x) / log(z_params.y / z_params.x) * f32(cluster_size.z),
        0.0,
        f32(cluster_size.z - 1)
    ));

    let cluster_index = tile_x + tile_y * cluster_size.x + z_slice * cluster_size.x * cluster_size.y;
    return cluster_index;
}



fn point_light_calc(N:vec3f,V:vec3f,index:u32,input: VertexOutput,dither:f32) -> vec3<f32>{
    let light = point_lights[index];
    let light_view_pos = (view_matrix * light.position).xyz;
    let L = light_view_pos - input.view_position;
    let dist = length(L);
    let L_dir = normalize(L);

    let d = dist / light.radius;
    let attenuation = light.intensity * (1.0 - smoothstep(0.8, 1.0, d));

    let NdotL = max(dot(N, L_dir), 0.0);
    let diffuse = NdotL * attenuation;

    let H = normalize(L_dir + V);
    let NdotH = max(dot(N, H), 0.0);
    let specular = diffuse * pow(NdotH, 32.0) * attenuation;

    return textureSample(gradient_texture, gradient_sampler,vec2<f32>((diffuse + specular) + dither,f32(light.index)/255.0) ).xyz;
}

fn spot_light_calc(
    N: vec3<f32>,
    V: vec3<f32>,
    index: u32,
    input: VertexOutput,
    dither: f32
) -> vec3<f32> {
    let light = spot_lights[index];
    /*var light: SpotLight;
    light.direction = vec3f(0.0,0.0,-1.0);
    light.radius = 4;
    light.intensity = 1;
    light.inner_angle = 3.14/6.0;
    light.outer_angle = 3.14/4.0;
    light.index = 2u;
    light.attenuation = vec3f(1.0, 0.0, 0.1);*/

    // Transform light position and direction to view space
    let light_pos_view = (view_matrix * light.position).xyz;
    let light_dir_view = normalize((view_matrix * light.direction).xyz);

    let L = light_pos_view - input.view_position;
    let dist = length(L);
    if (dist > light.radius) {
        return vec3<f32>(0.0);
    }

    let L_dir = normalize(L);
    let H = normalize(L_dir + V);

    // Spotlight cone falloff
    let cos_theta = dot(-L_dir, light_dir_view);
    let cos_inner = cos(light.inner_angle);
    let cos_outer = cos(light.outer_angle);

    let spot_effect = smoothstep(cos_outer, cos_inner, cos_theta);
    //let spot_effect = clamp((cos_theta- cos_outer)/ (cos_inner-cos_outer), 0.0,1.0);
    if (spot_effect <= 0.0) {
        return vec3f(0.0);  // Fixed: Return zero instead of normalize(cos_theta)
    }

    // Attenuation
    let attenuation = light.attenuation.w / (
        light.attenuation.x +
        light.attenuation.y * dist +
        light.attenuation.z * dist * dist
    );
    /*let d = dist / light.radius;
        let attenuation = light.intensity * (1.0 - smoothstep(0.8, 1.0, d));*/

    // Diffuse
    let NdotL = max(dot(N, L_dir), 0.0);
    let diffuse = NdotL * attenuation * spot_effect;

    // Specular (Blinn-Phong)
    let NdotH = max(dot(N, H), 0.0);
    let specular = pow(NdotH, 32.0) * attenuation * spot_effect;

    // Sample from gradient texture
    let u =1.0- clamp(diffuse + specular + dither, 0.0, 1.0);
    let v = f32(light.index) / 255.0;
    return textureSample(gradient_texture, gradient_sampler, vec2<f32>(u, v)).xyz;
    //return vec3f(u);
}


@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Reconstruct normalized vectors
    // View direction

    // Initialize lighting
    let N = normalize(input.view_normal);
    let V = normalize(-input.view_position);

    var total_light = vec3<f32>(0.0); // Ambient
    /*var cluster_found = true;
    var cluster_index = u32((input.view_position.x + 1)  * f32(cluster_size.x/2)) + u32((input.view_position.y + 1)  *  f32(cluster_size.y * cluster_size.x / 4)) + u32((input.view_position.z + 1)  *  f32(cluster_size.y * cluster_size.x *cluster_size.z/8));
*/
    var cluster_index =  get_cluster_index(input.clip_position,input.view_position.z);
    let cluster = clusters[cluster_index];
    let count = min(cluster.count, 100u);

    let ndc = input.view_position.xyz ;
    let screen_space = ndc.xy * 0.5 + 0.5;

    let pixel_coords = vec2<u32>(screen_space * vec2<f32>(f32(resolution.x), f32(resolution.y)));
    let textureDimensions = textureDimensions(dither_view).xy;

    let x = pixel_coords.x % textureDimensions.x;
    let y = pixel_coords.y % textureDimensions.y ;

    let dither = (textureLoad(dither_view, vec2<u32>(x,y), 0).x - 0.5) * 0.3;

    for (var i: u32 = 0; i < count; i++) {
        let light_idx = cluster.lightIndices[i];

        if ((light_idx & 1u) != 0u){
            let idx = light_idx >> 2;
            total_light += point_light_calc(N,V,idx,input,dither);
            continue;
        }
        if ((light_idx & 2u) != 0u){
            let idx = light_idx >> 2;
            total_light += spot_light_calc(N,V,idx,input,dither);
        }
        /*if (count == 1){
            return vec4f(1.0,0.0,0.0,1.0);
        }
        if (count == 2){
                    return vec4f(0.0,1.0,0.0,1.0);
        }
        if (count == 3){
                    return vec4f(0.0,0.0,1.0,1.0);
        }*/

    }
    //total_light += spot_light_calc(N,V,1,input,dither);
    //total_light += spot_light_calc(N,V,0,input,dither);

   return vec4<f32>(total_light.xyz , 1.0);
    //return vec4<f32>(f32(u32((input.view_position.x + 1)  * f32(cluster_size.x/2))) / 24.0,f32(u32((input.view_position.y + 1)  * f32(cluster_size.y/2)))/24.0,f32(u32((input.view_position.z + 1)  * f32(cluster_size.z/2))) / 24.0 , 1.0);
    //return vec4<f32>(vec3(spot_lights[0].outer_angle) , 1.0);
    //return vec4<f32>(vec3<f32>(f32(screen_space.x) / 9,f32(screen_space.y) / 9,1.0) , 1.0);
    //return vec4<f32>(f32(u32((input.view_position.x + 1)  * 6.0)) / 12 ,f32(u32((input.view_position.y + 1) * 6.0)) / 12,f32(u32((input.view_position.z + 1) * 12.0)) / 24 , 1.0);
    //return vec4<f32>(f32(cluster_index) / f32(arrayLength(&clusters)), f32(cluster_index ) / f32( arrayLength(&clusters)), f32(cluster_index ) / f32( arrayLength(&clusters)), 1.0);
}