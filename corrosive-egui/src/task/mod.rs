use std::cell::LazyCell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::{Arc, LazyLock, Mutex, RwLockWriteGuard};
use egui::{Context, FontDefinitions, TexturesDelta};
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::comp::{RenderGraph, WindowOptions};
use corrosive_ecs_renderer_backend::render_graph::{CommandEncoder, Device, Queue, RenderNode};
use corrosive_ecs_renderer_backend::{wgpu, winit};
use corrosive_ecs_renderer_backend::public_functions::{get_device, get_surface_format, get_window_resolution};
use corrosive_ecs_renderer_backend::wgpu::{Color, LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, StoreOp};
use corrosive_ecs_renderer_backend::winit::event::Event;
use corrosive_ecs_renderer_backend::winit::window::Window;
use egui_winit::{State as EguiState, State};
use egui_wgpu::{Renderer as EguiRenderer, Renderer, ScreenDescriptor};
use crate::comp::EguiObject;

static mut TEXTURES: LazyCell<TexturesDelta> =
    LazyCell::new(|| Default::default());

struct EguiNode {
    egui_object: Res<EguiObject>,
    window_options: Res<WindowOptions>
}
impl EguiNode{
    fn update(&self) -> egui::FullOutput {
        let mut lock = self.egui_object.f_write();
        let mut state = match &mut lock.state {
            None => { panic!()}
            Some(t) => {t}
        };
        let raw_input = state.take_egui_input(self.window_options.f_read().window());
        state.egui_ctx().run(raw_input, |ui| {
            // Build your UI here
            egui::SidePanel::left("side_panel").show(ui, |ui| {
                ui.heading("Egui + wgpu");
                if ui.button("Click me!").clicked() {
                    println!("Button clicked!");
                }
            });
        })
    }
}
impl RenderNode for EguiNode {
    fn name(&self) -> &str {
        "egui"
    }

    fn execute(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut lock = self.egui_object.f_write();

        let a = {
            let mut state = match &mut lock.state {
                None => { panic!()}
                Some(t) => {t}
            };
            let raw_input = state.take_egui_input(self.window_options.f_read().window());
            state.egui_ctx().run(raw_input, |ui| {
                // Build your UI here
                egui::SidePanel::left("side_panel").show(ui, |ui| {
                    ui.heading("Egui + wgpu");
                    if ui.button("Click me!").clicked() {
                        println!("Button clicked!");
                    }
                });
            })
        };

        {
            let mut renderer = match &mut lock.renderer {
                None => { panic!() }
                Some(t) => { t }
            };

            for (id, delta) in a.textures_delta.set {
                renderer.update_texture(device, queue, id, &delta);
            }
        }

        let scale_factor = self.window_options.f_read().window().scale_factor() as f32;


        let shapes = {
            let mut state = match &mut lock.state {
                None => { panic!()}
                Some(t) => {t}
            };
            state.egui_ctx().tessellate(a.shapes, scale_factor)
        };

        let a = get_window_resolution();

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [a.0,a.1],
            pixels_per_point: scale_factor,
        };

        let mut renderer = match &mut lock.renderer {
            None => { panic!() }
            Some(t) => { t }
        };

        renderer.update_buffers(
            device,
            queue,
            encoder,
            &shapes,
            &screen_descriptor,
        );

            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Option::from(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            renderer.render(&mut render_pass.forget_lifetime(), &shapes, &screen_descriptor);
    }
}

#[task]
pub fn start_egui(graph: Res<RenderGraph>, window_options: Res<WindowOptions> ,egui_object: Res<EguiObject>) {
    let ctx = Context::default();
    {
        let mut lock = egui_object.f_write();
        lock.state = Some(EguiState::new(ctx.clone(), ctx.viewport_id(), window_options.f_read().window(), None, None, None));
        lock.renderer = Some(EguiRenderer::new(get_device(), get_surface_format(), None, 1, false))
    }

    graph.f_write().add_node(Box::new(EguiNode {
        window_options: window_options.clone(),
        egui_object: egui_object.clone()
    }));
    graph.f_write().prepare()
}