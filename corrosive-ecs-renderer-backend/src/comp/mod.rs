use crate::helper::{BindGroupDescriptor, BindGroupEntry, BufferBindingType};
use crate::render_graph::GraphNode;
use crate::STATE;
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::{Component, Resource};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::JoinHandle;
use wgpu::hal::dx12::PipelineLayout;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    Buffer, BufferUsages, Device, RenderPipeline, ShaderStages, SurfaceTarget,
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window;
use winit::window::{Window, WindowId};

#[derive(Resource, Default)]
pub struct Renderer(pub Option<JoinHandle<()>>);

#[derive(Resource)]
pub struct WindowOptions {
    window: Option<Arc<Window>>,
    func: fn(&mut App, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent),
}
impl Default for WindowOptions {
    fn default() -> Self {
        WindowOptions {
            window: None,
            func: |app, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent| {
                match event {
                    WindowEvent::ActivationTokenDone { .. } => {}
                    WindowEvent::Resized(_) => {}
                    WindowEvent::Moved(_) => {}
                    WindowEvent::CloseRequested => {}
                    WindowEvent::Destroyed => {}
                    WindowEvent::DroppedFile(_) => {}
                    WindowEvent::HoveredFile(_) => {}
                    WindowEvent::HoveredFileCancelled => {}
                    WindowEvent::Focused(_) => {}
                    WindowEvent::KeyboardInput { .. } => {}
                    WindowEvent::ModifiersChanged(_) => {}
                    WindowEvent::Ime(_) => {}
                    WindowEvent::CursorMoved { .. } => {}
                    WindowEvent::CursorEntered { .. } => {}
                    WindowEvent::CursorLeft { .. } => {}
                    WindowEvent::MouseWheel { .. } => {}
                    WindowEvent::MouseInput { .. } => {}
                    WindowEvent::PinchGesture { .. } => {}
                    WindowEvent::PanGesture { .. } => {}
                    WindowEvent::DoubleTapGesture { .. } => {}
                    WindowEvent::RotationGesture { .. } => {}
                    WindowEvent::TouchpadPressure { .. } => {}
                    WindowEvent::AxisMotion { .. } => {}
                    WindowEvent::Touch(_) => {}
                    WindowEvent::ScaleFactorChanged { .. } => {}
                    WindowEvent::ThemeChanged(_) => {}
                    WindowEvent::Occluded(_) => {}
                    WindowEvent::RedrawRequested => {
                        if let Some(t) = &app.window_options.f_read().window {
                            t.request_redraw();
                            unsafe {
                                if let Some(t) = &STATE {
                                    t.render().unwrap()
                                }
                            }
                        }
                    }
                }
            },
        }
    }
}
impl WindowOptions {
    pub fn window(&self) -> &Window {
        self.window.as_ref().unwrap()
    }
}

#[derive(Resource, Default)]
pub struct RenderGraph {
    pub(crate) pass_names: HashMap<String, usize>,
    pub(crate) pass_nodes: HashMap<usize, GraphNode>,
    pub(crate) edges: Vec<(usize, usize)>,
    pub(crate) execution_levels: Vec<Vec<usize>>,
}

pub struct State<'a> {
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: Arc<wgpu::SurfaceConfiguration>,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    pub(crate) window: Arc<Window>,
    pub(crate) render_graph: Res<RenderGraph>,
    pub(crate) device: Device,
    pub(crate) resolution_buffer: Buffer,
    pub(crate) resolution_bind_group: BindGroup,
    pub(crate) resolution_bind_group_layout: BindGroupLayout,
}
impl<'a> State<'a> {
    async fn new(window: Arc<Window>, render_graph: Res<RenderGraph>) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .enumerate_adapters(wgpu::Backends::all())
            .into_iter()
            .filter(|adapter| adapter.is_surface_supported(&surface))
            .next()
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = Arc::new(wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        });

        surface.configure(&device, &config);

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: "resolution_buffer".into(),
            contents: &bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
            usage: BufferUsages::UNIFORM,
        });

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: "resolution_buffer_layout".into(),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: "resolution_bind_group".into(),
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: (buffer.as_entire_binding()),
            }],
        });

        State {
            surface,
            queue,
            config,
            size,
            window,
            render_graph,
            device,
            resolution_buffer: buffer,
            resolution_bind_group_layout: layout,
            resolution_bind_group: bind_group,
        }
    }
    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        /*let mut encoder = self
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });*/

        self.render_graph
            .f_read()
            .execute(&self.device, &self.queue, view);

        /*{
            // 1.
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(2, &self.light_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); // NEW!
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass
                .draw_mesh_instanced(&self.obj_model.meshes[0], 0..self.instances.len() as u32);

            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);

            use crate::model::DrawLight; // NEW!
            render_pass.set_pipeline(&self.light_render_pipeline); // NEW!
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            ); // NEW!

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw_light_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                &self.camera_bind_group,
                &self.light_bind_group, // NEW
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));*/
        output.present();

        Ok(())
    }
}

pub struct App {
    render_graph: Res<RenderGraph>,
    window_options: Res<WindowOptions>,
}

impl App {
    pub(crate) fn new(window_options: Res<WindowOptions>, render_graph: Res<RenderGraph>) -> App {
        App {
            render_graph,
            window_options,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window_options.f_write().window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));
        self.render_graph.f_write().prepare();

        if let Some(t) = &self.window_options.f_read().window {
            let state = pollster::block_on(State::new(t.clone(), self.render_graph.clone()));
            unsafe {
                STATE = Some(state);
            }
            t.request_redraw();
        } else {
            panic!("failed to run renderer backend")
        };
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let f = (self.window_options.f_read().func);
        f(self, event_loop, window_id, event)
    }
}
