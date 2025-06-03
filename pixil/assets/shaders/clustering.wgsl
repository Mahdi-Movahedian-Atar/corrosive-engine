// cluster_bounds.wgsl

struct Cluster {
    minPoint: vec4<f32>,
    maxPoint: vec4<f32>,
    count: u32,
    lightIndices: array<u32, 100>,
};

@group(0) @binding(0)
var<uniform> zParams: vec2<f32>;

@group(0) @binding(1)
var<uniform> inverseProjection: mat4x4<f32>;

@group(0) @binding(2)
var<uniform> gridSize: vec3<u32>;

@group(0) @binding(3)
var<uniform> screenDimensions: vec2<f32>;

@group(0) @binding(4)
var<storage, read_write> clusters: array<Cluster>;

fn lineIntersectionWithZPlane(startPoint: vec3<f32>, endPoint: vec3<f32>, zDistance: f32) -> vec3<f32> {
    let direction = endPoint - startPoint;
    let normal = vec3<f32>(0.0, 0.0, -1.0);
    let t = (zDistance - dot(normal, startPoint)) / dot(normal, direction);
    return startPoint + t * direction;
}

fn screenToView(screenCoord: vec2<f32>) -> vec3<f32> {
    let ndc = vec4<f32>((screenCoord / screenDimensions) * 2.0 - vec2<f32>(1.0, 1.0), -1.0, 1.0);
    let viewCoord = inverseProjection * ndc;
    return viewCoord.xyz / viewCoord.w;
}

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(workgroup_id) workGroupID: vec3<u32>) {
    let tileIndex = workGroupID.x + (workGroupID.y * gridSize.x) + (workGroupID.z * gridSize.x * gridSize.y);
    let tileSize = screenDimensions / vec2<f32>(gridSize.xy);

    let minTileScreen = vec2<f32>(workGroupID.xy) * tileSize;
    let maxTileScreen = (vec2<f32>(workGroupID.xy) + vec2<f32>(1.0, 1.0)) * tileSize;

    let minTileView = screenToView(minTileScreen);
    let maxTileView = screenToView(maxTileScreen);

    let zNear = zParams.x;
    let zFar = zParams.y;

    let slice = f32(workGroupID.z);
    let totalSlices = f32(gridSize.z);
    let planeNear = zNear * pow(zFar / zNear, slice / totalSlices);
    let planeFar = zNear * pow(zFar / zNear, (slice + 1.0) / totalSlices);

    let minPointNear = lineIntersectionWithZPlane(vec3<f32>(0.0), minTileView, planeNear);
    let minPointFar = lineIntersectionWithZPlane(vec3<f32>(0.0), minTileView, planeFar);
    let maxPointNear = lineIntersectionWithZPlane(vec3<f32>(0.0), maxTileView, planeNear);
    let maxPointFar = lineIntersectionWithZPlane(vec3<f32>(0.0), maxTileView, planeFar);

    clusters[tileIndex].minPoint = vec4<f32>(min(minPointNear, minPointFar), 0.0);
    clusters[tileIndex].maxPoint = vec4<f32>(max(maxPointNear, maxPointFar), 0.0);
}
