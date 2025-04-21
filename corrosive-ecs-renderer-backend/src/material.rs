use crate::assets::BindGroupLayoutAsset;
use crate::public_functions;
use corrosive_asset_manager::asset_server::Asset;

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
    fn get_bind_group_layout() -> wgpu::BindGroupLayout;
}
pub trait MaterialBindGroup {
    fn get_bind_group_layout() -> wgpu::BindGroupLayout;
    fn get_bind_group_data(&self) -> BindGroupData;
}

pub trait MaterialDesc {
    fn get_name_desc<'a>() -> &'a str;
    fn get_bind_group_layout_desc<'a>() -> Asset<BindGroupLayoutAsset>;
}
pub trait Material {
    fn get_bind_group(&self) -> &wgpu::BindGroup;
    fn get_shader(&self) -> (&str, String);
    fn get_name(&self) -> &str;
    fn get_bind_group_layout(&self) -> Asset<BindGroupLayoutAsset>;
}
