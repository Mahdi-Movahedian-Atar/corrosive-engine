// Yo, here's what the GPU munchies look like 🍔
struct VtxIn {
    @location(0) pos: vec3<f32>,  // "Where dis vertex at?"
    @location(1) uv: vec2<f32>,   // "Texture coordinates, my G"
};

// What we send to the frag shader 🚀
struct VtxOut {
    @builtin(position) clip_pos: vec4<f32>, // "Screen spot"
    @location(0) uv: vec2<f32>              // "Pass dat UV drip"
};

// Group 0: Moves/Rotates/Scales sh*t 🌪️
@group(0) @binding(0)
var<uniform> move_rotate_scale_mat: mat3x3<f32>;

// Group 1: How big da window be 📺
@group(1) @binding(0)
var<uniform> screen_size: vec2<f32>;

// Group 3: Base color vibe 🎨
@group(3) @binding(0)
var<uniform> base_tint: vec4<f32>;

// Group 2: Da main image + sampler 🖼️
@group(2) @binding(0)
var main_image: texture_2d<f32>;
@group(2) @binding(1)
var image_sampler: sampler;

// Vertex Shader: Position converter 🔄
@vertex
fn vs_main(in: VtxIn) -> VtxOut {
    var out: VtxOut;
    out.clip_pos = vec4<f32>(in.pos, 1.0); // "3D → 4D magic"
    out.uv = in.uv;                        // "UVs rollin' deep"
    return out;
}

// Fragment Shader: Pixel painter 🖌️ (currently boring)
@fragment
fn fs_main(in: VtxOut) -> @location(0) vec4<f32> {
    var final_color = vec4<f32>(0.0, 0.0, 0.0, 1.0); // "Still dark, fam 🕶️"
    return final_color;
}