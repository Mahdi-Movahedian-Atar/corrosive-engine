use crate::corrosive_engine::auto_prelude::*;
use corrosive_ecs_core::ecs_core::EngineArch;
use std::collections::HashSet;
use std::sync::RwLock;
#[derive(Copy, Clone)]
pub struct rotate_model0<'a> {
    ve0: &'a Vec<(Member<PositionPixil>, PixilDynamicObject)>,
    rve0: &'a RwLock<HashSet<usize>>,
    len: usize,
}
impl<'a> rotate_model0<'a> {
    pub fn new(
        ve0: &'a Vec<(Member<PositionPixil>, PixilDynamicObject)>,
        rve0: &'a RwLock<HashSet<usize>>,
    ) -> Self {
        rotate_model0 {
            ve0,
            rve0,
            len: ve0.len(),
        }
    }
}
impl<'a> EngineArch<(&'a PixilDynamicObject, &'a Member<PositionPixil>)> for rotate_model0<'a> {
    fn remove(&self, mut index: usize) {
        if index < self.ve0.len() {
            self.rve0.write().unwrap().insert(index);
            return;
        };
        index -= self.ve0.len();
        eprintln!(
            "Warning: index of out of {} is out of bounds",
            "rotate_model"
        );
    }
    fn len(&self) -> usize {
        self.len
    }
    fn get_item(
        &self,
        mut index: usize,
    ) -> Option<(&'a PixilDynamicObject, &'a Member<PositionPixil>)> {
        if index < self.ve0.len() {
            return Some((&self.ve0[index].1, &self.ve0[index].0));
        };
        index -= self.ve0.len();
        None
    }
}
#[derive(Copy, Clone)]
pub struct update_pixil_position0<'a> {
    ve0: &'a Vec<(Member<PositionPixil>, PixilDynamicObject)>,
    rve0: &'a RwLock<HashSet<usize>>,
    len: usize,
}
impl<'a> update_pixil_position0<'a> {
    pub fn new(
        ve0: &'a Vec<(Member<PositionPixil>, PixilDynamicObject)>,
        rve0: &'a RwLock<HashSet<usize>>,
    ) -> Self {
        update_pixil_position0 {
            ve0,
            rve0,
            len: ve0.len(),
        }
    }
}
impl<'a> EngineArch<(&'a PixilDynamicObject, &'a Member<PositionPixil>)>
    for update_pixil_position0<'a>
{
    fn remove(&self, mut index: usize) {
        if index < self.ve0.len() {
            self.rve0.write().unwrap().insert(index);
            return;
        };
        index -= self.ve0.len();
        eprintln!(
            "Warning: index of out of {} is out of bounds",
            "update_pixil_position"
        );
    }
    fn len(&self) -> usize {
        self.len
    }
    fn get_item(
        &self,
        mut index: usize,
    ) -> Option<(&'a PixilDynamicObject, &'a Member<PositionPixil>)> {
        if index < self.ve0.len() {
            return Some((&self.ve0[index].1, &self.ve0[index].0));
        };
        index -= self.ve0.len();
        None
    }
}
