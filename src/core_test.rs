//pub mod custom_ecs {
use corrosive_ecs_core::ecs_core::{Locked, LockedRef};
use corrosive_ecs_core_macro::task;
use std::thread;
use std::time::Instant;

/*struct Position1 {
        x: f32,
        y: f32,
    }
    struct Position2 {
        x: f32,
        y: f32,
    }
    struct Position3 {
        x: f32,
        y: f32,
    }


    struct MCR (f32);
    struct ST {
        start: Instant,
        runs: usize,
    }


    struct ArchType1<'a> {
        ve: &'a Vec<(LockedRef<Position1>,LockedRef< Position2>,LockedRef< Position3>)>,
        index: usize
    }
    impl<'a> ArchType1<'a> {
        fn new(ve: &'a Vec<(LockedRef<Position1>, LockedRef<Position2>, LockedRef<Position3>)>) -> Self {
            ArchType1 { ve, index: 0 }
        }
    }

    impl<'a> Iterator for ArchType1<'a> {
        type Item = (usize, &'a LockedRef<Position1>, &'a LockedRef<Position2>);

        fn next(&mut self) -> Option<Self::Item> {
            if self.index < self.ve.len() {
                let current_index = self.index;
                self.index += 1;
                let item = &self.ve[current_index];
                Some((current_index, &item.0, &item.1))
            } else {
                None
            }
        }
    }

    struct ArchType2<'a> {
        ve: &'a Vec<(LockedRef<Position1>,LockedRef< Position2>,LockedRef< Position3>)>,
        index: usize
    }
    impl<'a> ArchType2<'a> {
        fn new(ve: &'a Vec<(LockedRef<Position1>, LockedRef<Position2>, LockedRef<Position3>)>) -> Self {
            ArchType2 { ve, index: 0 }
        }
    }

    impl<'a> Iterator for ArchType2<'a> {
        type Item = (usize, &'a LockedRef<Position3>, &'a LockedRef<Position2>);

        fn next(&mut self) -> Option<Self::Item> {
            if self.index < self.ve.len() {
                let current_index = self.index;
                self.index += 1;
                let item = &self.ve[current_index];
                Some((current_index, &item.2, &item.1))
            } else {
                None
            }
        }
    }
    struct ArchType3<'a> {
        ve: &'a Vec<(LockedRef<Position1>,LockedRef< Position2>,LockedRef< Position3>)>,
        index: usize
    }
    impl<'a> ArchType3<'a> {
        fn new(ve: &'a Vec<(LockedRef<Position1>, LockedRef<Position2>, LockedRef<Position3>)>) -> Self {
            ArchType3 { ve, index: 0 }
        }
    }

    impl<'a> Iterator for ArchType3<'a> {
        type Item = (usize, &'a LockedRef<Position1>, &'a LockedRef<Position3>);

        fn next(&mut self) -> Option<Self::Item> {
            if self.index < self.ve.len() {
                let current_index = self.index;
                self.index += 1;
                let item = &self.ve[current_index];
                Some((current_index, &item.0, &item.2))
            } else {
                None
            }
        }
    }


    #[main]
    pub fn custom_ecs() {
        let mut data1: Vec<(Locked<Position1>, Locked<Position2>, Locked<Position3>)> = Vec::new();
        let mut data2: Vec<(Locked<Position1>, Locked<Position3>)> = Vec::new();
        let mut o1: Vec<(Locked<Position1>, Locked<Position2>, Locked<Position3>)> = Vec::new();
        let mut o2: Vec<(Locked<Position1>, Locked<Position3>)> = Vec::new();

        /*let mut res1: Locked<MCR> =Locked::new(MCR( 32.0 ));
        let mut res2: Locked<ST> =Locked::new(ST {
            start: Instant::now(),
            runs: 0,
        });*/

            thread::scope(|s| {
                let handle = s.spawn(|| -> (Vec<(Locked<Position1>, Locked<Position2>, Locked<Position3>)>, Vec<(Locked<Position1>, Locked<Position3>)>) {return setup()});
                let data = handle.join().unwrap();
                o1.extend(data.0);
                o2.extend(data.1);
                data1.extend(o1);
                data2.extend(o2);
            });
    }
    #[task]
    fn setup() -> (Vec<(Locked<Position1>, Locked<Position2>, Locked<Position3>)>, Vec<(Locked<Position1>, Locked<Position3>)>) {
        let mut o1: Vec<(Locked<Position1>, Locked<Position2>, Locked<Position3>)> = Vec::new();
        let mut o2: Vec<(Locked<Position1>, Locked<Position3>)> = Vec::new();
        for _i in 0..10000 {
            o1.push((Locked::new(Position1 {x: 1.0,y: 1.0}),Locked::new(Position2 {x: 2.0,y: 2.0}),Locked::new(Position3 {x: 2.0,y: 2.0} )));
            o2.push((Locked::new(Position1 {x: 1.0,y: 1.0}),Locked::new(Position3 {x: 2.0,y: 2.0} )));
        }
        add_entity!(Position2 {x: 1.0,y: 1.0});
        return (o1,o2);
    }
}
*/
