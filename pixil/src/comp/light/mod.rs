use corrosive_ecs_renderer_backend::color::Color;
use crate::ordered_set::OrderedSetTrait;

pub struct PointLight {
    pub(crate) color: Color,
    pub(crate) radius: f32,
    pub(crate) intensity: f32,
    pub(crate) id: u32,
}
impl PointLight {
    pub fn new(color: Color, radius: f32, intensity: f32) -> Self {
        Self {
            color,
            radius,
            intensity,
            id: 0,
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct LightData {
    pub position: [f32; 4],
    pub color: [f32; 4],
    pub radius: f32,
    pub intensity: f32,
}
impl OrderedSetTrait for LightData {
    const NAME: &'static str = "LightData";
}
