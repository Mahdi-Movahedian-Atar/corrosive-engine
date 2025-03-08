use crate::comp::{UIBuffers, UIStyle, UIVertex};
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::comp::{RenderGraph, WindowOptions};
use corrosive_ecs_renderer_backend::helper::{
    create_bind_group, create_bind_group_layout, create_buffer_init, create_pipeline,
    create_pipeline_layout, create_shader_module, get_resolution_bind_group,
    get_resolution_bind_group_layout, get_surface_format, get_window_ratio, BindGroupEntry,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindGroupRenderable, BindingType,
    BlendComponent, BlendFactor, BlendOperation, BlendState, BufferBindingType, BufferUsages,
    Color, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, IndexFormat, LoadOp,
    Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderStage, StoreOp, TextureView, VertexRenderable, VertexState,
};
use corrosive_ecs_renderer_backend::render_graph::{CommandEncoder, Device, Queue, RenderNode};
use std::sync::Arc;

struct UIRenderNode {
    buffers: Res<UIBuffers>,
    pipeline: RenderPipeline,
}
impl RenderNode for UIRenderNode {
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
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, item.0.slice(..));
            render_pass.set_bind_group(0, &item.2, &[]);
            render_pass.draw(0..4, 0..1)
        }
    }
}
#[task]
pub fn setup_ui_pass(graph: Res<RenderGraph>, buffers: Res<UIBuffers>) {
    let a = create_shader_module("ui_shader", include_str!("ui_shader.wgsl"));
    /*let a = RenderPipelineBuilder::new("ui-pipeline", &a)
    .with_vertex_entry_point("vs_main")
    .with_color_format(window_options.f_read().config().format)
    .with_vertex_buffer(UIVertex::desc())
    .with_fragment_state(&a, "fs_main")
    .with_color_format(window_options.f_read().config().format)
    .with_fragment_targets(&[])
    .build();*/
    let h = create_bind_group_layout(&UIStyle::desc());
    graph.f_write().add_node(Box::new(UIRenderNode {
        buffers: buffers.clone(),
        pipeline: create_pipeline(&RenderPipelineDescriptor {
            label: "ui_pipeline".into(),
            layout: Some(&create_pipeline_layout(&PipelineLayoutDescriptor {
                label: "ui_pipeline_layout".into(),
                bind_group_layouts: &[&h],
                push_constant_ranges: &[],
            })),
            vertex: VertexState {
                module: &(a),
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
                module: &(a),
                entry_point: "fs_main".into(),
                compilation_options: Default::default(),
                targets: &[ColorTargetState {
                    format: get_surface_format(),
                    blend: BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,         // Source: Alpha
                            dst_factor: BlendFactor::OneMinusSrcAlpha, // Destination: 1 - Alpha
                            operation: BlendOperation::Add,            // Standard Alpha Blending
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
    }));
    graph.f_write().prepare();
    let v = create_buffer_init(
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
    let u = create_buffer_init(
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
    let b = create_bind_group(
        "ui_uniform_bind_group",
        &h,
        &[BindGroupEntry {
            binding: 0,
            resource: u.as_entire_binding(),
        }],
    );
    buffers.f_write().buffers.push(Arc::new((v, u, b)))
    /*buffers.f_write().buffers.push(Arc::new(create_buffer_init((
        "ui_vertex_buffer",
        bytemuck::cast_slice(&[
            UIVertex { position: [0, 0] },
            UIVertex { position: [1, 0] },
            UIVertex { position: [0, 1] },
        ]),
        BufferUsages::VERTEX,
    ),create_buffer_init(
        "ui_index_buffer",
        bytemuck::cast_slice(&[0,1,2]),
        BufferUsages::INDEX,
    ))))*/
}
