use crate::comp::{App, Renderer, WindowOptions};
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::task;
use std::sync::Arc;
use std::thread;
use winit::application::ApplicationHandler;
use winit::event_loop::{ControlFlow, EventLoop};

#[task]
pub fn run_renderer(re: Res<Renderer>, window_options: Res<WindowOptions>) {
    if re.f_read().0.is_none() {
        re.f_write().0 = Some(thread::spawn(move || {
            env_logger::init();

            let mut event_loop_builder = EventLoop::<()>::with_user_event();
            #[cfg(all(target_os = "linux", feature = "x11"))]
            {
                use winit::platform::x11::EventLoopBuilderExtX11;
                event_loop_builder.with_any_thread(true);
            }
            #[cfg(all(target_os = "linux", feature = "wayland"))]
            {
                use winit::platform::wayland::EventLoopBuilderExtWayland;
                event_loop_builder.with_any_thread(true);
            }
            #[cfg(target_os = "windows")]
            {
                use winit::platform::windows::EventLoopBuilderExtWindows;
                event_loop_builder.with_any_thread(true);
            }

            let event_loop_builder = event_loop_builder.build().unwrap();

            event_loop_builder.set_control_flow(ControlFlow::Poll);

            let mut app = App::new(window_options);

            event_loop_builder.run_app(&mut app).unwrap();
        }));
    }
}
