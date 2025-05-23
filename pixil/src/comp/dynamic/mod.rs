use glam::Mat4;
use crate::material::{PixilMaterial, PixilMaterialWrapper};
use crate::mesh::{Mesh, Vertex};
use crate::task::renderer::DYNAMIC_OBJECTS;
use crate::view_data::VIEW_DATA;
use corrosive_asset_manager::asset_server::{Asset, AssetServer};
use corrosive_asset_manager_macro::static_hasher;
use corrosive_ecs_core::ecs_core::{Member, Reference};
use corrosive_ecs_core_macro::Component;
use corrosive_ecs_renderer_backend::assets::{BindGroupLayoutAsset, PipelineAsset};
use corrosive_ecs_renderer_backend::public_functions::{create_bind_group, create_bind_group_layout, create_buffer_init, create_pipeline, create_pipeline_layout, create_shader_module, get_device, get_surface_format};
use corrosive_ecs_renderer_backend::wgpu;
use corrosive_ecs_renderer_backend::wgpu::{BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer, BufferAddress, BufferBindingType, BufferUsages, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, IndexFormat, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderBundleDescriptor, RenderBundleEncoderDescriptor, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode};
use crate::comp::position_pixil::PositionPixil;

#[derive(Component)]
pub struct PixilDynamicObject {
    pub mesh: Asset<Mesh>,
    pub(crate) material: Box<dyn PixilMaterialWrapper + Send + Sync>,
    pub(crate) transform_data: (Buffer, BindGroup),
}
impl PixilDynamicObject {
    pub fn new(mesh: Asset<Mesh>, material: &Asset<impl PixilMaterial>,position_pixil:&Member<PositionPixil> , name: &str) -> Self {
        let material_ref = material.get();

        //todo: add as cash
        let bind_group_layout: Asset<BindGroupLayoutAsset> =
            AssetServer::add(static_hasher!("PixilTransformBindGroupLayout"), || {
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


        let transform_buffer = create_buffer_init(
            "PixilTransformBuffer",
            &match &*position_pixil.f_read() {
                Reference::Some(t) => {
                    bytemuck::cast_slice(&[t.uniform()]).to_vec()
                }
                Reference::Expired => {
                    bytemuck::cast_slice(&[Mat4::IDENTITY.to_cols_array_2d()]).to_vec()
                }
            },
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let transform_bind_group = create_bind_group(
            "PixilTransformBindGroup",
            &bind_group_layout.get().layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                },
            ],
        );

        let mut bundle =
            get_device().create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
                label: name.into(),
                color_formats: &[Option::from(get_surface_format())],
                depth_stencil: None,
                sample_count: 1,
                multiview: None,
            });
        bundle.set_pipeline(material_ref.get_layout());
        material_ref.encode_to_bundle(&mut bundle);
        bundle.set_bind_group(0, &VIEW_DATA.bind_group, &[]);
        bundle.set_bind_group(1, &transform_bind_group, &[]);
        bundle.set_vertex_buffer(0, mesh.get().vertex_buffer.slice(..));
        bundle.set_index_buffer(mesh.get().index_buffer.slice(..), IndexFormat::Uint32);
        bundle.draw_indexed(0..mesh.get().index_count, 0, 0..1);

        DYNAMIC_OBJECTS.add_enabled(bundle.finish(&RenderBundleDescriptor { label: name.into() }));

        Self {
            mesh,
            material: material_ref.generate_wrapper(material.clone()),
            transform_data: (transform_buffer,transform_bind_group),
        }
    }
}
