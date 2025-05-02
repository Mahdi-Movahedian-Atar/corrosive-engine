use crate::comp::Position2D;
use crate::math2d::Mat3;
use corrosive_ecs_core::ecs_core::{LockedRef, Member, Ref, Reference};
use corrosive_ecs_core_macro::{Component, Resource, State};
use corrosive_ecs_renderer_backend::public_functions::{create_buffer_init, write_to_buffer};
use corrosive_ecs_renderer_backend::wgpu::{Buffer, BufferUsages};

#[derive(Component, Default)]
pub struct Camera2D {
    pub left_boundary: Option<f32>,
    pub right_boundary: Option<f32>,
    pub top_boundary: Option<f32>,
    pub bottom_boundary: Option<f32>,
    pub min_zoom: Option<f32>,
    pub max_zoom: Option<f32>,
}

#[derive(Resource, Default)]
pub struct ActiveCamera2D {
    pub(crate) buffer: Option<(Buffer)>,
    pub(crate) data: Option<(LockedRef<Camera2D>, Member<Position2D>)>,
}

impl ActiveCamera2D {
    pub fn get_camera_position(&self) -> Option<Member<Position2D>> {
        self.data.as_ref().map(|tuple| tuple.1.clone())
    }
}

impl ActiveCamera2D {
    pub fn set_camera(&mut self, camera: &LockedRef<Camera2D>, position: &Member<Position2D>) {
        self.buffer = None;
        if let Some(t) = &self.buffer {
            match &*position.f_read() {
                Reference::Some(p) => {
                    write_to_buffer(t, 0, bytemuck::cast_slice(&[p.view_matrix()]))
                }
                Reference::Expired => {
                    return;
                }
            };
        } else {
            self.buffer = Some(match &*position.f_read() {
                Reference::Some(p) => create_buffer_init(
                    "2d_camera_buffer",
                    bytemuck::cast_slice(&[p.view_matrix()]),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                ),
                Reference::Expired => {
                    return;
                }
            });
        }
        self.data = Some((camera.clone(), position.clone()));
    }
}
