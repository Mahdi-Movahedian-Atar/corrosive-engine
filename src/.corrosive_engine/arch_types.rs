use crate::corrosive_engine::auto_prelude::*;
use corrosive_ecs_core::ecs_core::EngineArch;
use std::collections::HashSet;
use std::sync::RwLock;
#[derive(Copy, Clone)]
pub struct render_2d0<'a> {
    ve0: &'a Vec<(Member<Position2D>, RendererMeta2D, Sprite2D)>,
    rve0: &'a RwLock<HashSet<usize>>,
    len: usize,
}
impl<'a> render_2d0<'a> {
    pub fn new(
        ve0: &'a Vec<(Member<Position2D>, RendererMeta2D, Sprite2D)>,
        rve0: &'a RwLock<HashSet<usize>>,
    ) -> Self {
        render_2d0 {
            ve0,
            rve0,
            len: ve0.len(),
        }
    }
}
impl<'a> EngineArch<(&'a dyn Mesh2D, &'a RendererMeta2D)> for render_2d0<'a> {
    fn remove(&self, mut index: usize) {
        if index < self.ve0.len() {
            self.rve0.write().unwrap().insert(index);
            return;
        };
        index -= self.ve0.len();
        eprintln!("Warning: index of out of {} is out of bounds", "render_2d");
    }
    fn len(&self) -> usize {
        self.len
    }
    fn get_item(&self, mut index: usize) -> Option<(&'a dyn Mesh2D, &'a RendererMeta2D)> {
        if index < self.ve0.len() {
            return Some((&self.ve0[index].2, &self.ve0[index].1));
        };
        index -= self.ve0.len();
        None
    }
}
