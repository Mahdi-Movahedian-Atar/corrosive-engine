use crate::comp::EguiObject;
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::comp::{RenderGraph, WindowOptions};
use corrosive_ecs_renderer_backend::public_functions::{
    get_device, get_surface_format, get_window_resolution,
};
use corrosive_ecs_renderer_backend::render_graph::RenderNode;
use corrosive_ecs_renderer_backend::wgpu;
use egui::Context;
use egui_wgpu::{Renderer as EguiRenderer, ScreenDescriptor};
use egui_winit::winit::event::WindowEvent;
use egui_winit::State as EguiState;
use std::sync::{LazyLock, Mutex};

static INPUT: LazyLock<Mutex<Vec<WindowEvent>>> = LazyLock::new(|| Default::default());

struct EguiNode {
    egui_object: Res<EguiObject>,
    window_options: Res<WindowOptions>,
    scale_factor: f32,
}
impl EguiNode {
    fn update(&self) -> egui::FullOutput {
        let mut lock = self.egui_object.f_write();
        match &mut lock.state {
            None => {
                panic!()
            }
            Some(t) => unsafe {
                let raw_input = t.0.take_egui_input(self.window_options.f_read().window());
                t.0.egui_ctx().run(raw_input, &mut t.1)
            },
        }
    }
}
impl RenderNode for EguiNode {
    fn name(&self) -> &str {
        "egui"
    }

    fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let egui_output = self.update();

        let mut egui_lock = self.egui_object.f_write();

        let input_enabled = egui_lock.input;
        if input_enabled {
            let window = self.window_options.f_read();
            let mut input_events = INPUT.lock().unwrap();

            if let Some(state) = &mut egui_lock.state {
                for event in input_events.drain(..) {
                    state.0.on_window_event(window.window(), &event);
                }
            }
        } else {
            INPUT.lock().unwrap().clear();
        }

        if let Some(renderer) = &mut egui_lock.renderer {
            for (texture_id, texture_delta) in egui_output.textures_delta.set {
                renderer.update_texture(device, queue, texture_id, &texture_delta);
            }
        }

        let shapes = {
            if let Some(state) = &egui_lock.state {
                state
                    .0
                    .egui_ctx()
                    .tessellate(egui_output.shapes, self.scale_factor)
            } else {
                Vec::new()
            }
        };

        let (width, height) = get_window_resolution();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: self.scale_factor,
        };

        if let Some(renderer) = &mut egui_lock.renderer {
            renderer.update_buffers(device, queue, encoder, &shapes, &screen_descriptor);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            renderer.render(
                &mut render_pass.forget_lifetime(),
                &shapes,
                &screen_descriptor,
            );
        }
    }
}

#[task]
pub fn start_egui(
    graph: Res<RenderGraph>,
    window_options: Res<WindowOptions>,
    egui_object: Res<EguiObject>,
) {
    let ctx = Context::default();
    {
        let mut lock = egui_object.f_write();
        lock.state = Some((
            EguiState::new(
                ctx.clone(),
                ctx.viewport_id(),
                window_options.f_read().window(),
                None,
                None,
                None,
            ),
            Box::new(|_| {}),
        ));
        lock.renderer = Some(EguiRenderer::new(
            get_device(),
            get_surface_format(),
            None,
            1,
            false,
        ));
    }
    window_options.f_write().func.push(|_, _, _, window_event| {
        INPUT.lock().unwrap().push(window_event.clone());
    });

    graph.f_write().add_node(Box::new(EguiNode {
        window_options: window_options.clone(),
        egui_object: egui_object.clone(),
        scale_factor: window_options.f_read().window().scale_factor() as f32,
    }));
    graph.f_write().add_dependency("Renderer2D", "egui");
    graph.f_write().prepare()
}
