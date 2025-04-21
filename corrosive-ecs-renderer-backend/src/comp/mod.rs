use crate::render_graph::GraphNode;
use crate::wgpu::BindGroupEntry;
use crate::STATE;
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::{Component, Resource};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use wgpu::hal::dx12::PipelineLayout;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device,
    RenderPipeline, ShaderStages, SurfaceTarget,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window;
use winit::window::{Window, WindowId};

#[derive(Resource, Default)]
pub struct Renderer(pub Option<JoinHandle<()>>);

#[derive(Resource)]
pub struct WindowOptions {
    pub window: Option<Arc<Window>>,
    pub func:
        Vec<fn(&mut App, event_loop: &ActiveEventLoop, window_id: &WindowId, event: &WindowEvent)>,
}
impl Default for WindowOptions {
    fn default() -> Self {
        WindowOptions {
            window: None,
            func: vec![|app,
                        event_loop: &ActiveEventLoop,
                        window_id: &WindowId,
                        event: &WindowEvent| {
                match event {
                    WindowEvent::ActivationTokenDone { .. } => {}
                    WindowEvent::Resized(t) => unsafe {
                        if let Some(s) = &mut STATE {
                            s.resize(t)
                        }
                    },
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
            }],
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
    pub(crate) config: Arc<RwLock<wgpu::SurfaceConfiguration>>,
    pub(crate) size: PhysicalSize<u32>,
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

        let config = Arc::new(RwLock::new(wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }));

        surface.configure(&device, &config.read().unwrap());

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: "resolution_buffer".into(),
            contents: &bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
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

        self.render_graph
            .f_read()
            .execute(&self.device, &self.queue, view);

        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size.clone();
            self.config.write().unwrap().width = new_size.width.clone();
            self.config.write().unwrap().height = new_size.height.clone();
            self.surface
                .configure(&self.device, &self.config.read().unwrap());
            self.queue.write_buffer(
                &self.resolution_buffer,
                0,
                &bytemuck::cast_slice(&[new_size.width as f32, new_size.height as f32]),
            )
        }
    }
}

pub struct App {
    render_graph: Res<RenderGraph>,
    pub window_options: Res<WindowOptions>,
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
        let funcs = self.window_options.f_read().func.clone();
        funcs.iter().for_each(|f| {
            f(self, event_loop, &window_id, &event);
        });
    }
}
