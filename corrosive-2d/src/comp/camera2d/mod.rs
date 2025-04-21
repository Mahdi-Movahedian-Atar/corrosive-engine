use crate::comp::Position2D;
use crate::math2d::Mat3;
use corrosive_ecs_core::ecs_core::{Member, Ref, Reference};
use corrosive_ecs_core_macro::{Component, Resource, State};
use corrosive_ecs_renderer_backend::public_functions::{create_buffer_init, write_to_buffer};
use corrosive_ecs_renderer_backend::wgpu::{Buffer, BufferUsages};

#[derive(Component)]
pub struct Camera2D {}

#[derive(Resource, Default)]
pub struct ActiveCamera2D {
    pub(crate) buffer: Option<(Buffer)>,
    pub(crate) data: Option<(Ref<Camera2D>, Member<Position2D>)>,
}

impl ActiveCamera2D {
    pub fn set_camera(&mut self, camera: &Ref<Camera2D>, position: &Member<Position2D>) {
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
