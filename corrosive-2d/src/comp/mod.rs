pub mod camera2d;
pub mod sprite2d;

use crate::comp::camera2d::ActiveCamera2D;
use crate::material2d::{Material2D, Material2DWrapper, StandardMaterial2D};
use crate::math2d::{Mat3, Vec2};
use crate::mesh2d::Vertex2D;
use crate::task::UnsafeRenderPass;
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

#[derive(Debug, Clone, Copy, Component)]
pub struct Position2D {
    pub depth: f32,
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
            depth: 0.0,
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
            depth: 0.0,
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
        Mat3::from_scale_rotation_translation(
            self.local_scale,
            self.local_rotation,
            self.local_position,
        )
    }

    pub fn global_matrix(&self) -> Mat3 {
        Mat3::from_scale_rotation_translation(
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
        let sx = self.global_scale.x;
        let sy = self.global_scale.y;
        let rotation = self.global_rotation;
        let tx = self.global_position.x;
        let ty = self.global_position.y;
        let tz = self.depth;

        let cos = rotation.cos();
        let sin = rotation.sin();
        [
            [cos * sx, sin * sx, 0.0, 0.0],
            [-sin * sy, cos * sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [tx * sx, ty * sy, tz, 1.0],
        ]
    }
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        let inv_zoom = 1.0 / self.global_scale.x;
        let (sin, cos) = (-self.global_rotation).sin_cos(); // Negative for view matrix

        // Combined rotation and scaling (inverse of camera transform)
        let a = cos * inv_zoom;
        let b = sin * inv_zoom;
        let c = -sin * inv_zoom * get_window_ratio();
        let d = cos * inv_zoom * get_window_ratio();

        // Inverse translation components (negative camera position)
        let tx = -self.global_position.x;
        let ty = -self.global_position.y;

        // Apply rotation to translation before scaling
        let rotated_tx = tx * cos - ty * sin;
        let rotated_ty = tx * sin + ty * cos;

        // Column-major matrix (suitable for OpenGL/WebGPU)
        let view = [
            [a,    c,    0.0, 0.0],  // First column
            [b,    d,    0.0, 0.0],  // Second column
            [0.0,  0.0,  1.0, 0.0],  // Third column
            [
                rotated_tx * inv_zoom,
                rotated_ty * inv_zoom * get_window_ratio(),
                0.0,
                1.0
            ],  // Fourth column
        ];

        let proj = [
            [self.global_scale.x / get_window_ratio(), 0.0,        0.0, 0.0],
            [0.0,           self.global_scale.x,       0.0, 0.0],
            [0.0,           0.0,        1.0, 0.0],
            [0.0,           0.0,        0.0, 1.0],
        ];

        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = proj[i][0] * view[0][j]
                    + proj[i][1] * view[1][j]
                    + proj[i][2] * view[2][j]
                    + proj[i][3] * view[3][j];
            }
        }

        result
    }
}
impl SharedBehavior for Position2D {
    fn shaded_add_behavior(&mut self, parent: &Self) {
        let parent_matrix = parent.global_matrix();
        let local_matrix = self.local_matrix();
        let global_matrix = parent_matrix.multiply(&local_matrix);

        let (scale, rotation, translation) = global_matrix.decompose();

        self.global_scale = scale;
        self.global_rotation = rotation;
        self.global_position = translation;
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
}

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

        let transform_buffer = create_buffer_init(
            "TransformBuffer",
            &match &*position_2d.f_read() {
                Reference::Some(t) => {
                    println!("{:?}", t.uniform_matrix());
                    bytemuck::cast_slice(&[t.uniform_matrix()]).to_vec()
                }
                Reference::Expired => {
                    bytemuck::cast_slice(&[Mat3::identity().to_mat4_4()]).to_vec()
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
        }
    }
}

#[derive(Resource, Default)]
pub struct Renderer2dData {
    pub(crate) data: Option<(Receiver<UnsafeRenderPass>, Sender<()>)>,
}
