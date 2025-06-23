
use corrosive_ecs_core::ecs_core::Member;
use crate::comp::position_pixil::PositionPixil;

pub struct PointLight {
    pub radius: f32,
    pub intensity: f32,
    pub pallet_index:u16,
    pub cast_shadow_mask:u16,
    pub shade_mask:u16,
    pub(crate) id: u32,
}
impl PointLight {
    pub fn new(position: Member<PositionPixil>,radius: f32, intensity: f32,pallet: u16,cast_shadow: u16,shade_mask:u16) -> Self {
        Self {
            radius,
            intensity,
            pallet_index: 0,
            cast_shadow_mask: 0,
            shade_mask: 0,
            id: 0,
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PointLightData {
    pub position: [f32; 4],
    pub radius: f32,
    pub intensity: f32,
    pub color_index: u32,
    pub shade_mask: u16,
    pub cast_shadow_mask:u16
}