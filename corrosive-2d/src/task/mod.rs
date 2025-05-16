use crate::comp::camera2d::{ActiveCamera2D, Camera2D};
use crate::comp::{Depth, Mesh2D, Position2D, Renderer2dData, RendererMeta2D};
use crate::position2d_operations::Move2D;
use corrosive_ecs_core::ecs_core::{Arch, LockedRef, Member, Ref, Reference, Res};
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::comp::RenderGraph;
use corrosive_ecs_renderer_backend::public_functions::{
    create_buffer_init, get_resolution_bind_group, get_window_ratio, write_to_buffer,
};
use corrosive_ecs_renderer_backend::render_graph::{CommandEncoder, Device, Queue, RenderNode};
use corrosive_ecs_renderer_backend::wgpu::{
    BufferUsages, Color, LoadOp, Operations, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp,
};
use corrosive_ecs_renderer_backend::{public_functions, wgpu};
use crossbeam_channel::{unbounded, Receiver, Sender};
use glam::Mat4;
use std::cmp::PartialEq;
use std::sync::atomic::Ordering;
struct Renderer2DNode {
    rx: Receiver<()>,
    tx: Sender<RenderPass<'static>>,
}
impl RenderNode for Renderer2DNode {
    fn name(&self) -> &str {
        "Renderer2D"
    }

