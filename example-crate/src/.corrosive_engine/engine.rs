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
    let a0: RwLock<Vec<(Member<PositionPixil>, PixilDynamicObject)>> = RwLock::new(Vec::new());
    let o0: RwLock<Vec<(Member<PositionPixil>, PixilDynamicObject)>> = RwLock::new(Vec::new());
    let or0: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la0: AtomicU8 = AtomicU8::new(0);
    let a1: RwLock<Vec<(LockedRef<PixilCamera>, Member<PositionPixil>)>> = RwLock::new(Vec::new());
    let o1: RwLock<Vec<(LockedRef<PixilCamera>, Member<PositionPixil>)>> = RwLock::new(Vec::new());
    let or1: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la1: AtomicU8 = AtomicU8::new(0);
    let a2: RwLock<Vec<(Locked<DirectionalLight>, Member<PositionPixil>)>> =
        RwLock::new(Vec::new());
    let o2: RwLock<Vec<(Locked<DirectionalLight>, Member<PositionPixil>)>> =
        RwLock::new(Vec::new());
    let or2: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la2: AtomicU8 = AtomicU8::new(0);
    let a3: RwLock<Vec<(Locked<PointLight>, Member<PositionPixil>)>> = RwLock::new(Vec::new());
    let o3: RwLock<Vec<(Locked<PointLight>, Member<PositionPixil>)>> = RwLock::new(Vec::new());
    let or3: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la3: AtomicU8 = AtomicU8::new(0);
    let a4: RwLock<Vec<(Locked<SpotLight>, Member<PositionPixil>)>> = RwLock::new(Vec::new());
    let o4: RwLock<Vec<(Locked<SpotLight>, Member<PositionPixil>)>> = RwLock::new(Vec::new());
    let or4: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la4: AtomicU8 = AtomicU8::new(0);
    let r_RenderGraph: Res<RenderGraph> = Res::new(Default::default());
    let r_EguiObject: Res<EguiObject> = Res::new(Default::default());
    let r_WindowOptions: Res<WindowOptions> = Res::new(Default::default());
    let r_Renderer: Res<Renderer> = Res::new(Default::default());
    let r_PixilRenderSettings: Res<PixilRenderSettings> = Res::new(Default::default());
    let r_ActivePixilCamera: Res<ActivePixilCamera> = Res::new(Default::default());
    let r_Inputs: Res<Inputs> = Res::new(Default::default());
    let h_PositionPixil: Hierarchy<PositionPixil> = Hierarchy::default();
    let mut loop_trigger = Trigger::new();
    let mut bus_update_camera = Trigger::new();
    let mut bus_rotate_model = Trigger::new();
    let mut bus_update_pixil_position = Trigger::new();
    let mut update_camera_end = bus_update_camera.add_trigger();
    let mut rotate_model_end = bus_rotate_model.add_trigger();
    let mut update_pixil_position_end = bus_update_pixil_position.add_trigger();
    let mut ut_update_camera = loop_trigger.add_trigger();
    let mut ut_rotate_model = loop_trigger.add_trigger();
    let mut ut_update_pixil_position = loop_trigger.add_trigger();
    thread::scope(|s: &Scope| {
        s.spawn(|| loop {
            ut_update_camera.read("failed");
            let o = update_camera(r_ActivePixilCamera.clone(), r_PixilRenderSettings.clone());
            bus_update_camera.trigger();
        });
        s.spawn(|| loop {
            ut_rotate_model.read("failed");
            let o = rotate_model(
                Arch::new(&mut rotate_model0::new(&*a0.read().unwrap(), &or0)),
                &f64::from_bits(delta_time.load(Ordering::Relaxed)),
            );
            bus_rotate_model.trigger();
        });
        s.spawn(|| loop {
            ut_update_pixil_position.read("failed");
            let o = update_pixil_position(Arch::new(&mut update_pixil_position0::new(
                &*a0.read().unwrap(),
                &or0,
            )));
            bus_update_pixil_position.trigger();
        });
        if reset.load(SeqCst) {
            let mut bus_start_egui = Trigger::new();
            let mut bus_run_renderer = Trigger::new();
            let mut bus_start_events = Trigger::new();
            let mut bus_start_pixil_renderer = Trigger::new();
            let mut bus_pixil_test = Trigger::new();
            let mut start_egui_end = bus_start_egui.add_trigger();
            let mut start_egui_run_renderer = bus_run_renderer.add_trigger();
            let mut run_renderer_end = bus_run_renderer.add_trigger();
            let mut start_events_end = bus_start_events.add_trigger();
            let mut start_pixil_renderer_end = bus_start_pixil_renderer.add_trigger();
            let mut start_pixil_renderer_run_renderer = bus_run_renderer.add_trigger();
            let mut pixil_test_end = bus_pixil_test.add_trigger();
            let mut pixil_test_run_renderer = bus_run_renderer.add_trigger();
            thread::scope(|s: &Scope| {
                reset.store(false, Ordering::SeqCst);
                let handle_start_events = s.spawn(|| {
                    let o = start_events(r_WindowOptions.clone());
                    bus_start_events.trigger();
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
                let handle_start_pixil_renderer = s.spawn(|| {
                    start_pixil_renderer_run_renderer.read("failed");
                    let o = start_pixil_renderer(
                        r_PixilRenderSettings.clone(),
                        r_ActivePixilCamera.clone(),
                        r_RenderGraph.clone(),
                        r_WindowOptions.clone(),
                    );
                    bus_start_pixil_renderer.trigger();
                });
                let handle_pixil_test = s.spawn(|| {
                    pixil_test_run_renderer.read("failed");
                    let o = pixil_test(
                        h_PositionPixil.clone(),
                        r_ActivePixilCamera.clone(),
                        r_WindowOptions.clone(),
                    );
                    (&o0)
                        .write()
                        .unwrap()
                        .extend(o.0.vec.into_iter().map(|(m0, m1)| (m1, m0)));
                    (&o1)
                        .write()
                        .unwrap()
                        .extend(o.1.vec.into_iter().map(|(m0, m1)| (m0, m1)));
                    (&o2)
                        .write()
                        .unwrap()
                        .extend(o.2.vec.into_iter().map(|(m0, m1)| (m0, m1)));
                    (&o3)
                        .write()
                        .unwrap()
                        .extend(o.3.vec.into_iter().map(|(m0, m1)| (m0, m1)));
                    (&o4)
                        .write()
                        .unwrap()
                        .extend(o.4.vec.into_iter().map(|(m0, m1)| (m0, m1)));
                    bus_pixil_test.trigger();
                });
                let handle_run_renderer = s.spawn(|| {
                    let o = run_renderer(
                        r_Renderer.clone(),
                        r_WindowOptions.clone(),
                        r_RenderGraph.clone(),
                    );
                    bus_run_renderer.trigger();
                });
                handle_start_events.join().expect("TODO: panic message");
                handle_start_egui.join().expect("TODO: panic message");
                handle_start_pixil_renderer
                    .join()
                    .expect("TODO: panic message");
                handle_pixil_test.join().expect("TODO: panic message");
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
            let m_2 = s.spawn(|| {
                if la2.load(Ordering::SeqCst) > 0 {
                    return;
                }
                let mut write = a2.write().unwrap();
                let vlen = write.len();
                if vlen > 0 {
                    let indices_to_remove = take(&mut *or2.write().unwrap());
                    let mut new = Vec::with_capacity(vlen);
                    for (i, mut item) in write.drain(..).enumerate() {
                        if !indices_to_remove.contains(&i) {
                            new.push(item);
                            continue;
                        }
                        item.1.expire();
                    }
                    *write = new;
                }
                write.extend(o2.write().unwrap().drain(..));
            });
            let m_3 = s.spawn(|| {
                if la3.load(Ordering::SeqCst) > 0 {
                    return;
                }
                let mut write = a3.write().unwrap();
                let vlen = write.len();
                if vlen > 0 {
                    let indices_to_remove = take(&mut *or3.write().unwrap());
                    let mut new = Vec::with_capacity(vlen);
                    for (i, mut item) in write.drain(..).enumerate() {
                        if !indices_to_remove.contains(&i) {
                            new.push(item);
                            continue;
                        }
                        item.1.expire();
                    }
                    *write = new;
                }
                write.extend(o3.write().unwrap().drain(..));
            });
            let m_4 = s.spawn(|| {
                if la4.load(Ordering::SeqCst) > 0 {
                    return;
                }
                let mut write = a4.write().unwrap();
                let vlen = write.len();
                if vlen > 0 {
                    let indices_to_remove = take(&mut *or4.write().unwrap());
                    let mut new = Vec::with_capacity(vlen);
                    for (i, mut item) in write.drain(..).enumerate() {
                        if !indices_to_remove.contains(&i) {
                            new.push(item);
                            continue;
                        }
                        item.1.expire();
                    }
                    *write = new;
                }
                write.extend(o4.write().unwrap().drain(..));
            });
            signals
                .write()
                .unwrap()
                .extend(o_signals.write().unwrap().drain());
            *o_signals.write().unwrap() = HashSet::new();
            m_0 . join () . expect ("Failed to update archetype of type -> [\"Member<PositionPixil>\", \"PixilDynamicObject\"]") ;
            m_1 . join () . expect ("Failed to update archetype of type -> [\"LockedRef<PixilCamera>\", \"Member<PositionPixil>\"]") ;
            m_2 . join () . expect ("Failed to update archetype of type -> [\"Locked<DirectionalLight>\", \"Member<PositionPixil>\"]") ;
            m_3 . join () . expect ("Failed to update archetype of type -> [\"Locked<PointLight>\", \"Member<PositionPixil>\"]") ;
            m_4 . join () . expect ("Failed to update archetype of type -> [\"Locked<SpotLight>\", \"Member<PositionPixil>\"]") ;
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
            update_camera_end.read("failed");
            rotate_model_end.read("failed");
            update_pixil_position_end.read("failed");
        }
    });
}
