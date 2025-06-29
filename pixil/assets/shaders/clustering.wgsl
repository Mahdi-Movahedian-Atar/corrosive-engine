struct Cluster {
    minPoint: vec4<f32>,
    maxPoint: vec4<f32>,
    count: u32,
    lightIndices: array<u32, 100>,
};

@group(0) @binding(0)
var<uniform> zRange: vec2<f32>;

@group(0) @binding(1)
var<uniform> inverseProjection: mat4x4<f32>;

@group(0) @binding(2)
var<uniform> gridSize: vec3<u32>;

@group(0) @binding(3)
var<uniform> screenDimensions: vec2<u32>;

@group(0) @binding(4)
var<storage, read_write> clusters: array<Cluster>;

fn screenToView(screenCoord: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(
        (screenCoord / vec2<f32>(screenDimensions)) * 2.0 - vec2<f32>(1.0, 1.0),
        depth,
        1.0
    );
    let view = inverseProjection * ndc;
    return view.xyz / view.w;
}

fn getLogDepth(slice: u32, sliceCount: u32, zNear: f32, zFar: f32) -> f32 {
    return zNear * pow(zFar / zNear, f32(slice) / f32(sliceCount));
}

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(workgroup_id) tileID: vec3<u32>) {
    let tileX = tileID.x;
    let tileY = tileID.y;
    let sliceZ = tileID.z;

    let tileIndex = tileX + tileY * gridSize.x + sliceZ * gridSize.x * gridSize.y;

    let tileSize = vec2<f32>(screenDimensions) / vec2<f32>(gridSize.xy);

    let minScreen = vec2<f32>(f32(tileX), f32(tileY)) * tileSize;
    let maxScreen = vec2<f32>(f32(tileX + 1u), f32(tileY + 1u)) * tileSize;

    let zNear = getLogDepth(sliceZ, gridSize.z, zRange.x, zRange.y);
    let zFar  = getLogDepth(sliceZ + 1u, gridSize.z, zRange.x, zRange.y);

    let p0 = screenToView(minScreen, -1.0);
    let p1 = screenToView(maxScreen, -1.0);

    let minViewNear = normalize(p0) * zNear;
    let maxViewNear = normalize(p1) * zNear;

    let minViewFar = normalize(p0) * zFar;
    let maxViewFar = normalize(p1) * zFar;

    let minView = min(min(minViewNear, maxViewNear), min(minViewFar, maxViewFar));
    let maxView = max(max(minViewNear, maxViewNear), max(minViewFar, maxViewFar));

    clusters[tileIndex].minPoint = vec4<f32>(minView, 1.0);
    clusters[tileIndex].maxPoint = vec4<f32>(maxView, 1.0);
    clusters[tileIndex].count = 0u;
}

