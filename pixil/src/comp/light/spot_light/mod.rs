#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct SpotLightData {
    pub(crate) position: [f32;4],
    pub(crate) direction: [f32;4],
    pub(crate) radius: f32,
    pub(crate) color_index:u32,
    pub(crate) inner_angle: f32,
    pub(crate) outer_angle: f32,
    pub(crate) attenuation: [f32;4],
}
