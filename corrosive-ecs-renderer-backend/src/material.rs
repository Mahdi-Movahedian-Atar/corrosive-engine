use crate::assets::{BindGroupLayoutAsset, ShaderAsset};
use crate::helper;
use crate::helper::ShaderModule;
use corrosive_asset_manager::{Asset, AssetObject};
use corrosive_ecs_core::ecs_core::Res;

pub struct BindGroupData {
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}
impl BindGroupData {
    pub fn new(bind_group: wgpu::BindGroup, buffer: wgpu::Buffer) -> Self {
        Self { bind_group, buffer }
    }
    pub fn update(&self, material: impl MaterialData) {
        material.update_by_data(self);
    }
}
pub trait MaterialData {
    fn update_by_data(&self, material_data: &BindGroupData);
    fn get_bind_group_layout() -> helper::BindGroupLayout;
}
pub trait MaterialBindGroup {
    fn get_bind_group_layout() -> helper::BindGroupLayout;
    fn get_bind_group_data(&self) -> BindGroupData;
}

pub trait MaterialDesc {
    fn get_name_desc<'a>() -> &'a str;
    fn get_bind_group_layout_desc() -> Asset<BindGroupLayoutAsset>;
}
pub trait Material: AssetObject + MaterialDesc {
    fn get_bind_group(&self) -> &helper::BindGroup;
    fn get_shader(&self) -> &ShaderModule;
    fn get_name(&self) -> &str {
        Self::get_name_desc()
    }
    fn get_bind_group_layout(&self) -> Asset<BindGroupLayoutAsset> {
        Self::get_bind_group_layout_desc()
    }
}
