use corrosive_ecs_core_macro::{Component, Resource};
use corrosive_ecs_renderer_backend::helper::{
    BindGroup, Buffer, BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexRenderable, VertexStepMode,
};
use std::sync::Arc;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct UIVertex {
    position: [u16; 2],
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct UIStyle {
    border: [u16; 4],
    corner: [u16; 4],
    color: [u16; 2],
}

#[derive(Resource, Default)]
pub struct UIBuffers {
    pub(crate) buffers: Vec<Arc<(Buffer, Buffer, BindGroup)>>,
}
/*pub struct UIBoxRenderable {
    pub vertexes: [UIVertex; 3],
    pub color: u32,
    pub border: u16,
    pub vertex_buffer: Buffer,
}*/
impl VertexRenderable for UIVertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<UIVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Uint16x2,
                offset: 0,
                shader_location: 0,
            }],
        }
    }
}
/*impl UniformRenderable for UIStyle {
    fn desc() -> BindGroupLayoutDescriptor {
        create_bind_group_layout_descriptor(
            Some("UIStyleBindGroupLayout"),
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: (ShaderStages::VERTEX_FRAGMENT),
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: BufferSize::new(size_of::<UIStyle>() as u64),
                },
                count: None,
            }],
        )
    }
}*/

#[derive(Component)]
pub struct UIRenderMeta {
    pub(crate) buffers: Arc<(Buffer, Buffer, BindGroup)>,
}

/*#[derive(Default)]
struct UIRendererable {
    tl: [u16; 2],
    br: [u16; 2],
    corners: [u16; 4],
    edges: [u16; 4],
    background_color: [f32; 3],
    border_color: [f32; 3],
}

#[derive(Default)]
pub enum LenType {
    PX(u16),
    PER(f32),
    #[default]
    None,
}

#[derive(Default)]
struct UIBox {
    pub name: String,
    width: LenType,
    height: LenType,
    left: LenType,
    right: LenType,
    top: LenType,
    bottom: LenType,
    corners: [LenType; 4],
    edges: [LenType; 4],
    background_color: [f32; 3],
    border_color: [f32; 3],
    visible: bool,
    children: Vec<LockedRef<UIBox>>,
    //element: SimpleBox,
}

impl UIBox {
    pub fn new(name: String) -> UIBox {
        UIBox {
            name,
            ..UIBox::default()
        }
    }
}
*/
