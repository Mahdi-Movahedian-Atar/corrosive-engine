use crate::comp::Position2D;
use crate::math2d::Mat3;
use corrosive_ecs_core::ecs_core::{Member, Ref};
use corrosive_ecs_core_macro::{Component, Resource, State};
use corrosive_ecs_renderer_backend::helper::Buffer;

#[derive(Component)]
pub struct Camera2D {}

#[derive(Resource, Default)]
pub struct ActiveCamera2D {
    pub(crate) buffer: Option<(Buffer)>,
    pub(crate) data: Option<(Ref<Camera2D>, Member<Position2D>)>,
}
