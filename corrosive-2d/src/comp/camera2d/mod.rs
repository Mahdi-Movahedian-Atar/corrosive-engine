use crate::comp::Position2D;
use corrosive_ecs_core::ecs_core::{LockedRef, Member, Ref, Reference};
use corrosive_ecs_core_macro::{Component, Resource, State};
use corrosive_ecs_renderer_backend::public_functions::{
    create_buffer_init, get_window_resolution, write_to_buffer,
};
use corrosive_ecs_renderer_backend::wgpu::{Buffer, BufferUsages};
use glam::Vec2;

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

    pub fn mouse_to_world(&self, mouse: (f64, f64)) -> (f32, f32) {
        self.data
            .as_ref()
            .and_then(|(c, p)| {
                let mut world_pos = Vec2::new(mouse.1 as f32, mouse.1 as f32)
                    - Vec2::new(
                        get_window_resolution().0 as f32,
                        get_window_resolution().1 as f32,
                    ) / 2.0;
                let binding = p.f_read();
                let camera_lock = match &*binding {
                    Reference::Some(t) => t,
                    Reference::Expired => {
                        return None;
                    }
                };

                world_pos /= Vec2::new(camera_lock.global_scale.x, camera_lock.global_scale.x);

                let angle = -camera_lock.global_rotation;
                let cos_theta = angle.cos();
                let sin_theta = angle.sin();

                world_pos = Vec2::new(
                    world_pos.x * cos_theta - world_pos.y * sin_theta,
                    world_pos.x * sin_theta + world_pos.y * cos_theta,
                );

                world_pos += camera_lock.global_position;

                Some((world_pos.x, world_pos.y))
            })
            .unwrap_or((0.0, 0.0))
    }

    pub fn delta_mouse_to_world(&self, mouse: (f64, f64)) -> (f32, f32) {
        self.data
            .as_ref()
            .and_then(|(c, p)| {
                let mut world_pos = Vec2::new(
                    -mouse.0 as f32 / get_window_resolution().0 as f32,
                    mouse.1 as f32 / get_window_resolution().1 as f32,
                );
                let binding = p.f_read();
                let camera_lock = match &*binding {
                    Reference::Some(t) => t,
                    Reference::Expired => {
                        return None;
                    }
                };

                world_pos *= Vec2::new(camera_lock.global_scale.x, camera_lock.global_scale.x);

                /*let angle = -camera_lock.global_rotation;
                            let cos_theta = angle.cos();
                            let sin_theta = angle.sin();

                            world_pos = Vec2::new(
                                world_pos.x * cos_theta - world_pos.y * sin_theta,
                                world_pos.x * sin_theta + world_pos.y * cos_theta,
                            );
                */
                Some((world_pos.x, world_pos.y))
            })
            .unwrap_or((0.0, 0.0))
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
