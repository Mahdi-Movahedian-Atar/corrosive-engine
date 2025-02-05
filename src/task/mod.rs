pub mod other_tasks;

use crate::comp::sub::{MarkedResources, Position3, Position4, StateExample};
use crate::comp::{Position1, Position2};
use crate::{corrosive_engine, TestUtArch, TestUtArch2};
use corrosive_ecs_core::ecs_core::{Arch, DeltaTime, Locked, LockedRef, Ref, Res, State};
use corrosive_ecs_core::{add_entity, reset, signal};
use corrosive_ecs_core_macro::task;
use rand::Rng;

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
    add_entity!(Locked<Position1>= Locked::new(Position1 { x: 1.0, y: 1.0 }));
    (o1, o2)
}

#[task]
pub fn macro_test(
    b: Arch<(&LockedRef<Position3>,)>,
    a: Arch<(&LockedRef<Position3>, &Ref<Position2>)>,
    aa: Arch<(&Ref<Position2>, &LockedRef<Position3>)>,
    c: Res<MarkedResources>,
    d: State<StateExample>,
) {
    add_entity!(Ref<Position2> = Ref::new(Position2 { x: 1.0, y: 1.0 }),LockedRef<Position3>= LockedRef::new(Position3 { x: 1.0, y: 1.0 }) );
    add_entity!(Ref<Position2> = Ref::new(Position2 { x: 1.0, y: 1.0 }),Position3= Position3 { x: 1.0, y: 1.0 },LockedRef<Position3>= LockedRef::new(Position3 { x: 1.0, y: 1.0 }) );
    add_entity!(LockedRef<Position3> = LockedRef::new(Position3 { x: 1.0, y: 1.0 }) );
    signal!("sss");
    reset!();
}

/*pub fn macro_test(
    a: corrosive_engine::arch_types::arch_types::macro_test0,
    aa: corrosive_engine::arch_types::arch_types::macro_test1,
    b: corrosive_engine::arch_types::arch_types::macro_test2,
    c: corrosive_ecs_core::ecs_core::Res<MarkedResources>,
    d: corrosive_ecs_core::ecs_core::State<StateExample>,
) -> (
    Vec<(LockedRef<Position3>, Ref<Position2>)>,
    Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
    Vec<(LockedRef<Position3>,)>,
    bool,
    bool,
) {
    let mut engine_signal_trigger: bool = false;
    let mut engine_trigger_signal0: bool = false;
    let mut engine_add_arch2: Vec<(LockedRef<Position3>,)> = Vec::new();
    let mut engine_add_arch1: Vec<(LockedRef<Position3>, Position3, Ref<Position2>)> = Vec::new();
    let mut engine_add_arch0: Vec<(LockedRef<Position3>, Ref<Position2>)> = Vec::new();

    engine_add_arch0.push((
        LockedRef::new(Position3 { x: 1.0, y: 1.0 }),
        Ref::new(Position2 { x: 1.0, y: 1.0 }),
    ));
    engine_add_arch1.push((
        LockedRef::new(Position3 { x: 1.0, y: 1.0 }),
        Position3 { x: 1.0, y: 1.0 },
        Ref::new(Position2 { x: 1.0, y: 1.0 }),
    ));
    engine_add_arch2.push((LockedRef::new(Position3 { x: 1.0, y: 1.0 }),));
    engine_trigger_signal0 = true;
    let mut engine_signal_trigger = true;
    return (
        engine_add_arch0,
        engine_add_arch1,
        engine_add_arch2,
        engine_trigger_signal0,
        engine_signal_trigger,
    );
}
*/
//#[task]
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
    add_entity!(Position1 = Position2 { x: 1.0, y: 1.0 });
    (o1, o2)
}
//#[task]
pub fn setup2() -> (Vec<Ref<Position2>>, Vec<(Locked<Position1>,)>) {
    let mut o1: Vec<Ref<Position2>> = Vec::new();
    let mut o2: Vec<(Locked<Position1>,)> = Vec::new();

    for _i in 0..10000 {
        o1.push(Ref::new(Position2 { x: 2.0, y: 2.0 }));
        o2.push((Locked::new(Position1 { x: 1.0, y: 1.0 }),));
    }
    add_entity!(Position1= Position2{x:1.0,y:1.0} , Locked<(Position1)>= Position2{x:5.0,y:1.0});
    add_entity!(Position1 = Position2 { x: 1.0, y: 1.0 });
    (o1, o2)
}
//#[task]
pub fn update_task(
    inp: Arch<(&Locked<Position1>,)>,
    res: Res<MarkedResources>,
    delta_time: DeltaTime,
) -> (bool, bool) {
    let mut s1 = false;
    let mut s2 = false;
    let mut mark: usize = 0;
    let a = vec!["sss", "ddd"];
    for x in inp.iter() {
        if x.0.read().unwrap().x == 10.0 {
            res.write().unwrap().0 = mark.clone();
            break;
        }
        mark += 1;
    }
    println!("{:?},{}", inp.len(), delta_time);
    for i in mark..inp.len() {
        inp.remove(i);
    }

    (s1, s2)
}
//#[task]
pub fn update_task_signal(inp: TestUtArch, inp2: TestUtArch2, sat: State<StateExample>) {
    let mut rng = rand::thread_rng();
    let random_number: usize = rng.gen_range(0..10000);

    let mut mark: usize = 0;
    /*for x in &inp {
        if mark == random_number {
            *x.0.write().unwrap() = Position1 { x: 10.0, y: 10.0 };
            break;
        }
        println!("{:?}", sat.read().unwrap());
        mark += 1;
    }*/
}

//#[task]
pub fn fixed_task() {
    println!("Fixed")
}
