use corrosive_ecs_core::ecs_core::SharedBehavior;
use corrosive_ecs_core_macro::Component;
use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Clone, PartialEq, Component)]
pub struct PositionPixil {
    pub(crate) position: Vec3,
    pub(crate) rotation: Quat,
    pub(crate) scale: Vec3,
    pub(crate) global: Mat4,
    pub(crate) dirty: bool,
}
impl Default for PositionPixil {
    fn default() -> Self {
        PositionPixil::NEW
    }
}

impl PositionPixil {
    pub const NEW: Self = PositionPixil {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        global: Mat4::IDENTITY,
        dirty: true,
    };
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        PositionPixil {
            position,
            rotation,
            scale,
            global: Mat4::from_scale_rotation_translation(scale, rotation, position),
            dirty: true,
        }
    }
    pub(crate) fn uniform(&self) -> [f32; 16] {
        self.global.to_cols_array()
    }
    pub(crate) fn view(&self) -> Mat4 {
        self.global.inverse()
    }
}
impl SharedBehavior for PositionPixil {
    fn shaded_add_behavior(&mut self, parent: &Self) {
        self.global = parent.global
            * Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
        self.dirty = true;
    }

    fn shaded_remove_behavior(&mut self) {
        self.global =
            Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
        self.dirty = true;
    }
}
