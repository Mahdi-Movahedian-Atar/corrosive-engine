
use corrosive_ecs_core::ecs_core::{Member, Reference};
use corrosive_ecs_core_macro::Component;
use crate::comp::light::directional_light::{DirectionalLight, DirectionalLightData};
use crate::comp::position_pixil::PositionPixil;
use crate::task::renderer::DYNAMIC_LIGHTS;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PointLightData {
    pub(crate) position: [f32; 4],
    pub(crate) attenuation: [f32;4],
    pub(crate) radius: f32,
    pub(crate) pallet_index: u32,
    pub(crate) shade_mask: u32,
    pub(crate) cast_shadow_mask:u32
}


#[derive( Clone, Debug, Component)]
pub struct PointLight {
    pub(crate) data:PointLightData,
    pub(crate) index:u32,
    pub(crate) is_enabled:bool,
}
impl PointLight {
    pub fn new(position: &Member<PositionPixil>,radius: f32, intensity: f32,attenuation: [f32;3],pallet_index: u8,enable:bool) -> Self {
        let data= PointLightData{
            position: match &*position.f_read(){
                            Reference::Some(t) => {let d = t.global.to_scale_rotation_translation().2.to_array();[d[0],d[1],d[2],1.0]}
                        Reference::Expired => {[0.0,0.0,0.0,1.0]}
            },
            attenuation: [attenuation[0],attenuation[1],attenuation[2],intensity],
            radius,
            pallet_index: pallet_index as u32,
            shade_mask: 0,
            cast_shadow_mask: 0,
        };
        if enable{
            return Self{
                index: DYNAMIC_LIGHTS.add_point_light(&data),
                data,
                is_enabled: true,
            }
        }
        else {
            return Self{
                data,
                index: 0,
                is_enabled: false,
            }
        };
        todo!("mask");
    }
    pub fn disable(&mut self){
        self.is_enabled = false;
        DYNAMIC_LIGHTS.remove_point_light(self.index);
    }
    pub fn enable(&mut self){
        self.is_enabled = true;
        self.index = DYNAMIC_LIGHTS.add_point_light(&self.data);
    }
    pub fn set_pallet_index(&mut self,index:u8){
        self.data.pallet_index = index as u32;
        if self.is_enabled {
            DYNAMIC_LIGHTS.update_point_light(&self.data,self.index)
        }
    }
    pub fn set_intensity_index(&mut self,intensity:f32){
        self.data.attenuation[3] = intensity;
        if self.is_enabled {
            DYNAMIC_LIGHTS.update_point_light(&self.data,self.index)
        }
    }
    pub fn set_radius_index(&mut self,radius:f32){
        self.data.radius = radius;
        if self.is_enabled {
            DYNAMIC_LIGHTS.update_point_light(&self.data,self.index)
        }
    }
    pub fn set_attenuation_index(&mut self,attenuation:[f32;3]){
        self.data.attenuation[0] = attenuation[0];
        self.data.attenuation[1] = attenuation[1];
        self.data.attenuation[2] = attenuation[2];
        if self.is_enabled {
            DYNAMIC_LIGHTS.update_point_light(&self.data,self.index)
        }
    }
    pub fn get_pallet_index(&self) -> u8 {
        self.data.pallet_index as u8
    }
    pub fn get_intensity_index(&self) -> f32 {
        self.data.attenuation[3]
    }
    pub fn get_radius_index(& self) -> f32 {
        self.data.radius
    }
    pub fn get_attenuation_index(&mut self) -> [f32;3] {
        [self.data.attenuation[0],self.data.attenuation[1],self.data.attenuation[2]]
    }
}
impl Drop for  PointLight{
    fn drop(&mut self) {
        if self.is_enabled {
            DYNAMIC_LIGHTS.remove_point_light(self.index)
        }
    }
}