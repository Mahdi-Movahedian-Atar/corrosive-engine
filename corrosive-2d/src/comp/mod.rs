pub mod camera2d;
pub mod sprite2d;

use crate::comp::camera2d::ActiveCamera2D;
use crate::material2d::{Material2D, Material2DWrapper, StandardMaterial2D};
use crate::mesh2d::Vertex2D;
use corrosive_asset_manager::asset_server::{Asset, AssetServer};
use corrosive_asset_manager::dynamic_hasher;
use corrosive_asset_manager_macro::static_hasher;
use corrosive_ecs_core::ecs_core::{Member, Reference, Res, SharedBehavior};
use corrosive_ecs_core::trait_for;
use corrosive_ecs_core_macro::{trait_bound, Component, Resource};
use corrosive_ecs_renderer_backend::assets::{BindGroupLayoutAsset, PipelineAsset};
use corrosive_ecs_renderer_backend::material::Material;
use corrosive_ecs_renderer_backend::public_functions::*;
use corrosive_ecs_renderer_backend::wgpu::*;
use crossbeam_channel::{Receiver, Sender};
use glam::{EulerRot, Mat3, Mat4, Quat, Vec2, Vec3};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Depth {
    Far,
    Back,
    Mid,
    Front,
    Near,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Position2D {
    pub depth: Depth,
    pub local_position: Vec2,
    pub local_rotation: f32,
    pub local_scale: Vec2,

    pub global_position: Vec2,
    pub global_rotation: f32,
    pub global_scale: Vec2,
    pub(crate) dirty: bool,
}
impl Default for Position2D {
    fn default() -> Self {
        Self {
            depth: Depth::Mid,
            local_position: Vec2 { x: 0.0, y: 0.0 },
            local_rotation: 0.0,
            local_scale: Vec2 { x: 1.0, y: 1.0 },
            global_position: Vec2 { x: 0.0, y: 0.0 },
            global_rotation: 0.0,
            global_scale: Vec2 { x: 1.0, y: 1.0 },
            dirty: true,
        }
    }
}
impl Position2D {
    pub fn new() -> Self {
        Self {
            depth: Depth::Mid,
            local_position: Vec2 { x: 0.0, y: 0.0 },
            local_rotation: 0.0,
            local_scale: Vec2 { x: 1.0, y: 1.0 },
            global_position: Vec2 { x: 0.0, y: 0.0 },
            global_rotation: 0.0,
            global_scale: Vec2 { x: 1.0, y: 1.0 },
            dirty: true,
        }
    }

    pub fn local_matrix(&self) -> Mat3 {
        Mat3::from_scale_angle_translation(
            self.local_scale,
            self.local_rotation,
            self.local_position,
        )
    }

    pub fn global_matrix(&self) -> Mat3 {
        Mat3::from_scale_angle_translation(
            self.global_scale,
            self.global_rotation,
            self.global_position,
        )
    }

    pub fn move_position(&mut self, position: Vec2) {
        self.local_position.x += position.x;
        self.local_position.y += position.y;
    }

    pub fn rotate(&mut self, rotation: f32) {
        self.local_rotation += rotation;
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        self.local_scale = scale;
    }

    pub fn uniform_matrix(&self) -> [[f32; 4]; 4] {
        Mat4::from_scale_rotation_translation(
            self.global_scale.extend(1.0),
            Quat::from_axis_angle(Vec3::Z, self.global_rotation),
            self.global_position.extend(0.0),
        )
        .to_cols_array_2d()
    }
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        let sx = 1.0 / self.global_scale.x;
        let sy = get_window_ratio() / self.global_scale.x;
        Mat4::from_scale_rotation_translation(
            Vec3::new(sx, sy, 1.0),
            Quat::from_axis_angle(Vec3::Z, self.global_rotation),
            -Vec3::new(
                self.global_position.x * sx,
                self.global_position.y * sx,
                0.0,
            ),
        )
        .to_cols_array_2d()
    }
}
impl SharedBehavior for Position2D {
    fn shaded_add_behavior(&mut self, parent: &Self) {
        let parent_matrix = parent.global_matrix();
        let local_matrix = self.local_matrix();
        let global_matrix = parent_matrix * local_matrix;

        self.global_scale = Vec2::new(global_matrix.x_axis.length(), global_matrix.y_axis.length());

        let rotation_matrix = Mat3::from_cols(
            global_matrix.x_axis / self.global_scale.x,
            global_matrix.y_axis / self.global_scale.y,
            Vec3::Z,
        );
        let rotation_quat = Quat::from_mat3(&rotation_matrix);
        self.global_rotation = rotation_quat.to_euler(EulerRot::XYZ).2;

        self.global_position = Vec2::new(global_matrix.z_axis.x, global_matrix.z_axis.y);
        self.dirty = true;
    }

    fn shaded_remove_behavior(&mut self) {
        self.global_position = self.local_position;
        self.global_rotation = self.local_rotation;
        self.global_scale = self.local_scale;
        self.dirty = true;
    }
}

#[trait_bound]
pub trait Mesh2D {
    fn draw(&self, render_pass: &mut RenderPass);
    fn update(&self, render_pass: &mut RenderPass);
    fn name<'a>(&self) -> &'a str;
    fn get_bind_group_layout_desc(&self) -> &Asset<BindGroupLayoutAsset>;
}
#[derive(Component)]
pub struct RendererMeta2D {
    pub(crate) pipeline_asset: Asset<PipelineAsset>,
    pub(crate) transform_data: (Buffer, BindGroup),
    pub(crate) material: Box<dyn Material2DWrapper + Send + Sync>,
    pub(crate) mat_bind_group: &'static BindGroup,
    pub(crate) depth: UnsafeCell<Depth>,
    pub(crate) is_active: AtomicBool,
}

