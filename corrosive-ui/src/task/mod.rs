use crate::comp::{UIBuffers, UIStyle, UIVertex, UiNode};
use corrosive_asset_manager::asset_server::{Asset, AssetServer};
use corrosive_ecs_core::ecs_core::{Hierarchy, Reference, Res};
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::assets::PipelineAsset;
use corrosive_ecs_renderer_backend::comp::{RenderGraph, WindowOptions};
use corrosive_ecs_renderer_backend::material::{BindGroupData, MaterialData};
use corrosive_ecs_renderer_backend::public_functions::*;
use corrosive_ecs_renderer_backend::render_graph::{CommandEncoder, Device, Queue, RenderNode};
use corrosive_ecs_renderer_backend::wgpu::*;
use std::sync::Arc;

struct UIRenderNode {
    buffers: Res<UIBuffers>,
}
impl<'a> RenderNode for UIRenderNode {
    fn name(&self) -> &str {
        "UIRenderer"
    }

    fn execute(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("UI Render Pass"),
            color_attachments: &[Option::from(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        for item in &self.buffers.f_read().buffers {
            render_pass.set_pipeline(&item.0.get().layout);
            render_pass.set_vertex_buffer(0, item.1.slice(..));
            render_pass.set_bind_group(0, &item.2.bind_group, &[]);
            render_pass.draw(0..4, 0..1)
        }
    }
}
#[task]
pub fn setup_ui_pass(graph: Res<RenderGraph>, buffers: Res<UIBuffers>) {
    let shader = create_shader_module("ui_shader", include_str!("ui_shader.wgsl"));
    let bind_group_layout = UIStyle::get_bind_group_layout();
    let vertex_buffer = create_buffer_init(
        "ui_vertex_buffer",
        bytemuck::cast_slice(&[
            UIVertex {
                position: [-0.5, 0.5],
                location: [0.0, 0.0],
            },
            UIVertex {
                position: [0.5, 0.5],
                location: [1.0, 0.0],
            },
            UIVertex {
                position: [-0.5, -0.5],
                location: [0.0, 1.0],
            },
            UIVertex {
                position: [0.5, -0.5],
                location: [1.0, 1.0],
            },
        ]),
        BufferUsages::VERTEX,
    );
    let uniform_buffer = create_buffer_init(
        "ui_uniform_buffer",
        bytemuck::cast_slice(&[UIStyle {
            border: [0.2, 0.1, 0.1, 0.2],
            corner: [0.1, 0.1, 0.4, 0.1],
            color: [1.0, 1.0, 1.0, 1.0],
            border_r_color: [0.1, 0.1, 1.0, 1.0],
            border_t_color: [0.1, 1.0, 0.1, 1.0],
            border_l_color: [1.0, 0.1, 0.1, 1.0],
            border_b_color: [0.1, 0.1, 0.1, 1.0],
            ratio: get_window_ratio(),
            center: [0f32, 0f32],
            rotation: 45.0,
        }]),
        BufferUsages::UNIFORM,
    );
    let uniform_bind_group = create_bind_group(
        "ui_uniform_bind_group",
        &bind_group_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    );

    let ass: Asset<PipelineAsset> = AssetServer::add(1, move || {
        Ok(PipelineAsset {
            layout: create_pipeline(&RenderPipelineDescriptor {
                label: "ui_pipeline".into(),
                layout: Some(&create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: "ui_pipeline_layout".into(),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                })),
                vertex: VertexState {
                    module: &(shader),
                    entry_point: "vs_main".into(),
                    compilation_options: Default::default(),
                    buffers: &[UIVertex::desc()],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: FrontFace::Cw,
                    cull_mode: Face::Back.into(),
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: FragmentState {
                    module: &(shader),
                    entry_point: "fs_main".into(),
                    compilation_options: Default::default(),
                    targets: &[ColorTargetState {
                        format: get_surface_format(),
                        blend: BlendState {
                            color: BlendComponent {
                                src_factor: BlendFactor::SrcAlpha,         // Source: Alpha
                                dst_factor: BlendFactor::OneMinusSrcAlpha, // Destination: 1 - Alpha
                                operation: BlendOperation::Add, // Standard Alpha Blending
                            },
                            alpha: BlendComponent {
                                src_factor: BlendFactor::One,              // Preserve Alpha
                                dst_factor: BlendFactor::OneMinusSrcAlpha, // Blend Based on Alpha
                                operation: BlendOperation::Add,
                            },
                        }
                        .into(),
                        write_mask: ColorWrites::ALL,
                    }
                    .into()],
                }
                .into(),
                multiview: None,
                cache: None,
            }),
        })
    });

    graph.f_write().add_node(Box::new(UIRenderNode {
        buffers: buffers.clone(),
    }));
    graph.f_write().prepare();
    buffers.f_write().buffers.push(Arc::new((
        ass,
        vertex_buffer,
        BindGroupData::new(uniform_bind_group, uniform_buffer),
    )))
}

#[task]
pub fn rerender_ui(nodes: Hierarchy<UiNode>) {
    nodes.get_roots().iter().for_each(|x| {
        if let Reference::Some(node) = &*x.read().unwrap() {
            if node.modified == true {
                for child in x.get_children() {}
            }
        }
    })
}
