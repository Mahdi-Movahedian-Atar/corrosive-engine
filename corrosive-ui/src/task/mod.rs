use crate::comp::{UIBuffers, UIVertex};
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::comp::{RenderGraph, WindowOptions};
use corrosive_ecs_renderer_backend::helper::{create_pipeline, create_shader_module, IndexFormat, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, VertexRenderable, VertexState};
use corrosive_ecs_renderer_backend::render_graph::{CommandEncoder, Device, Queue, RenderNode};

struct UIRenderNode {
    buffers: Res<UIBuffers>,
    pipeline: RenderPipeline,
}
impl RenderNode for UIRenderNode {
    fn name(&self) -> &str {
        "UIRenderer"
    }

    fn execute(&self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&Default::default());

        render_pass.set_pipeline(&self.pipeline);

        println!("ssss");

        for item in &self.buffers.f_read().buffers {
            render_pass.set_vertex_buffer(0, item.0.slice(..));
            render_pass.set_index_buffer(item.1.slice(..), IndexFormat::Uint16);
            render_pass.set_bind_group(0, &item.2, &[]);
            render_pass.draw_indexed(0..3, 0, 0..1);
        }
    }
}
#[task]
pub fn setup_ui_pass(
    graph: Res<RenderGraph>,
    buffers: Res<UIBuffers>,
) {
    let a = create_shader_module("ui-shader", "../ui.wgsl");
    /*let a = RenderPipelineBuilder::new("ui-pipeline", &a)
        .with_vertex_entry_point("vs_main")
        .with_color_format(window_options.f_read().config().format)
        .with_vertex_buffer(UIVertex::desc())
        .with_fragment_state(&a, "fs_main")
        .with_color_format(window_options.f_read().config().format)
        .with_fragment_targets(&[])
        .build();*/
    graph.f_write().add_node(Box::new(UIRenderNode {
        buffers: buffers.clone(),
        pipeline: create_pipeline(&RenderPipelineDescriptor{
            label: "ui-pipeline".into(),
            layout: None,
            vertex: VertexState {
                module: &(a),
                entry_point: "vs-main".into(),
                compilation_options: Default::default(),
                buffers: &[UIVertex::desc()],
            },
            primitive: PrimitiveState{
                topology: Default::default(),
                strip_index_format: None,
                front_face: Default::default(),
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: Default::default(),
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: None,
            multiview: None,
            cache: None,
        }),
    }));
    graph.f_write().prepare();
}
