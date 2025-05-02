use crate::comp::camera2d::{ActiveCamera2D, Camera2D};
use crate::comp::{Mesh2D, Position2D, Renderer2dData, RendererMeta2D};
use crate::math2d::Mat3;
use corrosive_ecs_core::ecs_core::{Arch, LockedRef, Member, Ref, Reference, Res};
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::comp::RenderGraph;
use corrosive_ecs_renderer_backend::public_functions::{create_buffer_init, get_resolution_bind_group, get_window_ratio, write_to_buffer};
use corrosive_ecs_renderer_backend::render_graph::{CommandEncoder, Device, Queue, RenderNode};
use corrosive_ecs_renderer_backend::wgpu::{
    BufferUsages, Color, LoadOp, Operations, RenderPass, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp,
};
use corrosive_ecs_renderer_backend::{public_functions, wgpu};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::cmp::min;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;
use crate::position2d_operations::Move2D;

pub(crate) struct UnsafeRenderPass {
    ptr: NonNull<RenderPass<'static>>,
    _marker: PhantomData<&'static mut RenderPass<'static>>,
}
unsafe impl Send for UnsafeRenderPass {}
unsafe impl Sync for UnsafeRenderPass {}
struct Renderer2DNode {
    rx: Receiver<()>,
    tx: Sender<UnsafeRenderPass>,
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
        let ptr = NonNull::from(unsafe {
            std::mem::transmute::<&mut RenderPass<'_>, &mut RenderPass<'static>>(&mut render_pass)
        });

        let unsafe_pass = UnsafeRenderPass {
            ptr,
            _marker: PhantomData,
        };

        self.tx.send(unsafe_pass).unwrap();
        self.rx.recv().unwrap();
        drop(render_pass)
    }
}

#[task]
pub fn start_2d_renderer(graph: Res<RenderGraph>, renderer2d_data: Res<Renderer2dData>) {
    let (render_pass_tx, render_pass_rx) = unbounded::<UnsafeRenderPass>();
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
                    bytemuck::cast_slice(&[Mat3::identity().to_mat4_4()]).to_vec()
                }
            }
            _ => bytemuck::cast_slice(&[Mat3::identity().to_mat4_4()]).to_vec(),
        },
        BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    ))
}
#[task]
pub fn render_2d(meta: Arch<(&dyn Mesh2D, &RendererMeta2D)>, renderer2d_data: Res<Renderer2dData>) {
    if let Some(data) = &renderer2d_data.f_read().data {
        {
            let render_pass = unsafe { data.0.recv().unwrap().ptr.as_ptr().as_mut().unwrap() };
            let resolution_bind_group = get_resolution_bind_group();
            meta.iter().for_each(|x| {
                render_pass.set_pipeline(&x.1.pipeline_asset.get().layout);
                render_pass.set_bind_group(0, &x.1.transform_data.1, &[]);
                render_pass.set_bind_group(1, resolution_bind_group, &[]);
                render_pass.set_bind_group(3, x.1.material.get_bind_group(), &[]);
                x.0.draw(render_pass);
            });
        }
        data.1.send(()).unwrap();
    }
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
                    return;
                }
                    (/*t.global_position.x * t.global_scale.x) * t.global_rotation.cos()
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
                let mut new_scale = if let Some(p_l) = c.left_boundary {
                    if let Some(p_r) = c.right_boundary {
                        if (p_l - p_r).abs() < scale_x {
                            (p_l - p_r).abs()
                        } else {
                            scale_x
                        }
                    } else {
                        scale_x
                    }
                } else {
                    scale_x
                };

                if let Some(p_t) = c.top_boundary {
                    if let Some(p_b) = c.bottom_boundary {
                        if (p_t - p_b).abs() < new_scale {
                            new_scale = (p_t - p_b).abs();
                        }
                    }
                };

                if let Some(z) = c.min_zoom {
                    if z > new_scale{
                        new_scale = z;
                    }
                }
                if let Some(z) = c.max_zoom {
                    if z < new_scale{
                        new_scale = z;
                    }
                }
                let mut new_x = x ;
                println!("{}",new_scale);
                println!("{}",x - new_scale / 2.);
                if let Some(l) = c.left_boundary{
                    if l > x - new_scale / 2.0 {
                        new_x = l + new_scale / 2.0
                    }
                }
                if let Some(r) = c.right_boundary{
                    if r < x + new_scale / 2.0{
                        new_x = r - new_scale / 2.0
                    }
                }
                let mut new_y = y ;

                if let Some(b) = c.bottom_boundary{
                    if b > y - new_scale * get_window_ratio() / 2.0{
                        new_y = b + new_scale * get_window_ratio() / 2.0;
                    }
                }
                if let Some(t) = c.top_boundary{
                    if t < y + new_scale * get_window_ratio() / 2.0{
                        new_y = t - new_scale * get_window_ratio() / 2.0;
                    }
                }

                (new_x, new_y, new_scale)
            } else {
                continue;
            };
            Move2D::start(cam_poses.0)
                .set_transition_global(pos_x, pos_y)
                .set_scale_global(scale,scale).finish();
            if let Reference::Some(t) = &mut *cam_poses.0.dry_f_write(){
                t.dirty = false;
            }
        };
        //let x =
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
                write_to_buffer(b, 0, bytemuck::cast_slice(&[Mat3::identity().to_mat4_4()]));
            }
        }
    } else {
        panic!("Run init_camera in setup.")
    }
}