    fn execute(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("2D Render Pass"),
            color_attachments: &[Option::from(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.tx.send(render_pass.forget_lifetime()).unwrap();
        self.rx.recv().unwrap();
    }
}

#[task]
pub fn start_2d_renderer(graph: Res<RenderGraph>, renderer2d_data: Res<Renderer2dData>) {
    let (render_pass_tx, render_pass_rx) = unbounded::<RenderPass>();
    let (end_tx, end_rx) = unbounded::<()>();
    renderer2d_data.f_write().data = Some((render_pass_rx, end_tx));
    graph.f_write().add_node(Box::new(Renderer2DNode {
        tx: render_pass_tx,
        rx: end_rx,
    }));
}
#[task]
pub fn init_camera(active_camera: Res<ActiveCamera2D>) {
    let mut lock = active_camera.f_write();
    lock.buffer = Some(create_buffer_init(
        "2d_camera_buffer",
        &match &lock.data {
            Some((_, t)) => {
                if let Reference::Some(v) = &*t.f_read() {
                    bytemuck::cast_slice(&[v.view_matrix()]).to_vec()
                } else {
                    bytemuck::cast_slice(&[Mat4::IDENTITY.to_cols_array_2d()]).to_vec()
                }
            }
            _ => bytemuck::cast_slice(&[Mat4::IDENTITY.to_cols_array_2d()]).to_vec(),
        },
        BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    ))
}

#[task]
pub fn render_2d(meta: Arch<(&dyn Mesh2D, &RendererMeta2D)>, renderer2d_data: Res<Renderer2dData>) {
    if let Some(data) = &renderer2d_data.f_read().data {
        {
            let mut render_pass = &mut data.0.recv().unwrap();
            let resolution_bind_group = get_resolution_bind_group();
            let meta: Vec<_> = meta
                .iter()
                .filter(|x| {
                    if unsafe { *x.1.depth.get() } == Depth::Far {
                        if !x.1.is_active.load(Ordering::Relaxed) {
                            return false;
                        }
                        render_pass.set_pipeline(&x.1.pipeline_asset.get().layout);
                        render_pass.set_bind_group(0, &x.1.transform_data.1, &[]);
                        render_pass.set_bind_group(1, resolution_bind_group, &[]);
                        render_pass.set_bind_group(3, x.1.material.get_bind_group(), &[]);
                        x.0.draw(render_pass);
                        return false;
                    }
                    true
                })
                .collect();
            let meta: Vec<_> = meta
                .iter()
                .filter(|x| {
                    if unsafe { *x.1.depth.get() } == Depth::Back {
                        if !x.1.is_active.load(Ordering::Relaxed) {
                            return false;
                        }
                        render_pass.set_pipeline(&x.1.pipeline_asset.get().layout);
                        render_pass.set_bind_group(0, &x.1.transform_data.1, &[]);
                        render_pass.set_bind_group(1, resolution_bind_group, &[]);
                        render_pass.set_bind_group(3, x.1.material.get_bind_group(), &[]);
                        x.0.draw(render_pass);
                        return false;
                    }
                    true
                })
                .collect();
            let meta: Vec<_> = meta
                .iter()
                .filter(|x| {
                    if unsafe { *x.1.depth.get() } == Depth::Mid {
                        if !x.1.is_active.load(Ordering::Relaxed) {
                            return false;
                        }
                        render_pass.set_pipeline(&x.1.pipeline_asset.get().layout);
                        render_pass.set_bind_group(0, &x.1.transform_data.1, &[]);
                        render_pass.set_bind_group(1, resolution_bind_group, &[]);
                        render_pass.set_bind_group(3, x.1.material.get_bind_group(), &[]);
                        x.0.draw(render_pass);
                        return false;
                    }
                    true
                })
                .collect();
            let meta: Vec<_> = meta
                .iter()
                .filter(|x| {
                    if unsafe { *x.1.depth.get() } == Depth::Front {
                        if !x.1.is_active.load(Ordering::Relaxed) {
                            return false;
                        }
                        render_pass.set_pipeline(&x.1.pipeline_asset.get().layout);
                        render_pass.set_bind_group(0, &x.1.transform_data.1, &[]);
                        render_pass.set_bind_group(1, resolution_bind_group, &[]);
                        render_pass.set_bind_group(3, x.1.material.get_bind_group(), &[]);
                        x.0.draw(render_pass);
                        return false;
                    }
                    true
                })
                .collect();
            meta.iter().for_each(|x| {
                if !x.1.is_active.load(Ordering::Relaxed) {
                    return;
                }
                render_pass.set_pipeline(&x.1.pipeline_asset.get().layout);
                render_pass.set_bind_group(0, &x.1.transform_data.1, &[]);
                render_pass.set_bind_group(1, resolution_bind_group, &[]);
                render_pass.set_bind_group(3, x.1.material.get_bind_group(), &[]);
                x.0.draw(render_pass);
            })
        }
        data.1.send(()).unwrap();
    }
}

#[derive(Debug, Copy, Clone)]
struct Bounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_zoom: f32,
    max_zoom: f32,
}
#[task]
pub fn update_position(
    pos: Arch<(&Member<Position2D>, &RendererMeta2D)>,
    cam_pos: Arch<(&Member<Position2D>, &LockedRef<Camera2D>)>,
    active_camera: Res<ActiveCamera2D>,
) {
    for cam_poses in cam_pos.iter() {
        {
            let (x, y, scale_x) = if let Reference::Some(t) = &*cam_poses.0.f_read() {
                if !t.dirty {
                    continue;
                }
                (
                    /*t.global_position.x * t.global_scale.x) * t.global_rotation.cos()
                        + (t.global_position.y * t.global_scale.y) * t.global_rotation.sin(),
                    (t.global_position.y * t.global_scale.y) * t.global_rotation.cos()
                        + (t.global_position.x * t.global_scale.x) * t.global_rotation.sin(),*/
                    t.global_position.x,
                    t.global_position.y,
                    t.global_scale.x,
                )
            } else {
                continue;
            };

            let (pos_x, pos_y, scale) = if let Reference::Some(c) = &*cam_poses.1.f_read() {
                let mut new_x = x;
                let mut new_y = y;

                let mut new_scale = scale_x;
                new_scale = new_scale.min(
                    (c.right_boundary.unwrap_or(f32::MAX) - c.left_boundary.unwrap_or(f32::MIN))
                        .abs()
                        / 2.0,
                );
                new_scale = new_scale.min(
                    (c.top_boundary.unwrap_or(f32::MAX) - c.bottom_boundary.unwrap_or(f32::MIN))
                        .abs()
                        * get_window_ratio()
                        / 2.0,
                );
                new_scale = new_scale.max(c.min_zoom.unwrap_or(0.01));
                new_scale = new_scale.min(c.max_zoom.unwrap_or(f32::MAX));

                let left = c.left_boundary.unwrap_or(f32::MIN) + new_scale;
                let right = c.right_boundary.unwrap_or(f32::MAX) - new_scale;
                let bottom = c.bottom_boundary.unwrap_or(f32::MIN) * get_window_ratio() + new_scale;
                let top = c.top_boundary.unwrap_or(f32::MAX) * get_window_ratio() - new_scale;

                if new_x >= right {
                    new_x = right;
                };
                if new_x <= left {
                    new_x = left;
                };
                if new_y >= top {
                    new_y = top;
                };
                if new_y <= bottom {
                    new_y = bottom;
                };

                (new_x, new_y, new_scale)
            } else {
                continue;
            };
            let _ = Move2D::start(cam_poses.0)
                .set_transition_global(pos_x, pos_y)
                .set_scale_global(scale, scale)
                .finish();
            if let Reference::Some(t) = &mut *cam_poses.0.dry_f_write() {
                t.dirty = false;
            }
        };
    }
    for p in pos.iter().enumerate() {
        let mut lock = p.1 .0.dry_f_write();
        match &mut *lock {
            Reference::Some(t) => {
                write_to_buffer(
                    &p.1 .1.transform_data.0,
                    0,
                    bytemuck::cast_slice(&[t.uniform_matrix()]),
                );
                unsafe {
                    *p.1 .1.depth.get() = t.depth.clone();
                }
                t.dirty = false;
            }
            Reference::Expired => pos.remove(p.0),
        }
    }
    let lock = active_camera.f_write();

    if let Some(b) = &lock.buffer {
        if let Some(t) = &lock.data {
            if let Reference::Some(p) = &mut *t.1.dry_f_write() {
                write_to_buffer(b, 0, bytemuck::cast_slice(&[p.view_matrix()]));
                p.dirty = false;
            } else {
                write_to_buffer(
                    b,
                    0,
                    bytemuck::cast_slice(&[Mat4::IDENTITY.to_cols_array_2d()]),
                );
            }
        }
    } else {
        panic!("Run init_camera in setup.")
    }
}
