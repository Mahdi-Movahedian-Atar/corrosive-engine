use crate::components::components::{Position1, Position2, Position3, Position4};
use crate::ext::extras::{MarkedResources, StateExample};
use crate::tasks::normal_tasks::normal_tasks::{
    fixed_task, long_task, setup, setup1, setup2, sync_task, update_task, update_task_signal,
};
use corrosive_ecs_core::ecs_core::{Locked, LockedRef, Ref};
use std::cmp::PartialEq;
use std::collections::HashSet;
use std::hash::RandomState;
use std::mem::take;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};
use std::sync::atomic::Ordering::SeqCst;
use std::thread;
use std::thread::{JoinHandle, Scope, ScopedJoinHandle};
use std::time::Instant;

mod components;
mod core_test;
mod ext;
mod tasks;

#[derive(Copy, Clone)]
struct TestUtArch<'a> {
    ve1: &'a Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
    ve2: &'a Vec<(Locked<Position1>, LockedRef<Position3>)>,
    ve3: &'a Vec<(Locked<Position1>)>,
    len: usize,
    v_i: usize,
}
impl<'a> TestUtArch<'a> {
    fn new(
        ve1: &'a Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
        ve2: &'a Vec<(Locked<Position1>, LockedRef<Position3>)>,
        ve3: &'a Vec<(Locked<Position1>)>,
    ) -> Self {
        TestUtArch {
            ve1,
            ve2,
            ve3,
            len: ve1.len() + ve2.len() + ve3.len(),
            v_i: 0,
        }
    }
}
impl<'a> Iterator for TestUtArch<'a> {
    type Item = (&'a Locked<Position1>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.v_i < self.ve1.len() {
            let current_index = self.v_i.clone();
            self.v_i += 1;
            return Some(&self.ve1[current_index].0);
        };
        if self.v_i < self.ve2.len() + self.ve1.len() {
            let current_index = self.v_i.clone() - self.ve1.len();
            self.v_i += 1;
            return Some(&self.ve2[current_index].0);
        };
        if self.v_i < self.ve3.len() + self.ve2.len() + self.ve1.len() {
            let current_index = self.v_i.clone() - self.ve1.len() - self.ve2.len();
            self.v_i += 1;
            return Some(&self.ve3[current_index]);
        };
        None
    }
}
#[derive(Copy, Clone)]
struct TestUtArch2<'a> {
    ve1: &'a Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
    ve2: &'a Vec<(Ref<Position2>, LockedRef<Position3>)>,
    len: usize,
    v_i: usize,
}
impl<'a> TestUtArch2<'a> {
    fn new(
        ve1: &'a Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
        ve2: &'a Vec<(Ref<Position2>, LockedRef<Position3>)>,
    ) -> Self {
        TestUtArch2 {
            ve1,
            ve2,
            len: ve1.len() + ve2.len(),
            v_i: 0,
        }
    }
}
impl<'a> Iterator for TestUtArch2<'a> {
    type Item = (&'a Ref<Position2>, &'a LockedRef<Position3>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.v_i < self.ve1.len() {
            let current_index = self.v_i.clone();
            self.v_i += 1;
            return Some((&self.ve1[current_index].1, &self.ve1[current_index].2));
        };
        if self.v_i < self.ve2.len() {
            let current_index = self.v_i.clone() - self.ve1.len();
            self.v_i += 1;
            return Some((&self.ve2[current_index].0, &self.ve2[current_index].1));
        };
        None
    }
}

