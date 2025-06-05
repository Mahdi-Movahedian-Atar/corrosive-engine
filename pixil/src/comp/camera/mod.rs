use std::cell::LazyCell;
use crate::comp::position_pixil::PositionPixil;
use corrosive_ecs_core::ecs_core::{LockedRef, Member};
use corrosive_ecs_core_macro::{Component, Resource};
use corrosive_ecs_renderer_backend::public_functions::{
    create_buffer_init, get_window_ratio, write_to_buffer,
};
use corrosive_ecs_renderer_backend::wgpu::{Buffer, BufferUsages};
use glam::Mat4;
use std::sync::LazyLock;
use bytemuck::cast_slice;
use crate::comp::render::PixilRenderSettings;

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
#[derive(Resource)]
pub struct ActivePixilCamera {
    data: Option<ActivePixilCameraData>,
    pub(crate)view_buffer: LazyCell<Buffer>,
    pub(crate)position_buffer: LazyCell<Buffer>,
    pub(crate) z_params_buffer: LazyCell<Buffer>
}
impl Default for ActivePixilCamera{
    fn default() -> Self {
        Self{
            data: None,
            view_buffer: LazyCell::new(||{
                create_buffer_init(
                    "PixilViewBuffer",
                    cast_slice(&Mat4::IDENTITY.to_cols_array()),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                )
            }),
            position_buffer: LazyCell::new(||{
                create_buffer_init(
                    "PixilViewBuffer",
                    cast_slice(&[0.0,0.0,0.0]),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                )
            }),
            z_params_buffer: LazyCell::new(||{
                create_buffer_init(
                    "PixilViewBuffer",
                    cast_slice(&[0.0,1.0]),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                )
            }),
        }

    }
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
                &self.view_buffer,
                0,
                bytemuck::cast_slice(&(projection * view).to_cols_array()),
            );
            write_to_buffer(
                &self.position_buffer,
                0,
                bytemuck::bytes_of(&{
                    let (_,_,p) = t.position.f_read().unwrap().global.to_scale_rotation_translation();
                    p.to_array()
                }),
            );
            write_to_buffer(
                &self.z_params_buffer,
                0,
                bytemuck::cast_slice(&near_far),
            );
        }
    }
    pub fn new(&mut self, position: &Member<PositionPixil>, camera: &LockedRef<PixilCamera>) {
        self.data = Some(ActivePixilCameraData {
            position: position.clone(),
            camera: camera.clone(),
        });
        self.update_view_matrix();
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

unsafe impl Send for ActivePixilCamera {}
unsafe impl Sync for ActivePixilCamera {}