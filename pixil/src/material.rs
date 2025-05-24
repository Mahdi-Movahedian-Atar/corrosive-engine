use crate::mesh::Vertex;
use corrosive_asset_manager::asset_server::{Asset, AssetServer};
use corrosive_asset_manager::cache_server::{Cache, CacheServer};
use corrosive_asset_manager_macro::{Asset, static_hasher};
use corrosive_ecs_renderer_backend::assets::{BindGroupLayoutAsset, PipelineAsset};
use corrosive_ecs_renderer_backend::public_functions::{create_bind_group_layout, create_pipeline, create_pipeline_layout, get_device, get_surface_format, read_shader};
use corrosive_ecs_renderer_backend::wgpu::{BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState, BufferAddress, BufferBindingType, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderBundleEncoder, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode};

pub trait PixilMaterial {
    fn encode_to_bundle(&self, encoder: &mut RenderBundleEncoder);
    fn get_layout(&self) -> &RenderPipeline;
    fn new() -> Self
    where
        Self: Sized;
    fn generate_wrapper(&self, asset: Asset<Self>) -> Box<dyn PixilMaterialWrapper + Send + Sync>
    where
        Self: Sized;
}
pub trait PixilMaterialWrapper {}

#[derive(Asset)]
pub struct PixilDefaultMaterial {
    layout: Asset<PipelineAsset>,
}
pub struct PixilDefaultMaterialWrapper {
    material: Asset<PixilDefaultMaterial>,
}
impl PixilMaterial for PixilDefaultMaterial {
    fn encode_to_bundle(&self, encoder: &mut RenderBundleEncoder) {}

    fn get_layout(&self) -> &RenderPipeline {
        &self.layout.get().layout
    }

    fn new() -> Self {
        PixilDefaultMaterial {
            layout: AssetServer::add(static_hasher!("test"), || {
                let view_layout: Cache<BindGroupLayoutAsset> =
                    CacheServer::add(static_hasher!("ViewBindGroupLayout"), || {
                        Ok(BindGroupLayoutAsset {
                            layout: create_bind_group_layout(&BindGroupLayoutDescriptor {
                                label: "PixilViewBindGroupLayout".into(),
                                entries: &[
                                    BindGroupLayoutEntry {
                                        binding: 0,
                                        visibility: ShaderStages::FRAGMENT,
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
                let transfom_layout: Cache<BindGroupLayoutAsset> =
                    CacheServer::add(static_hasher!("PixilTransformBindGroupLayout"), || {
                        Ok(BindGroupLayoutAsset {
                            layout: create_bind_group_layout(&BindGroupLayoutDescriptor {
                                label: "PixilTransformBindGroupLayoutDescriptor".into(),
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
                                ],
                            }),
                        })
                    });

                let shader = get_device().create_shader_module(ShaderModuleDescriptor {
                    label: Some("pixil default shader"),
                    source: ShaderSource::Wgsl(
                        read_shader("packages/pixil/shaders/pixil_default_shader.wgsl")
                            .expect("failed to read shader")
                            .into(),
                    ),
                });

                Ok(PipelineAsset {
                    layout: create_pipeline(&RenderPipelineDescriptor {
                        label: "pixil default pipeline asset".into(),
                        layout: Some(&create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: "pixil temp".into(),
                            bind_group_layouts: &[&view_layout.get().layout, &transfom_layout.get().layout],
                            push_constant_ranges: &[],
                        })),
                        vertex: VertexState {
                            module: &shader,
                            entry_point: "vs_main".into(),
                            compilation_options: Default::default(),
                            buffers: &[VertexBufferLayout {
                                array_stride: size_of::<Vertex>() as BufferAddress,
                                step_mode: VertexStepMode::Vertex,
                                attributes: &[
                                    VertexAttribute {
                                        format: VertexFormat::Float32x3,
                                        offset: 0,
                                        shader_location: 0,
                                    },
                                    VertexAttribute {
                                        format: VertexFormat::Float32x3,
                                        offset: size_of::<[f32; 3]>() as BufferAddress,
                                        shader_location: 1,
                                    },
                                ],
                            }],
                        },
                        primitive: PrimitiveState {
                            topology: PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: FrontFace::Ccw,
                            cull_mode: Face::Back.into(),
                            unclipped_depth: false,
                            polygon_mode: PolygonMode::Fill,
                            conservative: false,
                        },
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: FragmentState {
                            module: &shader,
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
            }),
        }
    }

    fn generate_wrapper(&self, asset: Asset<Self>) -> Box<dyn PixilMaterialWrapper + Send + Sync> {
        Box::new(PixilDefaultMaterialWrapper { material: asset })
    }
}
impl PixilMaterialWrapper for PixilDefaultMaterialWrapper {}
