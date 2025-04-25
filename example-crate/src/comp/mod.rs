use corrosive_ecs_core::ecs_core::{Locked, Ref};
use corrosive_ecs_core::trait_for;
use corrosive_ecs_core_macro::{task, trait_bound, Component};
pub mod sub;

#[derive(Debug, Component)]
pub struct Position1 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}
impl test for Locked<Position1> {
    fn get_num(&self) -> f32 {
        2.0
    }
}
#[derive(Component)]
pub struct Position2 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}
trait_for!(trait test => Ref<Position2>, Locked<Position1>);

impl test for Ref<Position2> {
    fn get_num(&self) -> f32 {
        2.0
    }
}

#[trait_bound]
pub trait test {
    fn get_num(&self) -> f32;
}
