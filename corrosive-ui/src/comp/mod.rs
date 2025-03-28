use crate::style::Style;
use corrosive_asset_manager::Asset;
use corrosive_ecs_core::ecs_core::Ref;
use corrosive_ecs_core_macro::{Component, Resource};
use corrosive_ecs_renderer_backend::assets::PipelineAsset;
use corrosive_ecs_renderer_backend::helper;
use corrosive_ecs_renderer_backend::helper::{
    create_bind_group_layout, write_to_buffer, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindGroupRenderable, BindingType, Buffer, BufferAddress, BufferBindingType, ShaderStage,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexRenderable, VertexStepMode,
};
use corrosive_ecs_renderer_backend::material::{BindGroupData, MaterialData};
use std::sync::Arc;

pub mod screen;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct UIVertex {
    pub(crate) position: [f32; 2],
    pub(crate) location: [f32; 2],
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct UIStyle {
    pub(crate) border: [f32; 4],
    pub(crate) corner: [f32; 4],
    pub(crate) color: [f32; 4],
    pub(crate) border_l_color: [f32; 4],
    pub(crate) border_t_color: [f32; 4],
    pub(crate) border_r_color: [f32; 4],
    pub(crate) border_b_color: [f32; 4],
    pub(crate) ratio: f32,
    pub(crate) rotation: f32,
    pub(crate) center: [f32; 2],
}

#[derive(Resource, Default)]
pub struct UIBuffers {
    pub(crate) buffers: Vec<Arc<(Asset<PipelineAsset>, Buffer, BindGroupData)>>,
}
impl VertexRenderable for UIVertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<UIVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}
impl MaterialData for UIStyle {
    fn update_by_data(&self, material_data: &BindGroupData) {
        write_to_buffer(&material_data.buffer, 0, bytemuck::bytes_of(self));
    }

    fn get_bind_group_layout() -> helper::BindGroupLayout {
        create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: "UIStyle_Buffer_Layout".into(),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}

#[derive(Component)]
pub struct UIRenderMeta {
    pub(crate) buffers: Arc<(Asset<PipelineAsset>, Buffer, BindGroupData)>,
}
pub struct Rec{
    top_left:(f32,f32),
    bottom_right:(f32,f32)
}
#[derive(Component)]
pub struct UiData<'a> {
    pub style: Style<'a>,
    pub rec: Rec,
    pub modified: bool,
}