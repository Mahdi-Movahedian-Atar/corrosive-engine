use crate::style::Wrap;
use crate::style::Wrap::Warp;
use crate::style::{Display, Overflow, Style, Val};
use corrosive_asset_manager::asset_server::Asset;
use corrosive_ecs_core::ecs_core::{Member, Ref, Reference, SharedBehavior};
use corrosive_ecs_core_macro::{Component, Resource};
use corrosive_ecs_renderer_backend::assets::PipelineAsset;
use corrosive_ecs_renderer_backend::helper;
use corrosive_ecs_renderer_backend::helper::{
    create_bind_group_layout, write_to_buffer, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindGroupRenderable, BindingType, Buffer, BufferAddress, BufferBindingType, ShaderStage,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexRenderable, VertexStepMode,
};
use corrosive_ecs_renderer_backend::material::{BindGroupData, MaterialData};
use std::cmp::PartialEq;
use std::sync::{Arc, RwLockReadGuard};

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
#[derive(Default)]
pub struct Rec {
    top_left: (f32, f32),
    bottom_right: (f32, f32),
}
#[derive(Component, Default)]
pub struct UiNode {
    pub style: Style,
    pub rec: Rec,
    pub size: (f32, f32),
    pub modified: bool,
    pub visibility: bool,
    pub z_index: f32,
}

/*pub fn calculate_size(node: &Member<UiNode<'_>>, parent_size: &(f32, f32)) -> (f32, f32) {
    if let Reference::Some(node_value) = &mut *node.dry_e_write("failed to read ui node") {
        let mut new_size = (
            node_value.style.min_width.to_f32_width(parent_size)
                - node_value.style.border_l.to_f32_width(parent_size)
                - node_value.style.border_r.to_f32_width(parent_size),
            node_value.style.min_height.to_f32_height(parent_size)
                - node_value.style.border_t.to_f32_height(parent_size)
                - node_value.style.border_b.to_f32_height(parent_size),
        );
        let max_size = (
            node_value.style.max_width.to_f32_width(parent_size)
                - node_value.style.border_l.to_f32_width(parent_size)
                - node_value.style.border_r.to_f32_width(parent_size),
            node_value.style.max_height.to_f32_height(parent_size)
                - node_value.style.border_t.to_f32_height(parent_size)
                - node_value.style.border_b.to_f32_height(parent_size),
        );

        match node_value.style.display {
            Display::Block => {
                node.get_children().iter().for_each(|x| {
                    let v = calculate_size(x, &max_size);
                    if v.0 > new_size.0 {
                        new_size.0 = v.0;
                    }
                    if v.1 > new_size.1 {
                        new_size.1 = v.1;
                    }
                });
            }
            Display::Flex => {
                let mut available_width = max_size.0.clone();
                let mut taken_width = 0.0;
                let mut available_height = max_size.1.clone();
                let mut taken_height = 0.0;
                if node_value.style.wrap == Warp {
                    node.get_children().iter().for_each(|x| {
                        let v = calculate_size(x, &(available_width, available_height));

                        if v.0 > available_width {
                            available_height -= taken_height;
                            taken_width = 0.0;
                        } else {
                            taken_width += v.0;
                            if taken_width > new_size.0 {
                                new_size.0 = taken_width.clone();
                            }
                        }
                        if v.1 > taken_height {
                            taken_height = v.1;
                        }
                    });
                    available_height -= taken_height;
                    new_size.1 += max_size.1 - available_height;
                } else {
                    node.get_children().iter().for_each(|x| {
                        let v = calculate_size(x, &(available_width, available_height));

                        available_width -= v.0;
                        if v.1 > taken_height {
                            taken_height = v.1;
                        }
                    });
                    new_size.1 += taken_height;
                    new_size.0 += max_size.0 - available_width;
                }
            }
            Display::Grid => {
                node.get_children().iter().for_each(|x| {
                    let v = calculate_size(
                        x,
                        &(
                            max_size.0 / node_value.style.grid_columns as f32,
                            max_size.1 / node_value.style.grid_rows as f32,
                        ),
                    );
                    if v.0 > new_size.0 {
                        new_size.0 = v.0;
                    }
                    if v.1 > new_size.1 {
                        new_size.1 = v.1;
                    }
                });
            }
            Display::Sticky => {}
            Display::None => {}
        }
        if new_size.0 > max_size.0 {
            new_size.0 = max_size.0;
        }
        if new_size.1 > max_size.1 {
            new_size.1 = max_size.1;
        }
        node_value.size = new_size.clone();
        new_size
    } else {
        (0.0, 0.0)
    }
}*/

impl SharedBehavior for UiNode {
    fn shaded_add_behavior(&mut self, parent: &Self) {
        if parent.visibility {
            if self.visibility {
                self.modified = self.modified;
            } else {
                self.modified = false
            }
        } else {
            self.visibility = false;
            self.modified = false;
        }
    }

    fn shaded_remove_behavior(&mut self) {
        if self.visibility {
            self.modified = true;
        } else {
            self.modified = false
        }
    }
}
