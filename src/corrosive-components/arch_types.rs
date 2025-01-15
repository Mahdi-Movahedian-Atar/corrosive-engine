use crate::corrosive_engine::auto_prelude::*;
use corrosive_ecs_core::ecs_core::EngineArch;
use std::collections::HashSet;
use std::sync::RwLock;
#[derive(Copy, Clone)]
pub struct macro_test0<'a> {
    ve0: &'a Vec<(LockedRef<Position3>, Ref<Position2>)>,
    rve0: &'a RwLock<HashSet<usize>>,
    ve1: &'a Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
    rve1: &'a RwLock<HashSet<usize>>,
    ve2: &'a Vec<(LockedRef<Position3>,)>,
    rve2: &'a RwLock<HashSet<usize>>,
    len: usize,
}
impl<'a> macro_test0<'a> {
    pub fn new(
        ve0: &'a Vec<(LockedRef<Position3>, Ref<Position2>)>,
        rve0: &'a RwLock<HashSet<usize>>,
        ve1: &'a Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
        rve1: &'a RwLock<HashSet<usize>>,
        ve2: &'a Vec<(LockedRef<Position3>,)>,
        rve2: &'a RwLock<HashSet<usize>>,
    ) -> Self {
        macro_test0 {
            ve0,
            rve0,
            ve1,
            rve1,
            ve2,
            rve2,
            len: ve0.len() + ve1.len() + ve2.len(),
        }
    }
}
impl<'a> EngineArch<(&'a LockedRef<Position3>,)> for macro_test0<'a> {
    fn remove(&self, mut index: usize) {
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
        eprintln!("Warning: index of out of #exact_name is out of bounds");
    }
    fn len(&self) -> usize {
        self.len
    }
    fn get_item(&self, mut index: usize) -> Option<(&'a LockedRef<Position3>,)> {
        if index < self.ve0.len() {
            return Some((&self.ve0[index].0,));
        };
        index -= self.ve0.len();
        if index < self.ve1.len() {
            return Some((&self.ve1[index].0,));
        };
        index -= self.ve1.len();
        if index < self.ve2.len() {
            return Some((&self.ve2[index].0,));
        };
        index -= self.ve2.len();
        None
    }
}
#[derive(Copy, Clone)]
pub struct macro_test1<'a> {
    ve0: &'a Vec<(LockedRef<Position3>, Ref<Position2>)>,
    rve0: &'a RwLock<HashSet<usize>>,
    ve1: &'a Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
    rve1: &'a RwLock<HashSet<usize>>,
    len: usize,
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
        }
    }
}
impl<'a> EngineArch<(&'a LockedRef<Position3>, &'a Ref<Position2>)> for macro_test1<'a> {
    fn remove(&self, mut index: usize) {
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
        eprintln!("Warning: index of out of #exact_name is out of bounds");
    }
    fn len(&self) -> usize {
        self.len
    }
    fn get_item(&self, mut index: usize) -> Option<(&'a LockedRef<Position3>, &'a Ref<Position2>)> {
        if index < self.ve0.len() {
            return Some((&self.ve0[index].0, &self.ve0[index].1));
        };
        index -= self.ve0.len();
        if index < self.ve1.len() {
            return Some((&self.ve1[index].0, &self.ve1[index].2));
        };
        index -= self.ve1.len();
        None
    }
}
#[derive(Copy, Clone)]
pub struct macro_test2<'a> {
    ve0: &'a Vec<(LockedRef<Position3>, Ref<Position2>)>,
    rve0: &'a RwLock<HashSet<usize>>,
    ve1: &'a Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
    rve1: &'a RwLock<HashSet<usize>>,
    len: usize,
}
impl<'a> macro_test2<'a> {
    pub fn new(
        ve0: &'a Vec<(LockedRef<Position3>, Ref<Position2>)>,
        rve0: &'a RwLock<HashSet<usize>>,
        ve1: &'a Vec<(LockedRef<Position3>, Position3, Ref<Position2>)>,
        rve1: &'a RwLock<HashSet<usize>>,
    ) -> Self {
        macro_test2 {
            ve0,
            rve0,
            ve1,
            rve1,
            len: ve0.len() + ve1.len(),
        }
    }
}
impl<'a> EngineArch<(&'a Ref<Position2>, &'a LockedRef<Position3>)> for macro_test2<'a> {
    fn remove(&self, mut index: usize) {
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
        eprintln!("Warning: index of out of #exact_name is out of bounds");
    }
    fn len(&self) -> usize {
        self.len
    }
    fn get_item(&self, mut index: usize) -> Option<(&'a Ref<Position2>, &'a LockedRef<Position3>)> {
        if index < self.ve0.len() {
            return Some((&self.ve0[index].1, &self.ve0[index].0));
        };
        index -= self.ve0.len();
        if index < self.ve1.len() {
            return Some((&self.ve1[index].2, &self.ve1[index].0));
        };
        index -= self.ve1.len();
        None
    }
}
