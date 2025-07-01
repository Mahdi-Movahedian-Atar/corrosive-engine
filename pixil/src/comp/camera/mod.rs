use crate::comp::position_pixil::PositionPixil;
use crate::comp::render::PixilRenderSettings;
use bytemuck::cast_slice;
use corrosive_ecs_core::ecs_core::{LockedRef, Member};
use corrosive_ecs_core_macro::{Component, Resource};
use corrosive_ecs_renderer_backend::public_functions::{
    create_buffer_init, get_window_ratio, write_to_buffer,
};
use corrosive_ecs_renderer_backend::wgpu::{Buffer, BufferUsages};
use glam::Mat4;
use std::cell::LazyCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;

#[derive(Component,Debug)]
pub struct PixilCamera {
    pub(crate) fov: f32,
    pub(crate) near: f32,
    pub(crate) far: f32,
    pub(crate) has_updated:bool
}
impl PixilCamera {
    pub fn new(fov: f32, near: f32, far: f32) -> Self {
        Self { fov, near, far, has_updated:true }
    }
    pub fn set_fov(&mut self,fov:f32){ self.has_updated = true;self.fov = fov}
    pub fn set_near(&mut self,near:f32){ self.has_updated = true;self.near = near}
    pub fn set_far(&mut self,far:f32){ self.has_updated = true;self.far = far}
    pub fn get_fov(& self) -> f32{ self.fov }
    pub fn get_near(& self)-> f32{ self.near }
    pub fn get_far(& self)-> f32 { self.far }
}
pub struct ActivePixilCameraData {
    pub(crate) position: Member<PositionPixil>,
    pub(crate) camera: LockedRef<PixilCamera>,
}
#[derive(Resource)]
pub struct ActivePixilCamera {
    pub(crate) data: Option<ActivePixilCameraData>,
    pub(crate) view_buffer: LazyCell<Buffer>,
    pub(crate) position_buffer: LazyCell<Buffer>,
    pub(crate) z_params_buffer: LazyCell<Buffer>,
    pub(crate) projection_buffer: LazyCell<Buffer>,
    pub(crate) inverse_projection_buffer: LazyCell<Buffer>,
    pub(crate) has_updated: AtomicBool
}
impl Default for ActivePixilCamera {
    fn default() -> Self {
        Self {
            data: None,
            view_buffer: LazyCell::new(|| {
                create_buffer_init(
                    "PixilViewBuffer",
                    cast_slice(&Mat4::IDENTITY.to_cols_array()),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                )
            }),
            position_buffer: LazyCell::new(|| {
                create_buffer_init(
                    "PixilPositionBuffer",
                    cast_slice(&[0.0, 0.0, 0.0]),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                )
            }),
            z_params_buffer: LazyCell::new(|| {
                create_buffer_init(
                    "PixilZParamsBuffer",
                    cast_slice(&[0.0, 1.0]),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                )
            }),
            projection_buffer: LazyCell::new(|| {
                create_buffer_init(
                    "PixilProjectionBuffer",
                    cast_slice(&Mat4::IDENTITY.to_cols_array()),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                )
            }),
            inverse_projection_buffer: LazyCell::new(|| {
                create_buffer_init(
                    "PixilInverseProjectionBuffer",
                    cast_slice(&Mat4::IDENTITY.to_cols_array()),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                )
            }),
            has_updated: AtomicBool::new(true)
        }
    }
}
impl ActivePixilCamera {
    pub(crate) fn update_view_matrix(&self) {
        if let Some(t) = &self.data {
            let (projection, near_far) = {
                let c = t.camera.f_read();
                (
                    Mat4::perspective_rh(
                        c.unwrap().fov.clone(),
                        get_window_ratio(),
                        c.unwrap().near,
                        c.unwrap().far,
                    ),
                    [c.unwrap().near, c.unwrap().far],
                )
            };
            let view = { t.position.f_read().unwrap().view() };
            write_to_buffer(
                &self.view_buffer,
                0,
                bytemuck::cast_slice(&view.to_cols_array()),
            );
            write_to_buffer(
                &self.position_buffer,
                0,
                bytemuck::bytes_of(&{
                    let (_, _, p) = t
                        .position
                        .f_read()
                        .unwrap()
                        .global
                        .to_scale_rotation_translation();
                    p.to_array()
                }),
            );
        }
    }
    pub(crate) fn update_camera_data(&self){
        if let Some(t) = &self.data {
            let (projection, near_far) = {
                let c = t.camera.f_read();
                (
                    Mat4::perspective_rh(
                        c.unwrap().fov.clone(),
                        get_window_ratio(),
                        c.unwrap().near,
                        c.unwrap().far,
                    ),
                    [c.unwrap().near, c.unwrap().far],
                )
            };
            write_to_buffer(
                &self.projection_buffer,
                0,
                bytemuck::bytes_of(&projection.to_cols_array()),
            );
            write_to_buffer(
                &self.inverse_projection_buffer,
                0,
                bytemuck::bytes_of(&projection.inverse().to_cols_array()),
            );
            write_to_buffer(&self.z_params_buffer, 0, bytemuck::cast_slice(&near_far));
            self.has_updated.store(false,Ordering::Relaxed);
        }
    }
    pub fn new(&mut self, position: &Member<PositionPixil>, camera: &LockedRef<PixilCamera>) {
        self.data = Some(ActivePixilCameraData {
            position: position.clone(),
            camera: camera.clone(),
        });
        self.has_updated.store(true,Ordering::Relaxed);
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
