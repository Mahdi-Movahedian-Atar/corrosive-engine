pub mod other_tasks;

use crate::comp::sub::{MarkedResources, Position3, Position4, StateExample};
use crate::comp::{Position1, Position2};
use crate::corrosive_engine;
use corrosive_ecs_core::ecs_core::{Arch, DeltaTime, Locked, LockedRef, Ref, Res, State};
use corrosive_ecs_core::{add_entity, reset, signal};
use corrosive_ecs_core_macro::task;
use rand::Rng;

#[task]
pub fn setup() {
    let mut rng = rand::thread_rng();

    let random_number: u32 = rng.gen_range(0..10000);
    for i in 0..10000 {
        add_entity!(
            Locked<Position1>= Locked::new(if (random_number == i) {
                Position1 { x: 10.0, y: 10.0 }
            } else {
                Position1 { x: 2.0, y: 2.0 }
            }),
            Ref<Position2> = Ref::new(Position2 { x: 2.0, y: 2.0 }),
            LockedRef<Position3> = LockedRef::new(Position3 { x: 2.0, y: 2.0 })
        );
        add_entity!(
            Locked<Position1>=Locked::new(Position1 { x: 1.0, y: 1.0 }),
            LockedRef<Position3>=LockedRef::new(Position3 { x: 2.0, y: 2.0 })
        );
    }
}

#[task]
pub fn macro_test(
    b: Arch<(&LockedRef<Position3>,)>,
    a: Arch<(&LockedRef<Position3>, &Ref<Position2>)>,
    aa: Arch<(&Ref<Position2>, &LockedRef<Position3>)>,
    d: State<StateExample>,
    c: Res<MarkedResources>,
) {
    signal!("sss");
    add_entity!(Ref<Position2> = Ref::new(Position2 { x: 1.0, y: 1.0 }),LockedRef<Position3>= LockedRef::new(Position3 { x: 1.0, y: 1.0 }) );
    add_entity!(Ref<Position2> = Ref::new(Position2 { x: 1.0, y: 1.0 }),Position3= Position3 { x: 1.0, y: 1.0 },LockedRef<Position3>= LockedRef::new(Position3 { x: 1.0, y: 1.0 }) );
    add_entity!(LockedRef<Position3> = LockedRef::new(Position3 { x: 1.0, y: 1.0 }) );
    reset!();
}

#[task]
pub fn setup1() {
    for _i in 0..10000 {
        add_entity!(
            Ref<Position2>=Ref::new(Position2 { x: 1.0, y: 1.0 }),
            LockedRef<Position3>=ockedRef::new(Position3 { x: 2.0, y: 2.0 }));
        add_entity!(
            Ref<Position2>=Ref::new(Position2 { x: 1.0, y: 1.0 }),
            Position4=Position4 { x: 2.0, y: 2.0 });
    }
}
#[task]
pub fn setup2() {
    for _i in 0..10000 {
        add_entity!(Ref<Position2> = Ref::new(Position2 { x: 2.0, y: 2.0 }));
        add_entity!(Locked<Position1> = Locked::new(Position1 { x: 1.0, y: 1.0 }));
    }
}
#[task]
pub fn update_task(
    inp: Arch<(&Locked<Position1>,)>,
    res: Res<MarkedResources>,
    delta_time: DeltaTime,
) {
    let mut mark: usize = 0;
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
}
#[task]
pub fn update_task_signal(sat: State<StateExample>) {
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

#[task]
pub fn fixed_task() {
    println!("Fixed")
}
