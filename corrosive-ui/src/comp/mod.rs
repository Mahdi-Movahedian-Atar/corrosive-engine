use crate::style::Style;
use corrosive_ecs_core::ecs_core::{LockedRef, Ref};
use corrosive_ecs_core_macro::{task, Component, Resource};
use corrosive_ecs_renderer_backend::helper::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindGroupRenderable, BindingType,
    Buffer, BufferAddress, BufferBindingType, ShaderStage, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexRenderable, VertexStepMode,
};
use std::sync::Arc;

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
    pub(crate) buffers: Vec<Arc<(Buffer, Buffer, BindGroup)>>,
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
impl BindGroupRenderable for UIStyle {
    fn desc<'a>() -> BindGroupLayoutDescriptor<'a> {
        BindGroupLayoutDescriptor {
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
        }
    }
}

#[derive(Component)]
pub struct UIRenderMeta {
    pub(crate) buffers: Arc<(Buffer, Buffer, BindGroup)>,
}
#[derive(Component)]
pub struct UiBox<'a> {
    pub z: u32,
    pub style: Style<'a>,
    pub children: Vec<Ref<UiBox<'a>>>,
    pub rerender: bool,
}

pub trait UiElement {
    fn place(&self, max_width: f32, max_height: f32) -> (f32, f32);
}
