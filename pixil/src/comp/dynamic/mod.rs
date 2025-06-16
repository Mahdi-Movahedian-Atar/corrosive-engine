use crate::comp::position_pixil::PositionPixil;
use crate::material::{PixilMaterial, PixilMaterialWrapper};
use crate::mesh::{Mesh, Vertex};
use crate::task::renderer::DYNAMIC_OBJECTS;
use corrosive_asset_manager::asset_server::{Asset, AssetServer};
use corrosive_asset_manager::cache_server::{Cache, CacheServer};
use corrosive_asset_manager_macro::static_hasher;
use corrosive_ecs_core::ecs_core::{Member, Reference};
use corrosive_ecs_core_macro::Component;
use corrosive_ecs_renderer_backend::assets::{BindGroupLayoutAsset, PipelineAsset};
use corrosive_ecs_renderer_backend::public_functions::{
    create_bind_group, create_bind_group_layout, create_buffer_init, create_pipeline,
    create_pipeline_layout, create_shader_module, get_device, get_surface_format,
};
use corrosive_ecs_renderer_backend::wgpu;
use corrosive_ecs_renderer_backend::wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer, BufferAddress,
    BufferBindingType, BufferUsages, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
    IndexFormat, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderBundleDescriptor, RenderBundleEncoderDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use glam::Mat4;
use std::sync::LazyLock;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        normal: [0.0, 0.0, 1.0],
    },
];

pub struct PixilDynamicObjectData {
    pub vertex_buffer: &'static Buffer,
    pub index_buffer: &'static Buffer,
    pub transform_bind_group: BindGroup,
    pub material_bind_group: &'static BindGroup,
    pub count: &'static u32,
    pub pipeline: &'static RenderPipeline,
}

#[derive(Component)]
pub struct PixilDynamicObject {
    pub mesh: Asset<Mesh>,
    pub(crate) material: Box<dyn PixilMaterialWrapper + Send + Sync>,
    pub(crate) transform_data: Buffer,
}
impl PixilDynamicObject {
    pub fn new(
        mesh: Asset<Mesh>,
        material: &Asset<impl PixilMaterial>,
        position_pixil: &Member<PositionPixil>,
        name: &str,
    ) -> Self {
        /*let mesh = Mesh{
            vertex_buffer: create_buffer_init(
                    "Vertex Buffer",
                    bytemuck::cast_slice(VERTICES),
                    BufferUsages::VERTEX,
            ),
            index_buffer: create_buffer_init(
                "Index Buffer",
                bytemuck::cast_slice(&[0,1,2,0]),
                BufferUsages::INDEX,
            ),
            index_count: 4,
        };*/

        let material_ref = material.get();

        let bind_group_layout: Cache<BindGroupLayoutAsset> =
            CacheServer::add(static_hasher!("PixilTransformBindGroupLayout"), || {
                Ok(BindGroupLayoutAsset {
                    layout: create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: "PixilTransformBindGroupLayoutDescriptor".into(),
                        entries: &[BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::VERTEX_FRAGMENT,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                    }),
                })
            });

        let transform_buffer = create_buffer_init(
            "PixilTransformBuffer",
            &match &*position_pixil.f_read() {
                Reference::Some(t) => bytemuck::cast_slice(&[t.uniform()]).to_vec(),
                Reference::Expired => {
                    bytemuck::cast_slice(&[Mat4::IDENTITY.to_cols_array_2d()]).to_vec()
                }
            },
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let transform_bind_group = create_bind_group(
            "PixilTransformBindGroup",
            &bind_group_layout.get().layout,
            &[BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        );

        /*let mut bundle =
            get_device().create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
                label: name.into(),
                color_formats: &[Option::from(get_surface_format())],
                depth_stencil: None,
                sample_count: 1,
                multiview: None,
            });
        bundle.set_pipeline(material_ref.get_layout());
        bundle.set_bind_group(0, &VIEW_DATA.bind_group, &[]);
        bundle.set_bind_group(1, &transform_bind_group, &[]);
        bundle.set_bind_group(2, &transform_bind_group, &[]);
        bundle.set_vertex_buffer(0, mesh.get().vertex_buffer.slice(..));
        bundle.set_index_buffer(mesh.get().index_buffer.slice(..), IndexFormat::Uint32);
        bundle.draw_indexed(0..mesh.get().index_count, 0, 0..1);*/
        //bundle.finish(&RenderBundleDescriptor { label: name.into() };

        DYNAMIC_OBJECTS.add_enabled(PixilDynamicObjectData {
            vertex_buffer: &mesh.get().vertex_buffer,
            index_buffer: &mesh.get().index_buffer,
            transform_bind_group,
            material_bind_group: material.get().get_layout_bind_group(),
            count: &mesh.get().index_count,
            pipeline: &material.get().get_layout(),
        });

        Self {
            mesh,
            material: material_ref.generate_wrapper(material.clone()),
            transform_data: (transform_buffer),
        }
    }
}
