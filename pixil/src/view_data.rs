/*use bytemuck::cast_slice;
use corrosive_asset_manager::cache_server::{Cache, CacheServer};
use corrosive_asset_manager_macro::static_hasher;
use corrosive_ecs_renderer_backend::assets::BindGroupLayoutAsset;
use corrosive_ecs_renderer_backend::public_functions::{
    create_bind_group, create_bind_group_layout, create_buffer_init, get_absolute_window_resolution,
};
use corrosive_ecs_renderer_backend::wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    Buffer, BufferBindingType, BufferUsages, ShaderStages,
};
use glam::Mat4;
use std::sync::LazyLock;

pub struct ViewData {
    pub bind_group: BindGroup,
    pub resolution_buffer: Buffer,
    pub view_buffer: Buffer,
    pub position_buffer: Buffer,
    pub near_far_buffer: Buffer,
}

pub static VIEW_DATA: LazyLock<ViewData> = LazyLock::new(|| {
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
                    ],
                }),
            })
        });

    let r_buffer = create_buffer_init(
        "PixilRestitutionBuffer",
        cast_slice(&[360u32, 360u32]),
        BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    );
    let v_buffer = create_buffer_init(
        "PixilViewBuffer",
        cast_slice(&Mat4::IDENTITY.to_cols_array()),
        BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    );
    let position_buffer = create_buffer_init(
        "PixilPositionBuffer",
        cast_slice(&[0.0f32,0.0f32,0.0f32]),
        BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    );
    let near_far_buffer = create_buffer_init(
        "PixilPositionBuffer",
        cast_slice(&[0,10]),
        BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    );
    let bind_group = create_bind_group(
        "PixilViewBindGroup",
        &view_layout.get().layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: r_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: v_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: v_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: v_buffer.as_entire_binding(),
            },
        ],
    );
    ViewData {
        bind_group,
        resolution_buffer: r_buffer,
        view_buffer: v_buffer,
        position_buffer,
        near_far_buffer,
    }
});
*/
