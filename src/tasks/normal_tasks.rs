pub mod normal_tasks {
    use crate::components::components::{Position1, Position2, Position3, Position4};
    use crate::ext::extras::{MarkedResources, StateExample};
    use crate::{TestUtArch, TestUtArch2};
    use corrosive_ecs_core::ecs_core::{Locked, LockedRef, Ref};
    use corrosive_ecs_core_macro::{task,add_entity};
    use rand::{Rng};
    use std::collections::HashSet;
    use std::sync::RwLock;
    use std::thread::sleep;
    use std::time::Duration;


    #[task("A string argument", setup())]
    pub fn setup() -> (
        Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
        Vec<(Locked<Position1>, LockedRef<Position3>)>,
    ) {
        let mut o1: Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)> = Vec::new();
        let mut o2: Vec<(Locked<Position1>, LockedRef<Position3>)> = Vec::new();

        let mut rng = rand::thread_rng();

        let random_number: u32 = rng.gen_range(0..10000);

        for i in 0..10000 {
            o1.push((
                Locked::new(if (random_number == i) {
                    Position1 { x: 10.0, y: 10.0 }
                } else {
                    Position1 { x: 2.0, y: 2.0 }
                }),
                Ref::new(Position2 { x: 2.0, y: 2.0 }),
                LockedRef::new(Position3 { x: 2.0, y: 2.0 }),
            ));
            o2.push((
                Locked::new(Position1 { x: 1.0, y: 1.0 }),
                LockedRef::new(Position3 { x: 2.0, y: 2.0 }),
            ));
        }
        add_entity!(Position2 { x: 1.0, y: 1.0 });
        (o1, o2)
    }

    pub fn setup1() -> (
        Vec<(Ref<Position2>, LockedRef<Position3>)>,
        Vec<(Ref<Position2>, Position4)>,
    ) {
        let mut o1: Vec<(Ref<Position2>, LockedRef<Position3>)> = Vec::new();
        let mut o2: Vec<(Ref<Position2>, Position4)> = Vec::new();
        for _i in 0..10000 {
            o1.push((
                Ref::new(Position2 { x: 1.0, y: 1.0 }),
                LockedRef::new(Position3 { x: 2.0, y: 2.0 }),
            ));
            o2.push((
                Ref::new(Position2 { x: 1.0, y: 1.0 }),
                Position4 { x: 2.0, y: 2.0 },
            ));
        }
        add_entity!(Position2 { x: 1.0, y: 1.0 });
        (o1, o2)
    }

    pub fn setup2() -> (Vec<Ref<Position2>>, Vec<Locked<Position1>>) {
        let mut o1: Vec<Ref<Position2>> = Vec::new();
        let mut o2: Vec<Locked<Position1>> = Vec::new();

        for _i in 0..10000 {
            o1.push(Ref::new(Position2 { x: 2.0, y: 2.0 }));
            o2.push(Locked::new(Position1 { x: 1.0, y: 1.0 }));
        }
        add_entity!(Position2 { x: 1.0, y: 1.0 });
        (o1, o2)
    }

    pub fn update_task(
        inp: TestUtArch,
        res: &RwLock<MarkedResources>,
        delta_time: &f64,
    ) -> (u8, HashSet<usize>, HashSet<usize>, HashSet<usize>) {
        let mut r1: HashSet<_> = HashSet::new();
        let mut r2: HashSet<_> = HashSet::new();
        let mut r3: HashSet<_> = HashSet::new();

        let mut mark: usize = 0;
        for x in inp {
            if x.read().unwrap().x == 10.0 {
                res.write().unwrap().0 = mark.clone();
                break;
            }
            mark += 1;
        }
        println!("{:?},{}", inp.len, delta_time);
        for i in mark..inp.len {
            if i < inp.ve1.len() {
                r1.insert(i);
                continue;
            };
            if i < inp.ve2.len() {
                r2.insert(i);
                continue;
            };
            if i < inp.ve3.len() {
                r3.insert(i);
                continue;
            };
        }

        (0b00000101, r1, r2, r3)
    }
    pub fn update_task_signal(inp: TestUtArch, inp2: TestUtArch2, sat: &RwLock<StateExample>) {
        let mut rng = rand::thread_rng();
        let random_number: usize = rng.gen_range(0..10000);

        *sat.write().unwrap() = StateExample::B;

        let mut mark: usize = 0;
        for x in inp {
            if mark == random_number {
                *x.write().unwrap() = Position1 { x: 10.0, y: 10.0 };
                break;
            }
            println!("{:?}", sat.read().unwrap());
            mark += 1;
        }
    }
    pub fn fixed_task() {
        println!("Fixed")
    }

    pub fn long_task(inp: TestUtArch2) ->bool {
        let mut reset = false;
        sleep(Duration::from_secs(2));
        println!("Long");
        reset = true;
        (reset)
    }

    pub fn sync_task(inp: TestUtArch2) {
        println!("sync")
    }
}
