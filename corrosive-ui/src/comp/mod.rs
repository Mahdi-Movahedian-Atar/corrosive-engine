use corrosive_ecs_core_macro::{Component, Resource};
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

/*#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct resolution {
    resolution: [f32; 2],
}*/

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
