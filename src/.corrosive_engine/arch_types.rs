pub mod arch_types {
    use crate::corrosive_engine::auto_prelude::prelude::*;
    use corrosive_ecs_core::ecs_core::Locked;
    use std::collections::HashSet;
    use std::sync::RwLock;

    #[derive(Copy, Clone)]
    pub struct TestUtArch<'a> {
        ve1: &'a Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
        ve2: &'a Vec<(Locked<Position1>, LockedRef<Position3>)>,
        ve3: &'a Vec<(Locked<Position1>)>,
        rve1: &'a RwLock<HashSet<usize>>,
        rve2: &'a RwLock<HashSet<usize>>,
        rve3: &'a RwLock<HashSet<usize>>,
        pub len: usize,
        pub v_i: usize,
    }
    impl<'a> TestUtArch<'a> {
        pub fn new(
            ve1: &'a Vec<(Locked<Position1>, Ref<Position2>, LockedRef<Position3>)>,
            ve2: &'a Vec<(Locked<Position1>, LockedRef<Position3>)>,
            ve3: &'a Vec<(Locked<Position1>)>,
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
                v_i: 0,
            }
        }

        pub fn remove(&self, mut index: usize) {
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
}
