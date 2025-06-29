struct PointLight {
    position: vec4<f32>,
    radius: f32,
    intensity: f32,
};
struct SpotLight {
    position: vec4f,
    direction: vec4f,
    radius: f32,
    color_index:u32,
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

@group(0) @binding(0)
var<uniform> viewMatrix: mat4x4<f32>;
@group(0) @binding(1)
var<uniform> cluster_size: vec3<u32>;
@group(0) @binding(2)
var<storage, read_write> clusters: array<Cluster>;

@group(1) @binding(0)
var<uniform> point_len: u32;
@group(1) @binding(1)
var<storage, read> pointLight: array<PointLight>;
@group(1) @binding(2)
var<uniform> spot_len: u32;
@group(1) @binding(3)
var<storage, read> spotLight: array<SpotLight>;

fn sphereAABBIntersection(center: vec3<f32>, radius: f32, aabbMin: vec3<f32>, aabbMax: vec3<f32>) -> bool {
    let closestPoint = clamp(center, aabbMin, aabbMax);
    let distVec = closestPoint - center;
    return dot(distVec, distVec) <= radius * radius;
}

fn testSphereAABB(i: u32, cluster: Cluster) -> bool {
    let center = (viewMatrix * pointLight[i].position).xyz;
    let radius = pointLight[i].radius;
    let aabbMin = cluster.minPoint.xyz;
    let aabbMax = cluster.maxPoint.xyz;
    return sphereAABBIntersection(center, radius, aabbMin, aabbMax);
}

fn coneAABBIntersects(
    cone_origin: vec3<f32>,
    cone_dir: vec3<f32>,
    cone_angle_rad: f32,
    cone_range: f32,
    aabb_min: vec3<f32>,
    aabb_max: vec3<f32>
) -> bool {
    let cone_half_angle = cone_angle_rad * 0.5;
    let cos_half_angle = cos(cone_half_angle);
    let sin_half_angle = sin(cone_half_angle);

    // Early-out bounding sphere check (conservative)
    let aabb_center = 0.5 * (aabb_min + aabb_max);
    let aabb_extent = 0.5 * (aabb_max - aabb_min);
    let box_radius = length(aabb_extent);

    let to_center = aabb_center - cone_origin;
    let dist_to_center = length(to_center);
    if (dist_to_center > cone_range + box_radius) {
        return false; // too far
    }

    // Project center of AABB onto cone direction
    let projection = dot(to_center, cone_dir);

    // Closest approach from cone axis to box center
    let perpendicular_dist = sqrt(max(dot(to_center, to_center) - projection * projection, 0.0));
    let cone_radius_at_proj = projection * tan(cone_half_angle);

    // Conservative sphere-cone test
    if projection < 0.0 || projection > cone_range {
        return false;
    }

    if perpendicular_dist > (cone_radius_at_proj + box_radius) {
        return false;
    }

    // Angle test: check if the center lies within the cone’s sweep
    let dir_to_center = normalize(to_center);
    let angle_cos = dot(dir_to_center, cone_dir);
    if angle_cos < cos_half_angle {
        return false;
    }

    return true;
}



fn testConeAABB(i: u32, cluster: Cluster) -> bool {
    let pos_ws = spotLight[i].position;
    let dir_ws = spotLight[i].direction;

    let pos_vs = (viewMatrix * pos_ws).xyz;
    let dir_vs = normalize((viewMatrix * dir_ws).xyz);

    let radius = spotLight[i].radius;
    let angle = spotLight[i].outer_angle;

    let aabbMin = cluster.minPoint.xyz;
    let aabbMax = cluster.maxPoint.xyz;

    return coneAABBIntersects(pos_vs, dir_vs, angle, radius, aabbMin, aabbMax);
}




@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) workGroupID: vec3<u32>) {
    let index = workGroupID.x + (workGroupID.y * cluster_size.x) + (workGroupID.z * cluster_size.x * cluster_size.y);

    var cluster = clusters[index];
    cluster.count = 0u;

    for (var i: u32 = 0u; i < point_len; i = i + 1u) {
        if (cluster.count < 100u && testSphereAABB(i, cluster)) {
            var j = i;
            j <<= 2;
            j |=1;
            cluster.lightIndices[cluster.count] = j;
            cluster.count = cluster.count + 1u;
        }
    }
    for (var i: u32 = 0u; i < spot_len; i = i + 1u) {
         if (cluster.count < 100u && testConeAABB(i, cluster)) {
            var j = i;
            j <<= 2;
            j |= 2;
            cluster.lightIndices[cluster.count] = j;
            cluster.count = cluster.count + 1u;
         }
    }

    clusters[index] = cluster;
}
