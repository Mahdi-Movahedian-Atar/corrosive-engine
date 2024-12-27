use corrosive_ecs_core_macro::Component;
pub mod sub;

#[derive(Debug, Component)]
pub struct Position1 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}
#[derive(Component)]
pub struct Position2 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}