unsafe impl Send for RendererMeta2D {}
unsafe impl Sync for RendererMeta2D {}

impl RendererMeta2D {
    pub fn new(
        material_2d: &Asset<impl Material2D>,
        mesh_2d: &impl Mesh2D,
        position_2d: &Member<Position2D>,
        active_camera2d: &Res<ActiveCamera2D>,
    ) -> Self {
        let material_2d_wrapper = material_2d.get().generate_wrapper(material_2d.clone());
        let bind_group_layout: Asset<BindGroupLayoutAsset> =
            AssetServer::add(static_hasher!("2DTransformBindGroupLayout"), || {
                Ok(BindGroupLayoutAsset {
                    layout: create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: "2DTransformBindGroupLayoutDescriptor".into(),
                        entries: &[
                            BindGroupLayoutEntry {
                                binding: 0,
                                visibility: ShaderStages::VERTEX_FRAGMENT,
                                ty: BindingType::Buffer {
                                    ty: BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            BindGroupLayoutEntry {
                                binding: 1,
                                visibility: ShaderStages::VERTEX_FRAGMENT,
                                ty: BindingType::Buffer {
                                    ty: BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                    }),
                })
            });

        let mut depth = Depth::Mid;

        let transform_buffer = create_buffer_init(
            "TransformBuffer",
            &match &*position_2d.f_read() {
                Reference::Some(t) => {
                    depth = t.depth;
                    bytemuck::cast_slice(&[t.uniform_matrix()]).to_vec()
                }
                Reference::Expired => {
                    bytemuck::cast_slice(&[Mat4::IDENTITY.to_cols_array_2d()]).to_vec()
                }
            },
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let transform_bind_group = create_bind_group(
            "TransformBindGroup",
            &bind_group_layout.get().layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: <Option<Buffer> as Clone>::clone(&active_camera2d.f_read().buffer)
                        .unwrap()
                        .as_entire_binding(),
                },
            ],
        );

        let pipeline_asset: Asset<PipelineAsset> = AssetServer::add_sync(
            dynamic_hasher(format!("{}{}", mesh_2d.name(), material_2d.get().get_name()).as_str()),
            || {
                let material2d = material_2d.get();
                let shader = material2d.get_shader();
                Ok(PipelineAsset {
                    layout: create_pipeline(&RenderPipelineDescriptor {
                        label: format!("{}{}", mesh_2d.name(), material2d.get_name())
                            .as_str()
                            .into(),
                        layout: Some(&create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: "ui_pipeline_layout".into(),
                            bind_group_layouts: &[
                                &bind_group_layout.get().layout,
                                get_resolution_bind_group_layout(),
                                &mesh_2d.get_bind_group_layout_desc().get().layout,
                                &material2d.get_bind_group_layout().get().layout,
                            ],
                            push_constant_ranges: &[],
                        })),
                        vertex: VertexState {
                            module: &create_shader_module(shader.0, shader.1.as_str()),
                            entry_point: "vs_main".into(),
                            compilation_options: Default::default(),
                            buffers: &[VertexBufferLayout {
                                array_stride: size_of::<Vertex2D>() as BufferAddress,
                                step_mode: VertexStepMode::Vertex,
                                attributes: &[
                                    VertexAttribute {
                                        format: VertexFormat::Float32x3,
                                        offset: 0,
                                        shader_location: 0,
                                    },
                                    VertexAttribute {
                                        format: VertexFormat::Float32x2,
                                        offset: size_of::<[f32; 3]>() as BufferAddress,
                                        shader_location: 1,
                                    },
                                ],
                            }],
                        },
                        primitive: PrimitiveState {
                            topology: PrimitiveTopology::TriangleStrip,
                            strip_index_format: None,
                            front_face: FrontFace::Ccw,
                            cull_mode: Face::Front.into(),
                            unclipped_depth: false,
                            polygon_mode: PolygonMode::Fill,
                            conservative: false,
                        },
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: FragmentState {
                            module: &create_shader_module(shader.0, shader.1.as_str()),
                            entry_point: "fs_main".into(),
                            compilation_options: Default::default(),
                            targets: &[ColorTargetState {
                                format: get_surface_format(),
                                blend: BlendState {
                                    color: BlendComponent {
                                        src_factor: BlendFactor::SrcAlpha,
                                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                                        operation: BlendOperation::Add,
                                    },
                                    alpha: BlendComponent {
                                        src_factor: BlendFactor::One,
                                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                                        operation: BlendOperation::Add,
                                    },
                                }
                                .into(),
                                write_mask: ColorWrites::ALL,
                            }
                            .into()],
                        }
                        .into(),
                        multiview: None,
                        cache: None,
                    }),
                })
            },
        );
        Self {
            pipeline_asset,
            transform_data: (transform_buffer, transform_bind_group),
            mat_bind_group: material_2d_wrapper.get_bind_group(),
            material: material_2d_wrapper,
            depth: UnsafeCell::new(depth),
            is_active: AtomicBool::new(true),
        }
    }

    pub fn disable(&self) {
        self.is_active.store(false, Ordering::Relaxed);
    }
    pub fn enable(&self) {
        self.is_active.store(true, Ordering::Relaxed);
    }
}

#[derive(Resource, Default)]
pub struct Renderer2dData {
    pub(crate) data: Option<(Receiver<RenderPass<'static>>, Sender<()>)>,
}
