use glam::{EulerRot, Vec3};
use corrosive_ecs_core::ecs_core::{Member, Reference};
use corrosive_ecs_core_macro::Component;
use crate::comp::position_pixil::PositionPixil;
use crate::task::renderer::DYNAMIC_LIGHTS;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct DirectionalLightData{
    pub(crate) direction:[f32;4],
    pub(crate) intensity: f32,
    pub(crate) pallet_index:u32,
    pub(crate) _padding:[f32;2]
}

#[derive(Clone, Debug,Component)]
pub struct DirectionalLight{
    pub(crate) data:DirectionalLightData,
    pub(crate) index:u32,
    pub(crate) is_enabled:bool,
}
impl DirectionalLight{
    pub fn new(position:&Member<PositionPixil>,intensity:f32,pallet_index:u8,enable:bool)->Self{
        let data= DirectionalLightData{
            direction: match &*position.f_read(){
                Reference::Some(t) => {let d = (-t.global.col(2).truncate().normalize()).to_array();[d[0],d[1],d[2],0.0]}
                Reference::Expired => {[0.0,1.0,0.0,0.0]}
            },
            intensity,
            pallet_index: pallet_index as u32,
            _padding: [0.0,0.0],
        };
        if enable{
            return Self{
                index: DYNAMIC_LIGHTS.add_directional_light(&data),
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
        DYNAMIC_LIGHTS.remove_directional_light(self.index); 
    }
    pub fn enable(&mut self){
        self.is_enabled = true;
        self.index = DYNAMIC_LIGHTS.add_directional_light(&self.data);
    }
    pub fn set_pallet_index(&mut self,index:u8){
        self.data.pallet_index = index as u32;
        if self.is_enabled {
            DYNAMIC_LIGHTS.update_directional_light(&self.data,self.index)
        }
    }
    pub fn set_intensity_index(&mut self,intensity:f32){
        self.data.intensity = intensity;
        if self.is_enabled {
            DYNAMIC_LIGHTS.update_directional_light(&self.data,self.index)
        }
    }
    pub fn get_pallet_index(&self) -> u8 {
        self.data.pallet_index as u8
    }
    pub fn get_intensity_index(&self) -> f32 {
        self.data.intensity
    }
}
impl Drop for  DirectionalLight{
    fn drop(&mut self) {
        if self.is_enabled{
        DYNAMIC_LIGHTS.remove_directional_light(self.index)}
    }
}