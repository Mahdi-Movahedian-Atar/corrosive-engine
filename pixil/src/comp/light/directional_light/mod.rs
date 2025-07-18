use crate::comp::position_pixil::PositionPixil;
use crate::task::renderer::DYNAMIC_LIGHTS;
use corrosive_ecs_core::ecs_core::{Member, Reference};
use corrosive_ecs_core_macro::Component;
use glam::{EulerRot, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod,Default)]
pub struct DirectionalLightData {
    pub(crate) projections: [[f32; 16];3],
    pub(crate) direction: [f32; 4],
    pub(crate) intensity: f32,
    pub(crate) pallet_index: u32,
    pub(crate) cast_shadow_mask: u32,
    pub(crate) shade_mask: u32,
}

#[derive(Clone, Debug, Component)]
pub struct DirectionalLight {
    pub(crate) data: DirectionalLightData,
    pub(crate) index: u32,
    pub is_enabled: bool,
    pub cast_shadow_mask: u32,
    pub cast_shadow: bool,
}
impl DirectionalLight {
    pub fn new(
        position: &Member<PositionPixil>,
        intensity: f32,
        pallet_index: u8,
        enable: bool,
        cast_shadow:bool,
        cast_shadow_mask: u32,
        shade_mask: u32,
    ) -> Self {
        let cast_shadow_mask_data  = if !cast_shadow{0} else { cast_shadow_mask };
        let data = DirectionalLightData {
            projections: [[0.0;16];3],
            direction: match &*position.f_read() {
                Reference::Some(t) => {
                    let d = (-t.global.col(2).truncate().normalize()).to_array();
                    [d[0], d[1], d[2], 0.0]
                }
                Reference::Expired => [0.0, 1.0, 0.0, 0.0],
            },
            intensity,
            pallet_index: pallet_index as u32,
            cast_shadow_mask: cast_shadow_mask_data,
            shade_mask,
        };
        if enable {
            return Self {
                index: DYNAMIC_LIGHTS.add_directional_light(&data),
                data,
                is_enabled: true,
                cast_shadow_mask,
                cast_shadow
            };
        } else {
            return Self {
                data,
                index: 0,
                is_enabled: false,
                cast_shadow_mask,
                cast_shadow
            };
        };
    }
    pub fn disable(&mut self) {
        self.is_enabled = false;
        DYNAMIC_LIGHTS.remove_directional_light(self.index);
    }
    pub fn enable(&mut self) {
        self.is_enabled = true;
        self.index = DYNAMIC_LIGHTS.add_directional_light(&self.data);
    }
    pub fn set_pallet_index(&mut self, index: u8) {
        self.data.pallet_index = index as u32;
        if self.is_enabled {
            DYNAMIC_LIGHTS.update_directional_light(&self.data, self.index)
        }
    }
    pub fn set_intensity_index(&mut self, intensity: f32) {
        self.data.intensity = intensity;
        if self.is_enabled {
            DYNAMIC_LIGHTS.update_directional_light(&self.data, self.index)
        }
    }
    pub fn set_cast_shadow(&mut self, state:bool){
        self.cast_shadow = state;
        self.cast_shadow_mask  = if !state{0} else { self.cast_shadow_mask} ;
        if self.is_enabled {
            DYNAMIC_LIGHTS.update_directional_light(&self.data, self.index)
        }
    }
    pub fn get_pallet_index(&self) -> u8 {
        self.data.pallet_index as u8
    }
    pub fn get_intensity_index(&self) -> f32 {
        self.data.intensity
    }
    pub fn cast_shadow_enables(&self) ->bool{ self.cast_shadow}
}
impl Drop for DirectionalLight {
    fn drop(&mut self) {
        if self.is_enabled {
            DYNAMIC_LIGHTS.remove_directional_light(self.index)
        }
    }
}
