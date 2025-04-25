use corrosive_ecs_core_macro::{Component, Resource, State};

#[derive(Component)]
pub struct Position3 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}
#[derive(Component)]
pub struct Position4 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Default, Eq, PartialEq, Clone, Debug, Resource)]
pub struct MarkedResources(pub usize);

#[derive(Default, Eq, PartialEq, Clone, Debug, State)]
pub enum StateExample {
    #[default]
    A,
    B,
    C,
}
