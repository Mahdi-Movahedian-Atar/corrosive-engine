struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) location: vec2<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position : vec4 <f32>,
    @location(0) location : vec2 <f32>
};
struct UIStyle {
    border: vec4<f32>,
    corner: vec4<f32>,
    color: vec4<f32>,
    border_l_color: vec4<f32>,
    border_t_color: vec4<f32>,
    border_r_color: vec4<f32>,
    border_b_color: vec4<f32>,
    ratio: f32,
    rotation: f32,
    center: vec2<f32>
};

@group(0) @binding(0)
var<uniform> ui_style : UIStyle;

fn rotate_around_center(pos: vec2<f32>, center: vec2<f32>, angle: f32) -> vec2<f32> {
    let s = sin(degrees(angle));
    let c = cos(degrees(angle));

    // Translate position relative to center
    let translated = pos - center;

    // Apply rotation matrix
    let rotated = vec2<f32>(
        translated.x * c - translated.y * s,
        translated.x * s + translated.y * c
    );

    // Translate back
    return rotated + center;
}

@vertex
fn vs_main(
model : VertexInput,
) -> VertexOutput {
    var out : VertexOutput;

    out.location = model.location;
    out.clip_position = vec4 <f32> (rotate_around_center(model.position,ui_style.center,ui_style.rotation),0.0,1.0 );

    return out;
}

fn sdf_rounded_rect(p: vec2<f32>, half_extents: vec2<f32>, corner: vec4<f32>) -> f32 {
    var r: f32 = 0.0;
    if (p.x < 0.0 && p.y < 0.0) {
        r = corner.x;
    } else if (p.x >= 0.0 && p.y < 0.0) {
        r = corner.y;
    } else if (p.x >= 0.0 && p.y >= 0.0) {
        r = corner.z;
    } else {
        r = corner.w;
    }
    let adjusted_extents = half_extents - vec2<f32>(r);
    let d = abs(p) - adjusted_extents;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let half_extents = vec2<f32>(1, ui_style.ratio) * 0.5;
    let p = in.location * vec2<f32>(1, ui_style.ratio) - half_extents;

    let d_outer = sdf_rounded_rect(p, half_extents, ui_style.corner);
    let left_border = ui_style.border.x;
    let top_border = ui_style.border.y;
    let right_border = ui_style.border.z;
    let bottom_border = ui_style.border.w;

    let inner_left = -half_extents.x + left_border;
    let inner_right = half_extents.x - right_border;
    let inner_top = -half_extents.y + top_border;
    let inner_bottom = half_extents.y - bottom_border;

    let inner_half_extents = vec2<f32>(
        (inner_right - inner_left) * 0.5,
        (inner_bottom - inner_top) * 0.5
    );
    let inner_center = vec2<f32>(
        (inner_right + inner_left) * 0.5,
        (inner_bottom + inner_top) * 0.5
    );
    let p_inner = p - inner_center;

    let inner_corner = vec4<f32>(
        max(ui_style.corner.x - left_border, 0.0),
        max(ui_style.corner.y - top_border, 0.0),
        max(ui_style.corner.z - right_border, 0.0),
        max(ui_style.corner.w - bottom_border, 0.0)
    );

    let d_inner = sdf_rounded_rect(p_inner, inner_half_extents, inner_corner);

    var final_color: vec4<f32> = vec4<f32>(0.0);

    if (d_outer > 0.0) {
        final_color = vec4<f32>(0.0);
    } else if (d_inner > 0.0) {

        if (in.location.x < 0.5){
            if (in.location.y < 0.5){
                if (in.location.x * ui_style.border[1] < in.location.y * ui_style.border[0] * ui_style.ratio){
                    final_color = ui_style.border_l_color;
                } else {
                    final_color = ui_style.border_t_color;
                }
            }
            else {
                if (in.location.x * ui_style.border[3] < (1 - in.location.y) * ui_style.border[0]  * ui_style.ratio){
                    final_color = ui_style.border_l_color;
                } else {
                    final_color = ui_style.border_b_color;
                }
            }
        }else {
            if (in.location.y < 0.5){
                if ((1 - in.location.x) * ui_style.border[1] < in.location.y * ui_style.border[2] * ui_style.ratio){
                    final_color = ui_style.border_r_color;
                } else {
                    final_color = ui_style.border_t_color;
                }
            }else {
                if ((1 - in.location.x) * ui_style.border[3] < (1 - in.location.y) * ui_style.border[2]  * ui_style.ratio){
                    final_color = ui_style.border_r_color;
                } else {
                    final_color = ui_style.border_b_color;
                }
            }
        }
    } else {
        final_color = ui_style.color;
    }

    return final_color;
}

