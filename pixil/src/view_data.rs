use bytemuck::cast_slice;
use corrosive_ecs_renderer_backend::public_functions::{
    create_bind_group, create_bind_group_layout, create_buffer_init,
};
use corrosive_ecs_renderer_backend::wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    Buffer, BufferBindingType, BufferUsages, ShaderStages,
};
use glam::Mat4;
use std::sync::LazyLock;

pub struct ViewData {
    pub bind_group: BindGroup,
    pub restitution_buffer: Buffer,
    pub view_buffer: Buffer,
}

pub static VIEW_DATA: LazyLock<ViewData> = LazyLock::new(|| {
    let r_buffer = create_buffer_init(
        "PixilRestitutionBuffer",
        cast_slice(&[0u32, 0u32]),
        BufferUsages::UNIFORM,
    );
    let v_buffer = create_buffer_init(
        "PixilViewBuffer",
        cast_slice(&Mat4::IDENTITY.to_cols_array()),
        BufferUsages::UNIFORM,
    );
    let bind_group = create_bind_group(
        "PixilViewBindGroup",
        &create_bind_group_layout(&BindGroupLayoutDescriptor {
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
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }),
        &[
            BindGroupEntry {
                binding: 0,
                resource: r_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: v_buffer.as_entire_binding(),
            },
        ],
    );
    ViewData {
        bind_group,
        restitution_buffer: r_buffer,
        view_buffer: v_buffer,
    }
});
