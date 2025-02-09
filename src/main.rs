#![allow(warnings)]

use corrosive_ecs_core_macro::corrosive_engine_builder;
use std::sync::mpsc;

mod comp;
mod core_test;
#[path = ".corrosive_engine/mod.rs"]
mod corrosive_engine;

#[path = "corrosive-components/arch_types.rs"]
mod e;
mod task;

use crate::task::other_tasks::other_other_task::*;
use crate::task::other_tasks::*;
use crate::task::*;
use corrosive_engine::auto_prelude::*;

fn main() {
    use corrosive_ecs_core::ecs_core::Trigger;
    use std::cmp::PartialEq;
    use std::collections::HashSet;
    use std::mem::take;
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
    use std::sync::RwLock;
    use std::thread;
    use std::thread::{Scope, ScopedJoinHandle};
    use std::time::Instant;

    //corrosive_engine!(| update , sss|, | ss);
    corrosive_engine_builder!(
        path "./src",
        update "setup1",
        update "macro_test" in_group "group",
        fixed_update "setup" in_group "group" if !"sss" ||  State::new && !( Resorse{main: "mahdi"})
    );

    let mut last_time = Instant::now();
    let mut current_time = Instant::now();
    let delta_time = AtomicU64::new(0.0f64.to_bits());

    let fixed_time = AtomicU64::new(0.1f64.to_bits());
    let mut fixed_delta_time: u64 = 0.0f64 as u64;
    let is_fixed = AtomicBool::new(false);

    let reset: AtomicBool = AtomicBool::new(true);

    let a1: RwLock<
        Vec<(
            Locked<comp::Position1>,
            Ref<comp::Position2>,
            LockedRef<comp::sub::Position3>,
        )>,
    > = RwLock::new(Vec::new());
    let a2: RwLock<Vec<(Locked<Position1>, LockedRef<Position3>)>> = RwLock::new(Vec::new());
    let a3: RwLock<Vec<(Ref<Position2>, LockedRef<Position3>)>> = RwLock::new(Vec::new());
    let a4: RwLock<Vec<(Ref<Position2>, Position4)>> = RwLock::new(Vec::new());
    let a5: RwLock<Vec<(Ref<Position2>)>> = RwLock::new(Vec::new());
    let a6: RwLock<Vec<(Locked<Position1>,)>> = RwLock::new(Vec::new());

    let o1: RwLock<Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>> =
        RwLock::new(Vec::new());
    let o2: RwLock<Vec<(Locked<Position1>, LockedRef<Position3>)>> = RwLock::new(Vec::new());
    let o3: RwLock<Vec<(Ref<Position2>, LockedRef<Position3>)>> = RwLock::new(Vec::new());
    let o4: RwLock<Vec<(Ref<Position2>, Position4)>> = RwLock::new(Vec::new());
    let o5: RwLock<Vec<Ref<Position2>>> = RwLock::new(Vec::new());
    let o6: RwLock<Vec<(Locked<Position1>,)>> = RwLock::new(Vec::new());

    let or1: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let or2: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let or3: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let or4: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let or5: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
    let or6: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());

    let la1: AtomicU8 = AtomicU8::new(0);
    let la2: AtomicU8 = AtomicU8::new(0);
    let la3: AtomicU8 = AtomicU8::new(0);
    let la4: AtomicU8 = AtomicU8::new(0);
    let la5: AtomicU8 = AtomicU8::new(0);
    let la6: AtomicU8 = AtomicU8::new(0);

    let signal1: AtomicBool = AtomicBool::new(false);
    let signal2: AtomicBool = AtomicBool::new(false);
    let signal3: AtomicBool = AtomicBool::new(false);
    let signal_o1: AtomicBool = AtomicBool::new(false);
    let signal_o2: AtomicBool = AtomicBool::new(false);
    let signal_o3: AtomicBool = AtomicBool::new(false);

    let resources: RwLock<MarkedResources> = RwLock::new(MarkedResources::default());

    let state: RwLock<StateExample> = RwLock::new(StateExample::default());

    let mut loop_trigger = Trigger::new();

    let mut u1l = loop_trigger.add_trigger();
    let mut t_update_trigger = Trigger::new();

    let mut u2l = loop_trigger.add_trigger();
    let mut t_update_task_signal_trigger = Trigger::new();

    let mut u3l = loop_trigger.add_trigger();
    let mut t_long_task_trigger = Trigger::new();
    let mut aa = t_update_trigger.add_trigger();
    let mut ab = t_update_task_signal_trigger.add_trigger();

    let mut u4l = loop_trigger.add_trigger();
    let mut t_fixed_update_trigger = Trigger::new();

    let mut e1 = t_update_trigger.add_trigger();
    let mut e2 = t_update_task_signal_trigger.add_trigger();
    let mut e3 = t_long_task_trigger.add_trigger();
    let mut e4 = t_fixed_update_trigger.add_trigger();

    thread::scope(|s: &Scope| {
        let u1 = s.spawn(|| loop {
            u1l.read("failed");
            let o = update_task(
                Arch::new(&mut TestUtArch::new(
                    &*a1.read().unwrap(),
                    &*a2.read().unwrap(),
                    &*a6.read().unwrap(),
                    &or1,
                    &or2,
                    &or3,
                )),
                Res::new(&resources),
                &f64::from_bits(delta_time.load(Ordering::Relaxed)),
            );
            signal_o1.store(o.0, Ordering::Relaxed);
            signal_o2.store(o.1, Ordering::Relaxed);
            t_update_trigger.trigger();
        });

        let u2 = s.spawn(|| loop {
            u2l.read("failed");

            if (signal1.load(Ordering::Relaxed)
                && (signal2.load(Ordering::Relaxed)
                    || signal3.load(Ordering::Relaxed)
                        && *state.read().unwrap() == StateExample::A))
            {
                let o = update_task_signal(
                    TestUtArch::new(
                        &*a1.read().unwrap(),
                        &*a2.read().unwrap(),
                        &*a6.read().unwrap(),
                        &or1,
                        &or2,
                        &or3,
                    ),
                    TestUtArch2::new(&*a1.read().unwrap(), &*a3.read().unwrap()),
                    State::new(&state),
                );
            }
            t_update_task_signal_trigger.trigger();
        });

        let u3 = s.spawn(|| {
            let mut lu1: Option<ScopedJoinHandle<_>> = None::<ScopedJoinHandle<'_, _>>;
            loop {
                u3l.read("failed");
                aa.read("failed");
                ab.read("failed");

                match lu1.take() {
                    Some(task) if task.is_finished() => {
                        task.join().expect("Task finished but failed to join");
                    }
                    Some(task) => {
                        lu1 = Some(task);
                    }
                    None => {
                        lu1 = Some(s.spawn(|| {
                            la1.fetch_add(1, Ordering::SeqCst);
                            la3.fetch_add(1, Ordering::SeqCst);

                            let o = long_task(TestUtArch2::new(
                                &*a1.read().unwrap(),
                                &*a3.read().unwrap(),
                            ));

                            reset.store(o, Ordering::SeqCst);

                            la1.fetch_sub(1, Ordering::SeqCst);
                            la3.fetch_sub(1, Ordering::SeqCst);
                        }));
                    }
                }

                t_long_task_trigger.trigger();
            }
        });

        let u4 = s.spawn(|| loop {
            u4l.read("failed");
            if is_fixed.load(SeqCst) {
                let o = fixed_task();
            }
            t_fixed_update_trigger.trigger();
        });

        /*let lu1: Arc<RwLock<Option<ScopedJoinHandle<_>>>> =
            Arc::new(RwLock::new(None::<ScopedJoinHandle<_>>));
        let luo1 = Arc::clone(&lu1);*/

        loop {
            if reset.load(SeqCst) {
                let mut sb1 = Trigger::new();
                let mut ls1 = sb1.add_trigger();
                let mut sb2 = Trigger::new();
                let mut ls2 = sb2.add_trigger();
                let mut sb3 = Trigger::new();
                let mut ls3 = sb3.add_trigger();
                thread::scope(|s: &Scope| {
                    reset.store(false, Ordering::SeqCst);
                    {
                        let s1;
                        let s2;
                        let s3;
                        s1 = s.spawn(|| {
                            let o = setup();
                            (&o1).write().unwrap().extend(o.0);
                            (&o2).write().unwrap().extend(o.1);
                            sb1.trigger();
                        });
                        s2 = s.spawn(|| {
                            let o = setup1();
                            (&o3).write().unwrap().extend(o.0);
                            (&o4).write().unwrap().extend(o.1);
                            sb2.trigger();
                        });
                        s3 = s.spawn(|| {
                            let o = setup2();
                            (&o5).write().unwrap().extend(o.0);
                            (&o6).write().unwrap().extend(o.1);
                            sb3.trigger();
                        });
                        s1.join().expect("TODO: panic message");
                        s2.join().expect("TODO: panic message");
                        s3.join().expect("TODO: panic message");
                    }
                });
            }
            /*let lu1 = Arc::clone(&lu1);
            //loop init=============================================================================
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
            //loop treads===========================================================================
            let t1 = s.spawn(|| {
                let s1 = s.spawn(|| {
                    let o = update_task(
                        Arch::new(&mut TestUtArch::new(
                            &*a1.read().unwrap(),
                            &*a2.read().unwrap(),
                            &*a6.read().unwrap(),
                            &or1,
                            &or2,
                            &or3,
                        )),
                        Res::new(&resources),
                        &f64::from_bits(delta_time.load(Ordering::Relaxed)),
                    );
                    signal_o1.store(o.0, Ordering::Relaxed);
                    signal_o2.store(o.0, Ordering::Relaxed);
                });
                let u2 = if (signal1.load(Ordering::Relaxed)
                    && (signal2.load(Ordering::Relaxed)
                        || signal3.load(Ordering::Relaxed)
                            && *state.read().unwrap() == StateExample::A))
                {
                    Some(s.spawn(|| {
                        let o = update_task_signal(
                            TestUtArch::new(
                                &*a1.read().unwrap(),
                                &*a2.read().unwrap(),
                                &*a6.read().unwrap(),
                                &or1,
                                &or2,
                                &or3,
                            ),
                            TestUtArch2::new(&*a1.read().unwrap(), &*a3.read().unwrap()),
                            State::new(&state),
                        );
                    }))
                } else {
                    None
                };
                //inter join=======================================================================

                &s1.join();
                if let Some(o) = u2 {
                    o.join().expect("TODO: panic message");
                };
                //second thread =================================================================
                let t1 = s.spawn(|| {
                    let u1 = if is_fixed.load(SeqCst) {
                        Some(s.spawn(|| {
                            let o = fixed_task();
                        }))
                    } else {
                        None
                    };
                    let lu1 = (move || lu1)();
                    let mut lu1 = lu1.write().unwrap();

                    match lu1.take() {
                        Some(task) if task.is_finished() => {
                            task.join().expect("Task finished but failed to join");
                        }
                        Some(task) => {
                            *lu1 = Some(task);
                        }
                        None => {
                            *lu1 = Some(s.spawn(|| {
                                la1.fetch_add(1, Ordering::SeqCst);
                                la3.fetch_add(1, Ordering::SeqCst);

                                let o = long_task(TestUtArch2::new(
                                    &*a1.read().unwrap(),
                                    &*a3.read().unwrap(),
                                ));

                                reset.store(o, Ordering::SeqCst);

                                la1.fetch_sub(1, Ordering::SeqCst);
                                la3.fetch_sub(1, Ordering::SeqCst);
                            }));
                        }
                    }

                    {
                        let o = sync_task(TestUtArch2::new(
                            &*a1.read().unwrap(),
                            &*a3.read().unwrap(),
                        ));
                    }

                    //second inet join==============================================================
                    if let Some(o) = u1 {
                        o.join().expect("TODO: panic message");
                    };
                });
                t1.join().expect("TODO: panic message");
            });
            t1.join().expect("TODO: panic message");*/
            //loop joins==========================================================================

            {
                let m1 = s.spawn(|| {
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
                            item.2.expire();
                        }

                        *write = new;
                    }
                    write.extend(o1.write().unwrap().drain(..));
                });

                let m2 = s.spawn(|| {
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

                let m3 = s.spawn(|| {
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
                            item.0.expire();
                            item.1.expire();
                        }

                        *write = new;
                    }
                    write.extend(o3.write().unwrap().drain(..));
                });

                let m4 = s.spawn(|| {
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
                            item.0.expire();
                        }

                        *write = new;
                    }
                    write.extend(o4.write().unwrap().drain(..));
                });

                let m5 = s.spawn(|| {
                    if la5.load(Ordering::SeqCst) > 0 {
                        return;
                    }
                    let mut write = a5.write().unwrap();
                    let vlen = write.len();

                    if vlen > 0 {
                        let indices_to_remove = take(&mut *or5.write().unwrap());
                        let mut new = Vec::with_capacity(vlen);

                        for (i, mut item) in write.drain(..).enumerate() {
                            if !indices_to_remove.contains(&i) {
                                new.push(item);
                                continue;
                            }
                            item.expire();
                        }

                        *write = new;
                    }
                    write.extend(o5.write().unwrap().drain(..));
                });

                let m6 = s.spawn(|| {
                    if la6.load(Ordering::SeqCst) > 0 {
                        return;
                    }
                    let mut write = a6.write().unwrap();
                    let vlen = write.len();

                    if vlen > 0 {
                        let indices_to_remove = take(&mut *or6.write().unwrap());
                        let mut new = Vec::with_capacity(vlen);

                        for (i, mut item) in write.drain(..).enumerate() {
                            if !indices_to_remove.contains(&i) {
                                new.push(item);
                                continue;
                            }
                        }

                        *write = new;
                    }
                    write.extend(o6.write().unwrap().drain(..));
                });

                signal1.store(signal_o1.load(Ordering::Relaxed), Ordering::Relaxed);
                signal2.store(signal_o2.load(Ordering::Relaxed), Ordering::Relaxed);
                signal3.store(signal_o3.load(Ordering::Relaxed), Ordering::Relaxed);
                signal_o1.store(false, Ordering::Relaxed);
                signal_o2.store(false, Ordering::Relaxed);
                signal_o3.store(false, Ordering::Relaxed);

                m1.join().expect("TODO: panic message");
                m2.join().expect("TODO: panic message");
                m3.join().expect("TODO: panic message");
                m4.join().expect("TODO: panic message");
                m5.join().expect("TODO: panic message");
                m6.join().expect("TODO: panic message");
            }

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

            e1.read("failed");
            e2.read("failed");
            e3.read("failed");
            e4.read("failed");
        }
    })
}
