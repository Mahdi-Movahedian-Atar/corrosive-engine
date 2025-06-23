struct PointLight {
    position: vec4<f32>,
    radius: f32,
    intensity: f32,
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
var<uniform> len: u32;
@group(1) @binding(1)
var<storage, read> pointLight: array<PointLight>;

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

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) workGroupID: vec3<u32>) {
    let index = workGroupID.x + (workGroupID.y * cluster_size.x) + (workGroupID.z * cluster_size.x * cluster_size.y);

    var cluster = clusters[index];
    cluster.count = 0u;

    for (var i: u32 = 0u; i < len; i = i + 1u) {
        if (cluster.count < 100u && testSphereAABB(i, cluster)) {
            cluster.lightIndices[cluster.count] = i;
            cluster.count = cluster.count + 1u;
        }
    }

    clusters[index] = cluster;
}
