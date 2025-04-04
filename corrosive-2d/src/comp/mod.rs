pub mod image2d;
pub mod material2d;

use crate::math2d::{Mat3, Vec2};
use crate::mesh2d::Vertex2D;
use corrosive_asset_manager::comp::Asset;
use corrosive_asset_manager_macro::static_hasher;
use corrosive_ecs_core::ecs_core::{Member, Reference, SharedBehavior};
use corrosive_ecs_core_macro::Component;
use corrosive_ecs_renderer_backend::comp::assets::{BindGroupLayoutAsset, PipelineAsset};
use corrosive_ecs_renderer_backend::helper::{
    create_bind_group, create_bind_group_layout, create_buffer_init, create_pipeline,
    create_pipeline_layout, get_surface_format, BindGroup, BindGroupEntry,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor,
    BlendOperation, BlendState, Buffer, BufferAddress, BufferBindingType, BufferUsages,
    ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipelineDescriptor, ShaderStage,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use corrosive_ecs_renderer_backend::material::Material;

#[derive(Debug, Clone, Copy, Component)]
pub struct Position2D {
    pub depth: f32,
    pub local_position: Vec2,
    pub local_rotation: f32,
    pub local_scale: Vec2,
    world_matrix: Mat3,
}
impl Default for Position2D {
    fn default() -> Self {
        Self {
            depth: 0.0,
            local_position: Vec2 { x: 0.0, y: 0.0 },
            local_rotation: 0.0,
            local_scale: Vec2 { x: 1.0, y: 1.0 },
            world_matrix: Mat3::identity(),
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
            world_matrix: Mat3::identity(),
        }
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

    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        Vec2 {
            x: self.world_matrix.m[2][0]
                + point.x * self.world_matrix.m[0][0]
                + point.y * self.world_matrix.m[1][0],
            y: self.world_matrix.m[2][1]
                + point.x * self.world_matrix.m[0][1]
                + point.y * self.world_matrix.m[1][1],
        }
    }

    pub fn get_world_position(&self) -> Vec2 {
        Vec2 {
            x: self.world_matrix.m[0][2],
            y: self.world_matrix.m[0][2],
        }
    }
}
pub fn move_world_transformation(member: Member<Position2D>, target: Vec2) {
    member.write(|v| {
        if let Reference::Some(v) = &mut *v.unwrap() {
            v.local_position = if let Some(t) = member.get_parent() {
                if let Reference::Some(t) = (&*t.f_read()) {
                    t.world_matrix
                        .clone()
                        .inverse()
                        .unwrap()
                        .transform_point(target)
                } else {
                    target
                }
            } else {
                target
            }
        }
    })
}
impl SharedBehavior for Position2D {
    fn shaded_add_behavior(&mut self, parent: &Self) {
        let translation = Mat3::translate(self.local_position);
        let rotation = Mat3::rotate(self.local_rotation);
        let scale = Mat3::scale(self.local_scale);

        let local_matrix = translation.multiply(&rotation).multiply(&scale);

        self.world_matrix = parent.world_matrix.multiply(&local_matrix);
    }

    fn shaded_remove_behavior(&mut self) {
        let translation = Mat3::translate(self.local_position);
        let rotation = Mat3::rotate(self.local_rotation);
        let scale = Mat3::scale(self.local_scale);

        let local_matrix = translation.multiply(&rotation).multiply(&scale);

        self.world_matrix = local_matrix
    }
}

pub trait Mesh2D {
    fn draw(&self);
    fn update(&self);
}
pub trait Mesh2DDesc: Mesh2D {
    fn name<'a>() -> &'a str;
}

#[derive(Component)]
pub struct RendererMeta2D {
    pipeline_asset: Asset<PipelineAsset>,
    transform_data: (Buffer, BindGroup),
    material2d: Asset<dyn Material>,
}

impl RendererMeta2D {
    pub fn new<F: Mesh2DDesc>(
        material2d: Asset<impl Material>,
        position_2d: &Member<Position2D>,
    ) -> Self {
        let bind_group_layout: Asset<BindGroupLayoutAsset> =
            Asset::load(static_hasher!("2DTransformBindGroupLayout"), || {
                BindGroupLayoutAsset {
                    layout: create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: "2DTransformBindGroupLayoutDescriptor".into(),
                        entries: &[BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStage::FRAGMENT,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                    }),
                }
            });

        let transform_buffer = create_buffer_init(
            "TransformBuffer",
            &match &*position_2d.f_read() {
                Reference::Some(t) => bytemuck::cast_slice(&[t.world_matrix]).to_vec(),
                Reference::Expired => bytemuck::cast_slice(&[Mat3::identity()]).to_vec(),
            },
            BufferUsages::UNIFORM,
        );
        let transform_bind_group = create_bind_group(
            "TransformBindGroup",
            &bind_group_layout.get().layout,
            &[BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        );

        let new_mat = material2d.clone();

        let pipeline_asset: Asset<PipelineAsset> = Asset::load(static_hasher!("ss"), move || {
            let material2d = material2d.get();
            let bind_group_layout: Asset<BindGroupLayoutAsset> =
                Asset::load(static_hasher!("2DTransformBindGroupLayout"), || {
                    BindGroupLayoutAsset {
                        layout: create_bind_group_layout(&BindGroupLayoutDescriptor {
                            label: "2DTransformBindGroupLayoutDescriptor".into(),
                            entries: &[BindGroupLayoutEntry {
                                binding: 0,
                                visibility: ShaderStage::FRAGMENT,
                                ty: BindingType::Buffer {
                                    ty: BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            }],
                        }),
                    }
                });
            PipelineAsset {
                layout: create_pipeline(&RenderPipelineDescriptor {
                    label: format!("{}{}", F::name(), material2d.get_name())
                        .as_str()
                        .into(),
                    layout: Some(&create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: "ui_pipeline_layout".into(),
                        bind_group_layouts: &[
                            &bind_group_layout.get().layout,
                            &material2d.get_bind_group_layout(&Res {}).get().layout,
                        ],
                        push_constant_ranges: &[],
                    })),
                    vertex: VertexState {
                        module: &material2d.get_shader(),
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
                        module: material2d.get_shader(),
                        entry_point: "fs_main".into(),
                        compilation_options: Default::default(),
                        targets: &[ColorTargetState {
                            format: get_surface_format(),
                            blend: BlendState {
                                color: BlendComponent {
                                    src_factor: BlendFactor::SrcAlpha,         // Source: Alpha
                                    dst_factor: BlendFactor::OneMinusSrcAlpha, // Destination: 1 - Alpha
                                    operation: BlendOperation::Add, // Standard Alpha Blending
                                },
                                alpha: BlendComponent {
                                    src_factor: BlendFactor::One,              // Preserve Alpha
                                    dst_factor: BlendFactor::OneMinusSrcAlpha, // Blend Based on Alpha
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
            }
        });
        Self {
            pipeline_asset,
            transform_data: (transform_buffer, transform_bind_group),
            material2d,
        }
    }
}
