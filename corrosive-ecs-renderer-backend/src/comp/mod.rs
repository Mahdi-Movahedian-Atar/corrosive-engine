use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::Resource;
use std::sync::Arc;
use std::thread::JoinHandle;
use wgpu::SurfaceTarget;
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

            func: |App, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent| {
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
                        println!("sss");
                        if let Some(t) = &App.window_options.f_read().window {
                            t.request_redraw();
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

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Arc<Window>,
}

impl<'a> State<'a> {
    async fn new(&mut self, window: Arc<Window>) -> State<'a> {
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

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        State {
            surface,
            device,
            queue,
            config,
            size,
            window,
        }
    }
}

pub struct App<'a> {
    state: Option<State<'a>>,
    window_options: Res<WindowOptions>,
}

impl<'a> App<'a> {
    pub(crate) fn new(window_options: Res<WindowOptions>) -> App<'a> {
        App {
            state: None,
            window_options,
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window_options.f_write().window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));

        /*let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);*/
        if let Some(t) = &self.window_options.f_read().window {
            t.request_redraw();
        }
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
