pub mod rect2d;

use crate::material2d::{Material2D, Material2DWrapper, StandardMaterial2D};
use crate::math2d::{Mat3, Vec2};
use crate::mesh2d::Vertex2D;
use crate::task::UnsafeRenderPass;
use corrosive_asset_manager::asset_server::{Asset, AssetServer};
use corrosive_asset_manager::dynamic_hasher;
use corrosive_asset_manager_macro::static_hasher;
use corrosive_ecs_core::ecs_core::{Member, Reference, SharedBehavior};
use corrosive_ecs_core::trait_for;
use corrosive_ecs_core_macro::{trait_bound, Component, Resource};
use corrosive_ecs_renderer_backend::assets::{BindGroupLayoutAsset, PipelineAsset};
use corrosive_ecs_renderer_backend::helper::{
    create_bind_group, create_bind_group_layout, create_buffer_init, create_pipeline,
    create_pipeline_layout, create_shader_module, get_surface_format, BindGroup, BindGroupEntry,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor,
    BlendOperation, BlendState, Buffer, BufferAddress, BufferBindingType, BufferUsages,
    ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPass, RenderPipelineDescriptor,
    ShaderStage, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use corrosive_ecs_renderer_backend::material::Material;
use crossbeam_channel::{Receiver, Sender};
use std::ptr::NonNull;
use std::sync::Arc;

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

#[trait_bound]
pub trait Mesh2D {
    fn draw(&self);
    fn update(&self, render_pass: &mut RenderPass);
    fn name<'a>(&self) -> &'a str;
    fn get_bind_group_layout_desc(&self) -> Asset<BindGroupLayoutAsset>;
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
        mesh_2d: impl Mesh2D,
        position_2d: &Member<Position2D>,
    ) -> Self {
        let material_2d_wrapper = material_2d.get().generate_wrapper(material_2d.clone());
        let bind_group_layout: Asset<BindGroupLayoutAsset> =
            AssetServer::add(static_hasher!("2DTransformBindGroupLayout"), || {
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

        let pipeline_asset: Asset<PipelineAsset> =
            AssetServer::add_sync(static_hasher!("ss"), || {
                let material2d = material_2d.get();
                let shader = material2d.get_shader();
                PipelineAsset {
                    layout: create_pipeline(&RenderPipelineDescriptor {
                        label: format!("{}{}", mesh_2d.name(), material2d.get_name())
                            .as_str()
                            .into(),
                        layout: Some(&create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: "ui_pipeline_layout".into(),
                            bind_group_layouts: &[
                                &bind_group_layout.get().layout,
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
                }
            });
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
