use corrosive_asset_manager::cache_server::{Cache, CacheServer};
use corrosive_asset_manager_macro::static_hasher;
use corrosive_ecs_renderer_backend::assets::BindGroupLayoutAsset;
use corrosive_ecs_renderer_backend::public_functions::create_bind_group_layout;
use corrosive_ecs_renderer_backend::wgpu::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages,
};

pub(crate) fn view_bind_group_layout() -> Cache<BindGroupLayoutAsset> {
    CacheServer::get_or_add(static_hasher!("ViewBindGroupLayout"), || {
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
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            }),
        })
    })
}
pub(crate) fn transform_bind_group_layout() -> Cache<BindGroupLayoutAsset> {
    CacheServer::get_or_add(static_hasher!("PixilTransformBindGroupLayout"), || {
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
    })
}
