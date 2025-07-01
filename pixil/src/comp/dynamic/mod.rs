use crate::comp::position_pixil::PositionPixil;
use crate::helper_functions::transform_bind_group_layout;
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
    pub(crate) id: usize,
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
        let material_ref = material.get();

        let bind_group_layout: Cache<BindGroupLayoutAsset> = transform_bind_group_layout();

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

        let id = DYNAMIC_OBJECTS.add_enabled(PixilDynamicObjectData {
            vertex_buffer: &mesh.get().vertex_buffer,
            index_buffer: &mesh.get().index_buffer,
            transform_bind_group,
            material_bind_group: material.get().get_layout_bind_group(),
            count: &mesh.get().index_count,
            pipeline: &material.get().get_layout(),
        });

        Self {
            id,
            mesh,
            material: material_ref.generate_wrapper(material.clone()),
            transform_data: (transform_buffer),
        }
    }
    pub fn enable(&self){
         DYNAMIC_OBJECTS.enable(self.id)
    }
    pub fn disable(&self){
        DYNAMIC_OBJECTS.disable(self.id)
    }
}

impl Drop for PixilDynamicObject {
    fn drop(&mut self) {
        DYNAMIC_OBJECTS.remove(self.id)
    }
}
