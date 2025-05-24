use crate::comp::position_pixil::PositionPixil;
use corrosive_ecs_core::ecs_core::{LockedRef, Member};
use std::sync::LazyLock;
use glam::Mat4;
use corrosive_ecs_core_macro::{Component, Resource};
use corrosive_ecs_renderer_backend::public_functions::{get_window_ratio, write_to_buffer};
use crate::view_data::VIEW_DATA;

#[derive(Component)]
pub struct PixilCamera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
pub struct ActivePixilCameraData{
    position: Member<PositionPixil>,
    camera: LockedRef<PixilCamera>,
}
#[derive(Resource,Default)]
pub struct ActivePixilCamera {
    data: Option<ActivePixilCameraData>
}
impl ActivePixilCamera {
    pub(crate) fn update_view_matrix(&self) {
        if let Some(t) = &self.data{
            let projection = {
                let s = t.camera.f_read();
                Mat4::perspective_rh(s.unwrap().fov.clone(), get_window_ratio(), s.unwrap().near, s.unwrap().far)
            };
            let view = {
                t.position.f_read().unwrap().view()
            };
            println!("{:?}",&(projection * view).to_cols_array_2d());
            write_to_buffer(&VIEW_DATA.view_buffer, 0,  bytemuck::cast_slice(&(projection * view).to_cols_array()))
        }
    }
    pub fn new(position: &Member<PositionPixil>, camera: &LockedRef<PixilCamera>) -> Self {
        let data = ActivePixilCamera {
            data: Some(ActivePixilCameraData {
                position: position.clone(),
                camera: camera.clone(),
            }),
        };
        data.update_view_matrix();
        data
    }
}
