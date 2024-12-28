pub mod other_tasks;

use crate::comp::sub::{MarkedResources, Position3, Position4, StateExample};
use crate::comp::{Position1, Position2};
use crate::{TestUtArch, TestUtArch2};
use corrosive_ecs_core::ecs_core::{Arch, DeltaTime, Locked, LockedRef, Ref, Res, State};
use corrosive_ecs_core::{add_entity, reset, signal};
use corrosive_ecs_core_macro::task;
use rand::Rng;
use std::collections::HashSet;
use std::sync::RwLock;

#[task]
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
    add_entity!(Locked<Position1>: Position2 { x: 1.0, y: 1.0 });
    (o1, o2)
}

#[task]
pub fn macro_test(
    a: Arch<(Ref<Position2>, LockedRef<Position3>)>,
    aa: Arch<(Ref<Position2>, LockedRef<Position3>)>,
    b: Arch<LockedRef<Position3>>,
    c: Res<MarkedResources>,
    d: State<StateExample>,
) -> (
    Vec<(Ref<Position2>, LockedRef<Position3>)>,
    Vec<(Ref<Position2>, Position4)>,
) {
    let mut o1: Vec<(Ref<Position2>, LockedRef<Position3>)> = Vec::new();
    let mut o2: Vec<(Ref<Position2>, Position4)> = Vec::new();
    add_entity!(Ref<Position2>: Position2 { x: 1.0, y: 1.0 },LockedRef<Position3>: Position2 { x: 1.0, y: 1.0 } );
    add_entity!(Ref<Position2>: Position2 { x: 1.0, y: 1.0 },Position3: Position2 { x: 1.0, y: 1.0 },LockedRef<Position3>: Position2 { x: 1.0, y: 1.0 } );
    add_entity!(LockedRef<Position3>: Position2 { x: 1.0, y: 1.0 } );
    signal!("sss");
    reset!();
    (o1, o2)
}
#[task]
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
    add_entity!(Position1: Position2 { x: 1.0, y: 1.0 });
    (o1, o2)
}
#[task]
pub fn setup2() -> (Vec<Ref<Position2>>, Vec<(Locked<Position1>,)>) {
    let mut o1: Vec<Ref<Position2>> = Vec::new();
    let mut o2: Vec<(Locked<Position1>,)> = Vec::new();

    for _i in 0..10000 {
        o1.push(Ref::new(Position2 { x: 2.0, y: 2.0 }));
        o2.push((Locked::new(Position1 { x: 1.0, y: 1.0 }),));
    }
    add_entity!(Position1: Position2{x:1.0,y:1.0} , Locked<(Position1)>: Position2{x:5.0,y:1.0});
    add_entity!(Position1: Position2 { x: 1.0, y: 1.0 });
    (o1, o2)
}
#[task]
pub fn update_task(inp: TestUtArch, res: &RwLock<MarkedResources>, delta_time: DeltaTime) -> (u8) {
    let mut mark: usize = 0;
    for x in inp {
        if x.0.read().unwrap().x == 10.0 {
            res.write().unwrap().0 = mark.clone();
            break;
        }
        mark += 1;
    }
    println!("{:?},{}", inp.len, delta_time);
    for i in mark..inp.len {
        inp.remove(i);
    }

    0b00000101
}
#[task]
pub fn update_task_signal(inp: TestUtArch, inp2: TestUtArch2, sat: &RwLock<StateExample>) {
    let mut rng = rand::thread_rng();
    let random_number: usize = rng.gen_range(0..10000);

    *sat.write().unwrap() = StateExample::B;

    let mut mark: usize = 0;
    for x in inp {
        if mark == random_number {
            *x.0.write().unwrap() = Position1 { x: 10.0, y: 10.0 };
            break;
        }
        println!("{:?}", sat.read().unwrap());
        mark += 1;
    }
}
#[task]
pub fn fixed_task() {
    println!("Fixed")
}
