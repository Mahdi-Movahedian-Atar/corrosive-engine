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
    let a0: RwLock<Vec<(Locked<Position1>, LockedRef<Position3>, Ref<Position2>)>> =
        RwLock::new(Vec::new());
    let o0: RwLock<Vec<(Locked<Position1>, LockedRef<Position3>, Ref<Position2>)>> =
        RwLock::new(Vec::new());
    let or0: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la0: AtomicU8 = AtomicU8::new(0);
    let a1: RwLock<Vec<(Locked<Position1>, LockedRef<Position3>)>> = RwLock::new(Vec::new());
    let o1: RwLock<Vec<(Locked<Position1>, LockedRef<Position3>)>> = RwLock::new(Vec::new());
    let or1: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la1: AtomicU8 = AtomicU8::new(0);
    let st_StateExample: State<StateExample> = State::new(Default::default());
    let r_MarkedResources: Res<MarkedResources> = Res::new(Default::default());
    let mut loop_trigger = Trigger::new();
    let mut bus_update_task = Trigger::new();
    let mut bus_long_task = Trigger::new();
    let mut bus_update_task_signal = Trigger::new();
    let mut bus_fixed_task = Trigger::new();
    let mut update_task_end = bus_update_task.add_trigger();
    let mut long_task_end = bus_long_task.add_trigger();
    let mut update_task_signal_end = bus_update_task_signal.add_trigger();
    let mut update_task_signal_long_task = bus_long_task.add_trigger();
    let mut fixed_task_end = bus_fixed_task.add_trigger();
    let mut fixed_task_long_task = bus_long_task.add_trigger();
    let mut ut_update_task_signal = loop_trigger.add_trigger();
    let mut ut_fixed_task = loop_trigger.add_trigger();
    let mut ut_long_task = loop_trigger.add_trigger();
    let mut ut_update_task = loop_trigger.add_trigger();
    thread::scope(|s: &Scope| {
        s.spawn(|| loop {
            ut_update_task_signal.read("failed");
            update_task_signal_long_task.read("failed");
            if (signals.read().unwrap().contains("Signal1")
                && signals.read().unwrap().contains("signal2")
                || signals.read().unwrap().contains("signal3")
                    && *st_StateExample.f_read() == StateExample::A)
            {
                let o = update_task_signal(st_StateExample.clone());
            }
            bus_update_task_signal.trigger();
        });
        s.spawn(|| loop {
            ut_fixed_task.read("failed");
            fixed_task_long_task.read("failed");
            if is_fixed.load(SeqCst) {
                let o = fixed_task();
            }
            bus_fixed_task.trigger();
        });
        s.spawn(|| {
            let mut lock: Option<ScopedJoinHandle<_>> = None::<ScopedJoinHandle<'_, _>>;
            loop {
                ut_long_task.read("failed");
                match lock.take() {
                    Some(task) if task.is_finished() => {
                        task.join().expect("Task finished but failed to join");
                    }
                    Some(task) => {
                        lock = Some(task);
                    }
                    None => {
                        lock = Some(s.spawn(|| {
                            la0.fetch_add(1, Ordering::SeqCst);
                            la1.fetch_add(1, Ordering::SeqCst);
                            let o = long_task(Arch::new(&mut long_task0::new(
                                &*a0.read().unwrap(),
                                &or0,
                                &*a1.read().unwrap(),
                                &or1,
                            )));
                            la0.fetch_sub(1, Ordering::SeqCst);
                            la1.fetch_sub(1, Ordering::SeqCst);
                        }));
                    }
                }
                bus_long_task.trigger();
            }
        });
        s.spawn(|| loop {
            ut_update_task.read("failed");
            let o = update_task(
                Arch::new(&mut update_task0::new(
                    &*a0.read().unwrap(),
                    &or0,
                    &*a0.read().unwrap(),
                    &or0,
                    &*a1.read().unwrap(),
                    &or1,
                )),
                r_MarkedResources.clone(),
                &f64::from_bits(delta_time.load(Ordering::Relaxed)),
            );
            bus_update_task.trigger();
        });
        if reset.load(SeqCst) {
            let mut bus_setup1 = Trigger::new();
            let mut bus_setup2 = Trigger::new();
            let mut bus_setup = Trigger::new();
            let mut setup1_end = bus_setup1.add_trigger();
            let mut setup2_end = bus_setup2.add_trigger();
            let mut setup_end = bus_setup.add_trigger();
            thread::scope(|s: &Scope| {
                reset.store(false, Ordering::SeqCst);
                let handle_setup = s.spawn(|| {
                    let o = setup();
                    (&o0)
                        .write()
                        .unwrap()
                        .extend(o.0.vec.into_iter().map(|(m0, m1, m2)| (m0, m2, m1)));
                    (&o1)
                        .write()
                        .unwrap()
                        .extend(o.1.vec.into_iter().map(|(m0, m1)| (m0, m1)));
                    bus_setup.trigger();
                });
                let handle_setup1 = s.spawn(|| {
                    let o = setup1();
                    bus_setup1.trigger();
                });
                let handle_setup2 = s.spawn(|| {
                    let o = setup2();
                    bus_setup2.trigger();
                });
                handle_setup.join().expect("TODO: panic message");
                handle_setup1.join().expect("TODO: panic message");
                handle_setup2.join().expect("TODO: panic message");
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
                        item.1.expire();
                        item.2.expire();
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
            m_0 . join () . expect ("Failed to update archetype of type -> [\"Locked<Position1>\", \"LockedRef<Position3>\", \"Ref<Position2>\"]") ;
            m_1 . join () . expect ("Failed to update archetype of type -> [\"Locked<Position1>\", \"LockedRef<Position3>\"]") ;
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
            let o = sync_task(&f64::from_bits(delta_time.load(Ordering::Relaxed)));
            loop_trigger.trigger();
            update_task_signal_end.read("failed");
            fixed_task_end.read("failed");
            long_task_end.read("failed");
            update_task_end.read("failed");
        }
    });
}
