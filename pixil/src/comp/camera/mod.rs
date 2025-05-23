use crate::comp::position_pixil::PositionPixil;
use corrosive_ecs_core::ecs_core::{LockedRef, Member};
use std::sync::LazyLock;

pub struct PixilCamera {
    fov: f32,
    near: f32,
    far: f32,
}
pub struct ActivePixilCamera {
    position: Member<PositionPixil>,
    camera: LockedRef<PixilCamera>,
}
impl ActivePixilCamera {
    pub(crate) fn update_view_matrix(&self) {}
}
