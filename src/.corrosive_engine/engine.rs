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
    let a0: RwLock<Vec<(LockedRef<Position3>, Ref<Position2>)>> = RwLock::new(Vec::new());
    let o0: RwLock<Vec<(LockedRef<Position3>, Ref<Position2>)>> = RwLock::new(Vec::new());
    let or0: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la0: AtomicU8 = AtomicU8::new(0);
    let a1: RwLock<Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>> =
        RwLock::new(Vec::new());
    let o1: RwLock<Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>> =
        RwLock::new(Vec::new());
    let or1: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la1: AtomicU8 = AtomicU8::new(0);
    let a2: RwLock<Vec<(LockedRef<Position3>,)>> = RwLock::new(Vec::new());
    let o2: RwLock<Vec<(LockedRef<Position3>,)>> = RwLock::new(Vec::new());
    let or2: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let la2: AtomicU8 = AtomicU8::new(0);
    let st_StateExample: State<StateExample> = State::new(StateExample::default());
    let r_MarkedResources: Res<MarkedResources> = Res::new(MarkedResources::default());
    let mut loop_trigger = Trigger::new();
    let mut bus_macro_test = Trigger::new();
    let mut macro_test_end = bus_macro_test.add_trigger();
    let mut ut_macro_test = loop_trigger.add_trigger();
    thread::scope(|s: &Scope| {
        s.spawn(|| loop {
            ut_macro_test.read("failed");
            let o = macro_test(
                Arch::new(&mut macro_test0::new(
                    &*a0.read().unwrap(),
                    &or0,
                    &*a1.read().unwrap(),
                    &or1,
                    &*a2.read().unwrap(),
                    &or2,
                )),
                Arch::new(&mut macro_test1::new(
                    &*a0.read().unwrap(),
                    &or0,
                    &*a1.read().unwrap(),
                    &or1,
                )),
                Arch::new(&mut macro_test2::new(
                    &*a0.read().unwrap(),
                    &or0,
                    &*a1.read().unwrap(),
                    &or1,
                )),
                st_StateExample.clone(),
                r_MarkedResources.clone(),
            );
            (&o0)
                .write()
                .unwrap()
                .extend(o.0.vec.into_iter().map(|(m0, m1)| (m1, m0)));
            (&o1)
                .write()
                .unwrap()
                .extend(o.1.vec.into_iter().map(|(m0, m1, m2)| (m2, m1, m0)));
            (&o2)
                .write()
                .unwrap()
                .extend(o.2.vec.into_iter().map(|(m0,)| (m0,)));
            o_signals.write().unwrap().extend(o.3.vec);
            if o.4.get() {
                reset.store(o.4.get(), Ordering::Relaxed);
            }
            bus_macro_test.trigger();
        });
        if reset.load(SeqCst) {
            thread::scope(|s: &Scope| {
                reset.store(false, Ordering::SeqCst);
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
                        item.1.expire();
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
                        item.2.expire();
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
                        item.0.expire();
                    }
                    *write = new;
                }
                write.extend(o2.write().unwrap().drain(..));
            });
            signals
                .write()
                .unwrap()
                .extend(o_signals.write().unwrap().drain());
            *o_signals.write().unwrap() = HashSet::new();
            m_0 . join () . expect ("Failed to update archetype of type -> [\"LockedRef<Position3>\", \"Ref<Position2>\"]") ;
            m_1 . join () . expect ("Failed to update archetype of type -> [\"LockedRef<Position3>\", \"Position3\", \"Ref<Position2>\"]") ;
            m_2.join()
                .expect("Failed to update archetype of type -> [\"LockedRef<Position3>\"]");
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
            macro_test_end.read("failed");
        }
    });
}
