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
    let a0: RwLock<Vec<(Member<Position2D>, RendererMeta2D, Sprite2D)>> = RwLock::new(Vec::new());
    let o0: RwLock<Vec<(Member<Position2D>, RendererMeta2D, Sprite2D)>> = RwLock::new(Vec::new());
    let or0: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la0: AtomicU8 = AtomicU8::new(0);
    let a1: RwLock<Vec<(LockedRef<Camera2D>, Member<Position2D>)>> = RwLock::new(Vec::new());
    let o1: RwLock<Vec<(LockedRef<Camera2D>, Member<Position2D>)>> = RwLock::new(Vec::new());
    let or1: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la1: AtomicU8 = AtomicU8::new(0);
    let r_ActiveCamera2D: Res<ActiveCamera2D> = Res::new(Default::default());
    let r_Inputs: Res<Inputs> = Res::new(Default::default());
    let r_EguiObject: Res<EguiObject> = Res::new(Default::default());
    let r_WindowOptions: Res<WindowOptions> = Res::new(Default::default());
    let r_RenderGraph: Res<RenderGraph> = Res::new(Default::default());
    let r_Renderer: Res<Renderer> = Res::new(Default::default());
    let r_Renderer2dData: Res<Renderer2dData> = Res::new(Default::default());
    let h_Position2D: Hierarchy<Position2D> = Hierarchy::default();
    let mut loop_trigger = Trigger::new();
    let mut bus_move_camera = Trigger::new();
    let mut bus_render_2d = Trigger::new();
    let mut bus_update_position = Trigger::new();
    let mut move_camera_end = bus_move_camera.add_trigger();
    let mut render_2d_end = bus_render_2d.add_trigger();
    let mut update_position_end = bus_update_position.add_trigger();
    let mut ut_move_camera = loop_trigger.add_trigger();
    let mut ut_update_position = loop_trigger.add_trigger();
    let mut ut_render_2d = loop_trigger.add_trigger();
    thread::scope(|s: &Scope| {
        s.spawn(|| loop {
            ut_move_camera.read("failed");
            let o = move_camera(
                r_ActiveCamera2D.clone(),
                r_Inputs.clone(),
                &f64::from_bits(delta_time.load(Ordering::Relaxed)),
            );
            bus_move_camera.trigger();
        });
        s.spawn(|| loop {
            ut_update_position.read("failed");
            let o = update_position(
                Arch::new(&mut update_position0::new(&*a0.read().unwrap(), &or0)),
                Arch::new(&mut update_position1::new(&*a1.read().unwrap(), &or1)),
                r_ActiveCamera2D.clone(),
            );
            bus_update_position.trigger();
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
            let mut bus_init_camera = Trigger::new();
            let mut bus_start_events = Trigger::new();
            let mut bus_run_renderer = Trigger::new();
            let mut bus_test2_0 = Trigger::new();
            let mut bus_start_egui = Trigger::new();
            let mut bus_start_2d_renderer = Trigger::new();
            let mut init_camera_end = bus_init_camera.add_trigger();
            let mut init_camera_run_renderer = bus_run_renderer.add_trigger();
            let mut start_events_end = bus_start_events.add_trigger();
            let mut run_renderer_end = bus_run_renderer.add_trigger();
            let mut run_renderer_start_2d_renderer = bus_start_2d_renderer.add_trigger();
            let mut test2_0_end = bus_test2_0.add_trigger();
            let mut test2_0_run_renderer = bus_run_renderer.add_trigger();
            let mut start_egui_end = bus_start_egui.add_trigger();
            let mut start_egui_run_renderer = bus_run_renderer.add_trigger();
            let mut start_2d_renderer_end = bus_start_2d_renderer.add_trigger();
            thread::scope(|s: &Scope| {
                reset.store(false, Ordering::SeqCst);
                let handle_start_events = s.spawn(|| {
                    let o = start_events(r_WindowOptions.clone());
                    bus_start_events.trigger();
                });
                let handle_test2_0 = s.spawn(|| {
                    test2_0_run_renderer.read("failed");
                    let o = test2_0(
                        h_Position2D.clone(),
                        r_ActiveCamera2D.clone(),
                        r_ActiveCamera2D.clone(),
                    );
                    (&o0)
                        .write()
                        .unwrap()
                        .extend(o.0.vec.into_iter().map(|(m0, m1, m2)| (m0, m1, m2)));
                    (&o1)
                        .write()
                        .unwrap()
                        .extend(o.1.vec.into_iter().map(|(m0, m1)| (m1, m0)));
                    bus_test2_0.trigger();
                });
                let handle_start_egui = s.spawn(|| {
                    start_egui_run_renderer.read("failed");
                    let o = start_egui(
                        r_RenderGraph.clone(),
                        r_WindowOptions.clone(),
                        r_EguiObject.clone(),
                    );
                    bus_start_egui.trigger();
                });
                let handle_start_2d_renderer = s.spawn(|| {
                    let o = start_2d_renderer(r_RenderGraph.clone(), r_Renderer2dData.clone());
                    bus_start_2d_renderer.trigger();
                });
                let handle_init_camera = s.spawn(|| {
                    init_camera_run_renderer.read("failed");
                    let o = init_camera(r_ActiveCamera2D.clone());
                    bus_init_camera.trigger();
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
                handle_start_events.join().expect("TODO: panic message");
                handle_test2_0.join().expect("TODO: panic message");
                handle_start_egui.join().expect("TODO: panic message");
                handle_start_2d_renderer
                    .join()
                    .expect("TODO: panic message");
                handle_init_camera.join().expect("TODO: panic message");
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
                        item.0.expire();
                    }
                    *write = new;
                }
                write.extend(o0.write().unwrap().drain(..));
            });
            let m_1 = s.spawn(|| {
                if la1.load(Ordering::SeqCst) > 0 {
                    return;
                }
                let mut write = a1.write().unwrap();
                let vlen = write.len();
                if vlen > 0 {
                    let indices_to_remove = take(&mut *or1.write().unwrap());
                    let mut new = Vec::with_capacity(vlen);
                    for (i, mut item) in write.drain(..).enumerate() {
                        if !indices_to_remove.contains(&i) {
                            new.push(item);
                            continue;
                        }
                        item.0.expire();
                        item.1.expire();
                    }
                    *write = new;
                }
                write.extend(o1.write().unwrap().drain(..));
            });
            signals
                .write()
                .unwrap()
                .extend(o_signals.write().unwrap().drain());
            *o_signals.write().unwrap() = HashSet::new();
            m_0 . join () . expect ("Failed to update archetype of type -> [\"Member<Position2D>\", \"RendererMeta2D\", \"Sprite2D\"]") ;
            m_1 . join () . expect ("Failed to update archetype of type -> [\"LockedRef<Camera2D>\", \"Member<Position2D>\"]") ;
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
            let o = update_events(r_Inputs.clone());
            loop_trigger.trigger();
            move_camera_end.read("failed");
            update_position_end.read("failed");
            render_2d_end.read("failed");
        }
    });
}
