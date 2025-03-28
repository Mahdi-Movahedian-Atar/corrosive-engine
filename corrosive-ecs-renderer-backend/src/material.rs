use crate::helper;

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
