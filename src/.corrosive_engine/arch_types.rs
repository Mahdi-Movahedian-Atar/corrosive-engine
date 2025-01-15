pub mod arch_types {
    use crate::corrosive_engine::auto_prelude::*;
    use corrosive_ecs_core::ecs_core::{Arch, EngineArch};
    use std::cell::UnsafeCell;
    use std::collections::HashSet;
    use std::marker::PhantomData;
    use std::sync::RwLock;

    pub struct TestUtArch<'a> {
        ve1: &'a Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
        ve2: &'a Vec<(Locked<Position1>, LockedRef<Position3>)>,
        ve3: &'a Vec<(Locked<Position1>,)>,
        rve1: &'a RwLock<HashSet<usize>>,
        rve2: &'a RwLock<HashSet<usize>>,
        rve3: &'a RwLock<HashSet<usize>>,
        len: usize,
    }
    impl<'a> EngineArch<(&'a Locked<Position1>,)> for TestUtArch<'a> {
        fn remove(&self, mut index: usize) {
            if index < self.ve1.len() {
                self.rve1.write().unwrap().insert(index);
                return;
            };
            index -= self.ve1.len();
            if index < self.ve2.len() {
                self.rve2.write().unwrap().insert(index);
                return;
            };
            index -= self.ve2.len();
            if index < self.ve3.len() {
                self.rve3.write().unwrap().insert(index);
                return;
            };
            eprintln!(
                "Warning: index of out of {} is out of bounds",
                "this is test"
            );
        }

        fn len(&self) -> usize {
            self.len
        }

        fn get_item(&self, mut index: usize) -> Option<(&'a Locked<Position1>,)> {
            if index < self.ve1.len() {
                return Some((&self.ve1[index].0,));
            };
            index -= self.ve1.len();
            if index < self.ve2.len() {
                return Some((&self.ve2[index].0,));
            };
            index -= self.ve2.len();
            if index < self.ve3.len() {
                return Some((&self.ve3[index].0,));
            };
            None
        }
    }
    impl<'a> TestUtArch<'a> {
        pub fn new(
            ve1: &'a Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
            ve2: &'a Vec<(Locked<Position1>, LockedRef<Position3>)>,
            ve3: &'a Vec<(Locked<Position1>,)>,
            rve1: &'a RwLock<HashSet<usize>>,
            rve2: &'a RwLock<HashSet<usize>>,
            rve3: &'a RwLock<HashSet<usize>>,
        ) -> Self {
            TestUtArch {
                ve1,
                ve2,
                ve3,
                rve1,
                rve2,
                rve3,
                len: ve1.len() + ve2.len() + ve3.len(),
            }
        }
    }
    #[derive(Copy, Clone)]
    pub struct TestUtArch2<'a> {
        pub ve1: &'a Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
        pub ve2: &'a Vec<(Ref<Position2>, LockedRef<Position3>)>,
        pub len: usize,
        pub v_i: usize,
    }
    impl<'a> TestUtArch2<'a> {
        pub fn new(
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
    #[derive(Copy, Clone)]
    pub struct macro_test0<'a> {
        ve0: &'a Vec<(LockedRef<Position3>, Ref<Position2>)>,
        rve0: &'a RwLock<HashSet<usize>>,
        ve1: &'a Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
        rve1: &'a RwLock<HashSet<usize>>,
        pub len: usize,
        pub v_i: usize,
    }
    impl<'a> macro_test0<'a> {
        pub fn new(
            ve0: &'a Vec<(LockedRef<Position3>, Ref<Position2>)>,
            rve0: &'a RwLock<HashSet<usize>>,
            ve1: &'a Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
            rve1: &'a RwLock<HashSet<usize>>,
        ) -> Self {
            macro_test0 {
                ve0,
                rve0,
                ve1,
                rve1,
                len: ve0.len() + ve1.len(),
                v_i: 0,
            }
        }
        pub fn remove(&self, mut index: usize) {
            if index < self.ve0.len() {
                self.rve0.write().unwrap().insert(index);
                return;
            };
            index -= self.ve0.len();
            if index < self.ve1.len() {
                self.rve1.write().unwrap().insert(index);
                return;
            };
            index -= self.ve1.len();
        }
    }
    impl<'a> Iterator for macro_test0<'a> {
        type Item = (&'a Ref<Position2>, &'a LockedRef<Position3>);
        fn next(&mut self) -> Option<Self::Item> {
            let mut current_index = self.v_i.clone();
            self.v_i += 1;
            if current_index < self.ve0.len() {
                return Some((&self.ve0[current_index].1, &self.ve0[current_index].0));
            };
            current_index -= self.ve0.len();
            if current_index < self.ve1.len() {
                return Some((&self.ve1[current_index].2, &self.ve1[current_index].0));
            };
            current_index -= self.ve1.len();
            None
        }
    }
    #[derive(Copy, Clone)]
    pub struct macro_test1<'a> {
        ve0: &'a Vec<(LockedRef<Position3>, Ref<Position2>)>,
        rve0: &'a RwLock<HashSet<usize>>,
        ve1: &'a Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
        rve1: &'a RwLock<HashSet<usize>>,
        pub len: usize,
        pub v_i: usize,
    }
    impl<'a> macro_test1<'a> {
        pub fn new(
            ve0: &'a Vec<(LockedRef<Position3>, Ref<Position2>)>,
            rve0: &'a RwLock<HashSet<usize>>,
            ve1: &'a Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
            rve1: &'a RwLock<HashSet<usize>>,
        ) -> Self {
            macro_test1 {
                ve0,
                rve0,
                ve1,
                rve1,
                len: ve0.len() + ve1.len(),
                v_i: 0,
            }
        }
        pub fn remove(&self, mut index: usize) {
            if index < self.ve0.len() {
                self.rve0.write().unwrap().insert(index);
                return;
            };
            index -= self.ve0.len();
            if index < self.ve1.len() {
                self.rve1.write().unwrap().insert(index);
                return;
            };
            index -= self.ve1.len();
        }
    }
    impl<'a> Iterator for macro_test1<'a> {
        type Item = (&'a Ref<Position2>, &'a LockedRef<Position3>);
        fn next(&mut self) -> Option<Self::Item> {
            let mut current_index = self.v_i.clone();
            self.v_i += 1;
            if current_index < self.ve0.len() {
                return Some((&self.ve0[current_index].1, &self.ve0[current_index].0));
            };
            current_index -= self.ve0.len();
            if current_index < self.ve1.len() {
                return Some((&self.ve1[current_index].2, &self.ve1[current_index].0));
            };
            current_index -= self.ve1.len();
            None
        }
    }
    #[derive(Copy, Clone)]
    pub struct macro_test2<'a> {
        ve0: &'a Vec<(LockedRef<Position3>, Ref<Position2>)>,
        rve0: &'a RwLock<HashSet<usize>>,
        ve1: &'a Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
        rve1: &'a RwLock<HashSet<usize>>,
        ve2: &'a Vec<(LockedRef<Position3>,)>,
        rve2: &'a RwLock<HashSet<usize>>,
        pub len: usize,
        pub v_i: usize,
    }
    impl<'a> macro_test2<'a> {
        pub fn new(
            ve0: &'a Vec<(LockedRef<Position3>, Ref<Position2>)>,
            rve0: &'a RwLock<HashSet<usize>>,
            ve1: &'a Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
            rve1: &'a RwLock<HashSet<usize>>,
            ve2: &'a Vec<(LockedRef<Position3>,)>,
            rve2: &'a RwLock<HashSet<usize>>,
        ) -> Self {
            macro_test2 {
                ve0,
                rve0,
                ve1,
                rve1,
                ve2,
                rve2,
                len: ve0.len() + ve1.len() + ve2.len(),
                v_i: 0,
            }
        }
        pub fn remove(&self, mut index: usize) {
            if index < self.ve0.len() {
                self.rve0.write().unwrap().insert(index);
                return;
            };
            index -= self.ve0.len();
            if index < self.ve1.len() {
                self.rve1.write().unwrap().insert(index);
                return;
            };
            index -= self.ve1.len();
            if index < self.ve2.len() {
                self.rve2.write().unwrap().insert(index);
                return;
            };
            index -= self.ve2.len();
        }
    }
    impl<'a> Iterator for macro_test2<'a> {
        type Item = (&'a LockedRef<Position3>,);
        fn next(&mut self) -> Option<Self::Item> {
            let mut current_index = self.v_i.clone();
            self.v_i += 1;
            if current_index < self.ve0.len() {
                return Some((&self.ve0[current_index].0,));
            };
            current_index -= self.ve0.len();
            if current_index < self.ve1.len() {
                return Some((&self.ve1[current_index].0,));
            };
            current_index -= self.ve1.len();
            if current_index < self.ve2.len() {
                return Some((&self.ve2[current_index].0,));
            };
            current_index -= self.ve2.len();
            None
        }
    }
}
