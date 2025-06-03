use crate::comp::position_pixil::PositionPixil;
use crate::view_data::VIEW_DATA;
use corrosive_ecs_core::ecs_core::{LockedRef, Member};
use corrosive_ecs_core_macro::{Component, Resource};
use corrosive_ecs_renderer_backend::public_functions::{
    create_buffer_init, get_window_ratio, write_to_buffer,
};
use corrosive_ecs_renderer_backend::wgpu::{Buffer, BufferUsages};
use glam::Mat4;
use std::sync::LazyLock;

#[derive(Component)]
pub struct PixilCamera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
impl PixilCamera {
    pub fn new(fov: f32, near: f32, far: f32) -> Self {
        Self { fov, near, far }
    }
}
pub struct ActivePixilCameraData {
    position: Member<PositionPixil>,
    camera: LockedRef<PixilCamera>,
}
#[derive(Resource, Default)]
pub struct ActivePixilCamera {
    data: Option<ActivePixilCameraData>,
}
impl ActivePixilCamera {
    pub(crate) fn update_view_matrix(&self) {
        if let Some(t) = &self.data {
            let (projection,near_far) = {
                let c = t.camera.f_read();
                (Mat4::perspective_rh(
                    c.unwrap().fov.clone(),
                    get_window_ratio(),
                    c.unwrap().near,
                    c.unwrap().far,
                ), [c.unwrap().near, c.unwrap().far])
            };
            let view = { t.position.f_read().unwrap().view() };
            write_to_buffer(
                &VIEW_DATA.view_buffer,
                0,
                bytemuck::cast_slice(&(projection * view).to_cols_array()),
            );
            write_to_buffer(
                &VIEW_DATA.position_buffer,
                0,
                bytemuck::bytes_of(&{
                    let (_,_,p) = t.position.f_read().unwrap().global.to_scale_rotation_translation();
                    p.to_array()
                }),
            );
            write_to_buffer(
                &VIEW_DATA.near_far_buffer,
                0,
                bytemuck::cast_slice(&near_far),
            );
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
    pub fn get_z_params(&self) -> (f32, f32) {
        self.data
            .as_ref()
            .map(|tuple| {
                let data = tuple.camera.f_read();
                return (data.unwrap().near, data.unwrap().far);
            })
            .unwrap_or((0.1, 1.0))
    }
}
