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
    let a0: RwLock<Vec<(Rect2D, RendererMeta2D, StandardMaterial2DComponent)>> =
        RwLock::new(Vec::new());
    let o0: RwLock<Vec<(Rect2D, RendererMeta2D, StandardMaterial2DComponent)>> =
        RwLock::new(Vec::new());
    let or0: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la0: AtomicU8 = AtomicU8::new(0);
    let r_WindowOptions: Res<WindowOptions> = Res::new(Default::default());
    let r_Renderer: Res<Renderer> = Res::new(Default::default());
    let r_Renderer2dData: Res<Renderer2dData> = Res::new(Default::default());
    let r_RenderGraph: Res<RenderGraph> = Res::new(Default::default());
    let mut loop_trigger = Trigger::new();
    let mut bus_test2_0 = Trigger::new();
    let mut bus_render_2d = Trigger::new();
    let mut test2_0_end = bus_test2_0.add_trigger();
    let mut render_2d_end = bus_render_2d.add_trigger();
    let mut ut_test2_0 = loop_trigger.add_trigger();
    let mut ut_render_2d = loop_trigger.add_trigger();
    thread::scope(|s: &Scope| {
        s.spawn(|| loop {
            ut_test2_0.read("failed");
            let o = test2_0();
            (&o0)
                .write()
                .unwrap()
                .extend(o.0.vec.into_iter().map(|(m0, m1, m2)| (m2, m0, m1)));
            bus_test2_0.trigger();
        });
        s.spawn(|| loop {
            ut_render_2d.read("failed");
            let o = render_2d(
                Arch::new(&mut render_2d0::new(&*a0.read().unwrap(), &or0)),
                r_Renderer2dData.clone(),
            );
            bus_render_2d.trigger();
        });
        if reset.load(SeqCst) {
            let mut bus_run_renderer = Trigger::new();
            let mut bus_start_2d_renderer = Trigger::new();
            let mut run_renderer_end = bus_run_renderer.add_trigger();
            let mut run_renderer_start_2d_renderer = bus_start_2d_renderer.add_trigger();
            let mut start_2d_renderer_end = bus_start_2d_renderer.add_trigger();
            thread::scope(|s: &Scope| {
                reset.store(false, Ordering::SeqCst);
                let handle_start_2d_renderer = s.spawn(|| {
                    let o = start_2d_renderer(r_RenderGraph.clone(), r_Renderer2dData.clone());
                    bus_start_2d_renderer.trigger();
                });
                let handle_run_renderer = s.spawn(|| {
                    run_renderer_start_2d_renderer.read("failed");
                    let o = run_renderer(
                        r_Renderer.clone(),
                        r_WindowOptions.clone(),
                        r_RenderGraph.clone(),
                    );
                    bus_run_renderer.trigger();
                });
                handle_start_2d_renderer
                    .join()
                    .expect("TODO: panic message");
                handle_run_renderer.join().expect("TODO: panic message");
            });
        }
        loop {
            let m_0 = s.spawn(|| {
                if la0.load(Ordering::SeqCst) > 0 {
                    return;
                }
                let mut write = a0.write().unwrap();
                let vlen = write.len();
                if vlen > 0 {
                    let indices_to_remove = take(&mut *or0.write().unwrap());
                    let mut new = Vec::with_capacity(vlen);
                    for (i, mut item) in write.drain(..).enumerate() {
                        if !indices_to_remove.contains(&i) {
                            new.push(item);
                            continue;
                        }
                    }
                    *write = new;
                }
                write.extend(o0.write().unwrap().drain(..));
            });
            signals
                .write()
                .unwrap()
                .extend(o_signals.write().unwrap().drain());
            *o_signals.write().unwrap() = HashSet::new();
            m_0 . join () . expect ("Failed to update archetype of type -> [\"Rect2D\", \"RendererMeta2D\", \"StandardMaterial2DComponent\"]") ;
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
            test2_0_end.read("failed");
            render_2d_end.read("failed");
        }
    });
}
