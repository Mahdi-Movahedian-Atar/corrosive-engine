use crate::comp::{App, RenderGraph, Renderer, WindowOptions};
use crate::slang::ShaderManager;
use crate::STATE;
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::task;
use std::thread;
use winit::event_loop::{ControlFlow, EventLoop};

#[task]
pub fn run_renderer(
    re: Res<Renderer>,
    window_options: Res<WindowOptions>,
    render_graph: Res<RenderGraph>,
) {
    #[cfg(debug_assertions)]
    {
        let manager = ShaderManager::new();
        manager.sync_shaders().expect("failed_to_sync_shaders");
    }
    unsafe { STATE = None }
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

            let mut app = App::new(window_options, render_graph);

            event_loop_builder.run_app(&mut app).unwrap();
        }));
        unsafe { while STATE.is_none() {} }
    }
}