fn main() {
    let mut last_time = Instant::now();
    let mut current_time = Instant::now();
    let mut delta_time = AtomicU64::new(0.0f64.to_bits());

    let mut fixed_time = AtomicU64::new(0.1f64.to_bits());
    let mut fixed_delta_time: u64 = 0.0f64 as u64;
    let mut is_fixed = RwLock::new(false);

    let reset: AtomicBool = AtomicBool::new(false);

    let mut a1: RwLock<Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>> =
        RwLock::new(Vec::new());
    let mut a2: RwLock<Vec<(Locked<Position1>, LockedRef<Position3>)>> = RwLock::new(Vec::new());
    let mut a3: RwLock<Vec<(Ref<Position2>, LockedRef<Position3>)>> = RwLock::new(Vec::new());
    let mut a4: RwLock<Vec<(Ref<Position2>, Position4)>> = RwLock::new(Vec::new());
    let mut a5: RwLock<Vec<(Ref<Position2>)>> = RwLock::new(Vec::new());
    let mut a6: RwLock<Vec<(Locked<Position1>)>> = RwLock::new(Vec::new());

    let o1: RwLock<Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>> =
        RwLock::new(Vec::new());
    let o2: RwLock<Vec<(Locked<Position1>, LockedRef<Position3>)>> = RwLock::new(Vec::new());
    let o3: RwLock<Vec<(Ref<Position2>, LockedRef<Position3>)>> = RwLock::new(Vec::new());
    let o4: RwLock<Vec<(Ref<Position2>, Position4)>> = RwLock::new(Vec::new());
    let o5: RwLock<Vec<Ref<Position2>>> = RwLock::new(Vec::new());
    let o6: RwLock<Vec<Locked<Position1>>> = RwLock::new(Vec::new());

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

    let mut signal: RwLock<u8> = RwLock::new(0);
    let signal_o: RwLock<u8> = RwLock::new(0);

    let resources: RwLock<MarkedResources> = RwLock::new(MarkedResources::default());

    let mut state: RwLock<StateExample> = RwLock::new(StateExample::default());
    let state_o: RwLock<StateExample> = RwLock::new(StateExample::default());


    loop {
        thread::scope(|s: &Scope| {
            reset.store(false,Ordering::SeqCst);
            {
                let s1;
                let s2;
                let s3;

                s1 = s.spawn(|| {
                    let o = setup();
                    (&o1).write().unwrap().extend(o.0);
                    (&o2).write().unwrap().extend(o.1);
                });
                s2 = s.spawn(|| {
                    let o = setup1();
                    (&o3).write().unwrap().extend(o.0);
                    (&o4).write().unwrap().extend(o.1);
                });
                s3 = s.spawn(|| {
                    let o = setup2();
                    (&o5).write().unwrap().extend(o.0);
                    (&o6).write().unwrap().extend(o.1);
                });
                s1.join().expect("TODO: panic message");
                s2.join().expect("TODO: panic message");
                s3.join().expect("TODO: panic message");
            }

            {
                let m1 = s.spawn(|| {
                    if la1.load(Ordering::SeqCst) > 0 {
                        return;
                    }
                    let mut write = a1.write().unwrap();
                    let vlen = write.len();

                    if vlen > 0 {
                        let mut indices_to_remove = take(&mut *or1.write().unwrap());
                        let mut new = Vec::with_capacity(vlen);

                        for (i, item) in write.drain(..).enumerate() {
                            if !indices_to_remove.contains(&i) {
                                new.push(item);
                            }
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
                        let mut indices_to_remove = take(&mut *or2.write().unwrap());
                        let mut new = Vec::with_capacity(vlen);

                        for (i, item) in write.drain(..).enumerate() {
                            if !indices_to_remove.contains(&i) {
                                new.push(item);
                            }
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
                        let mut indices_to_remove = take(&mut *or3.write().unwrap());
                        let mut new = Vec::with_capacity(vlen);

                        for (i, item) in write.drain(..).enumerate() {
                            if !indices_to_remove.contains(&i) {
                                new.push(item);
                            }
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
                        let mut indices_to_remove = take(&mut *or4.write().unwrap());
                        let mut new = Vec::with_capacity(vlen);

                        for (i, item) in write.drain(..).enumerate() {
                            if !indices_to_remove.contains(&i) {
                                new.push(item);
                            }
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
                        let mut indices_to_remove = take(&mut *or5.write().unwrap());
                        let mut new = Vec::with_capacity(vlen);

                        for (i, item) in write.drain(..).enumerate() {
                            if !indices_to_remove.contains(&i) {
                                new.push(item);
                            }
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
                        let mut indices_to_remove = take(&mut *or6.write().unwrap());
                        let mut new = Vec::with_capacity(vlen);

                        for (i, item) in write.drain(..).enumerate() {
                            if !indices_to_remove.contains(&i) {
                                new.push(item);
                            }
                        }

                        *write = new;
                    }
                    write.extend(o6.write().unwrap().drain(..));
                });

                m1.join().expect("TODO: panic message");
                m2.join().expect("TODO: panic message");
                m3.join().expect("TODO: panic message");
                m4.join().expect("TODO: panic message");
                m5.join().expect("TODO: panic message");
                m6.join().expect("TODO: panic message");
            }

            let lu1: Arc<RwLock<Option<ScopedJoinHandle<_>>>> =
                Arc::new(RwLock::new(None::<ScopedJoinHandle<_>>));

            loop  {
                let lu1 = Arc::clone(&lu1);
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
                    *is_fixed.write().unwrap() = true
                } else {
                    *is_fixed.write().unwrap() = false
                }
                //loop treads===========================================================================
                let t1 = s.spawn(|| {
                    let mut u1 = s.spawn(|| {
                        let o = update_task(
                            TestUtArch::new(
                                &*a1.read().unwrap(),
                                &*a2.read().unwrap(),
                                &*a6.read().unwrap(),
                            ),
                            &resources,
                            &f64::from_bits(delta_time.load(Ordering::Relaxed)),
                        );
                        *(&signal_o).write().unwrap() = o.0;
                        *&or1.write().unwrap().extend(o.1);
                        *&or2.write().unwrap().extend(o.2);
                        *&or6.write().unwrap().extend(o.3);
                    });
                    let mut u2 = if (*signal.read().unwrap() & 0b00000001 != 0
                        && (*signal.read().unwrap() & 0b00000010 != 0
                        || *signal.read().unwrap() & 0b00000100 != 0)
                        && *state.read().unwrap() == StateExample::A)
                    {
                        Some(s.spawn(|| {
                            let o = update_task_signal(
                                TestUtArch::new(
                                    &*a1.read().unwrap(),
                                    &*a2.read().unwrap(),
                                    &*a6.read().unwrap(),
                                ),
                                TestUtArch2::new(&*a1.read().unwrap(), &*a3.read().unwrap()),
                                &state_o,
                            );
                        }))
                    } else {
                        None
                    };
                    //inter join=======================================================================

                    u1.join().expect("TODO: panic message");
                    if let Some(o) = u2 {
                        o.join().expect("TODO: panic message");
                    };
                    //second thread =================================================================
                    let t1 = s.spawn(|| {
                        let mut u1 = if *(is_fixed.read().unwrap()) {
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
                            let o =
                                sync_task(TestUtArch2::new(&*a1.read().unwrap(), &*a3.read().unwrap()));;
                        }

                        //second inet join==============================================================
                        if let Some(o) = u1 {
                            o.join().expect("TODO: panic message");
                        };
                    });
                    t1.join().expect("TODO: panic message");
                });
                t1.join().expect("TODO: panic message");
                //loop joins==========================================================================
                {
                    let m1 = s.spawn(|| {
                        if la1.load(Ordering::SeqCst) > 0 {
                            return;
                        }
                        let mut write = a1.write().unwrap();
                        let vlen = write.len();

                        if vlen > 0 {
                            let mut indices_to_remove = take(&mut *or1.write().unwrap());
                            let mut new = Vec::with_capacity(vlen);

                            for (i, item) in write.drain(..).enumerate() {
                                if !indices_to_remove.contains(&i) {
                                    new.push(item);
                                }
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
                            let mut indices_to_remove = take(&mut *or2.write().unwrap());
                            let mut new = Vec::with_capacity(vlen);

                            for (i, item) in write.drain(..).enumerate() {
                                if !indices_to_remove.contains(&i) {
                                    new.push(item);
                                }
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
                            let mut indices_to_remove = take(&mut *or3.write().unwrap());
                            let mut new = Vec::with_capacity(vlen);

                            for (i, item) in write.drain(..).enumerate() {
                                if !indices_to_remove.contains(&i) {
                                    new.push(item);
                                }
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
                            let mut indices_to_remove = take(&mut *or4.write().unwrap());
                            let mut new = Vec::with_capacity(vlen);

                            for (i, item) in write.drain(..).enumerate() {
                                if !indices_to_remove.contains(&i) {
                                    new.push(item);
                                }
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
                            let mut indices_to_remove = take(&mut *or5.write().unwrap());
                            let mut new = Vec::with_capacity(vlen);

                            for (i, item) in write.drain(..).enumerate() {
                                if !indices_to_remove.contains(&i) {
                                    new.push(item);
                                }
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
                            let mut indices_to_remove = take(&mut *or6.write().unwrap());
                            let mut new = Vec::with_capacity(vlen);

                            for (i, item) in write.drain(..).enumerate() {
                                if !indices_to_remove.contains(&i) {
                                    new.push(item);
                                }
                            }

                            *write = new;
                        }
                        write.extend(o6.write().unwrap().drain(..));
                    });

                    *signal.write().unwrap() = *signal_o.read().unwrap();
                    *signal_o.write().unwrap() = 0u8;
                    *state.write().unwrap() = (*state_o.read().unwrap()).clone();

                    m1.join().expect("TODO: panic message");
                    m2.join().expect("TODO: panic message");
                    m3.join().expect("TODO: panic message");
                    m4.join().expect("TODO: panic message");
                    m5.join().expect("TODO: panic message");
                    m6.join().expect("TODO: panic message");

                    if reset.load(SeqCst) { break() }
                }
            }
        })
    }
}
