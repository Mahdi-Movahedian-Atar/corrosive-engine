pub fn run_engine() {
    use crate::corrosive_engine::auto_prelude::*;
    use corrosive_ecs_core::ecs_core::*;
    use corrosive_ecs_core_macro::corrosive_engine_builder;
    use std::cmp::PartialEq;
    use std::collections::HashSet;
    use std::mem::take;
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
    use std::sync::mpsc;
    use std::sync::RwLock;
    use std::thread;
    use std::thread::{Scope, ScopedJoinHandle};
    use std::time::Instant;
    let mut signals = RwLock::new(HashSet::<String>::new());
    let mut o_signals = RwLock::new(HashSet::<String>::new());
    let mut last_time = Instant::now();
    let mut current_time = Instant::now();
    let delta_time = AtomicU64::new(0.0f64.to_bits());
    let fixed_time = AtomicU64::new(0.1f64.to_bits());
    let mut fixed_delta_time: u64 = 0.0f64 as u64;
    let is_fixed = AtomicBool::new(false);
    let reset: AtomicBool = AtomicBool::new(true);
    let r_WindowOptions: Res<WindowOptions> = Res::new(WindowOptions::default());
    let r_Renderer: Res<Renderer> = Res::new(Renderer::default());
    let r_RenderGraph: Res<RenderGraph> = Res::new(RenderGraph::default());
    let r_UIBuffers: Res<UIBuffers> = Res::new(UIBuffers::default());
    let mut loop_trigger = Trigger::new();
    thread::scope(|s: &Scope| {
        if reset.load(SeqCst) {
            let mut bus_run_renderer = Trigger::new();
            let mut bus_setup_ui_pass = Trigger::new();
            let mut run_renderer_end = bus_run_renderer.add_trigger();
            let mut setup_ui_pass_end = bus_setup_ui_pass.add_trigger();
            let mut setup_ui_pass_run_renderer = bus_run_renderer.add_trigger();
            thread::scope(|s: &Scope| {
                reset.store(false, Ordering::SeqCst);
                let handle_setup_ui_pass = s.spawn(|| {
                    setup_ui_pass_run_renderer.read("failed");
                    let o = setup_ui_pass(
                        r_RenderGraph.clone(),
                        r_WindowOptions.clone(),
                        r_UIBuffers.clone(),
                    );
                    bus_setup_ui_pass.trigger();
                });
                let handle_run_renderer = s.spawn(|| {
                    let o = run_renderer(
                        r_Renderer.clone(),
                        r_WindowOptions.clone(),
                        r_RenderGraph.clone(),
                    );
                    bus_run_renderer.trigger();
                });
                handle_setup_ui_pass.join().expect("TODO: panic message");
                handle_run_renderer.join().expect("TODO: panic message");
            });
        }
        loop {
            signals
                .write()
                .unwrap()
                .extend(o_signals.write().unwrap().drain());
            *o_signals.write().unwrap() = HashSet::new();
            current_time = Instant::now();
            let new_current_time = current_time
                .duration_since(last_time)
                .as_secs_f64()
                .to_bits();
            delta_time.store(new_current_time.clone(), Ordering::Relaxed);
            last_time = current_time;
            fixed_delta_time += new_current_time;
            if (fixed_time.load(Ordering::Relaxed) <= fixed_delta_time) {
                fixed_delta_time = 0;
                is_fixed.store(true, SeqCst);
            } else {
                is_fixed.store(false, SeqCst);
            }
            loop_trigger.trigger();
        }
    });
}
