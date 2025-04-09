use crate::comp::{Mesh2D, Renderer2dData, RendererMeta2D};
use corrosive_ecs_core::ecs_core::{Arch, Res};
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::comp::RenderGraph;
use corrosive_ecs_renderer_backend::helper;
use corrosive_ecs_renderer_backend::helper::{
    Color, LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
};
use corrosive_ecs_renderer_backend::render_graph::{CommandEncoder, Device, Queue, RenderNode};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;

pub(crate) struct UnsafeRenderPass {
    ptr: NonNull<RenderPass<'static>>,
    _marker: PhantomData<&'static mut RenderPass<'static>>,
}
unsafe impl Send for UnsafeRenderPass {}
unsafe impl Sync for UnsafeRenderPass {}
struct Renderer2DNode {
    rx: Receiver<()>,
    tx: Sender<UnsafeRenderPass>,
}
impl RenderNode for Renderer2DNode {
    fn name(&self) -> &str {
        "Renderer2D"
    }

    fn execute(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &helper::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("2D Render Pass"),
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
        let ptr = NonNull::from(unsafe {
            std::mem::transmute::<&mut RenderPass<'_>, &mut RenderPass<'static>>(&mut render_pass)
        }) as NonNull<RenderPass<'static>>;

        let unsafe_pass = UnsafeRenderPass {
            ptr,
            _marker: PhantomData,
        };

        self.tx.send(unsafe_pass).unwrap();
        self.rx.recv().unwrap();
    }
}

#[task]
pub fn start_2d_renderer(graph: Res<RenderGraph>, renderer2d_data: Res<Renderer2dData>) {
    let (render_pass_tx, render_pass_rx) = unbounded::<UnsafeRenderPass>();
    let (end_tx, end_rx) = unbounded::<()>();
    renderer2d_data.f_write().data = Some((render_pass_rx, end_tx));
    graph.f_write().add_node(Box::new(Renderer2DNode {
        tx: render_pass_tx,
        rx: end_rx,
    }));
}
#[task]
pub fn render_2d(meta: Arch<(&dyn Mesh2D, &RendererMeta2D)>, renderer2d_data: Res<Renderer2dData>) {
    if let Some(data) = &renderer2d_data.f_read().data {
        {
            let render_pass = unsafe { data.0.recv().unwrap().ptr.as_ptr().as_mut().unwrap() };
            meta.iter().for_each(|x| {
                /*render_pass.set_pipeline(&x.2.pipeline_asset.get().layout);
                render_pass.set_bind_group(0, &x.2.transform_data.1, &[]);
                render_pass.set_bind_group(2, x.1.get_bind_group(), &[]);*/
                x.0.update(render_pass);
            })
        }
        data.1.send(()).unwrap()
    }
}
