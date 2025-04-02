use corrosive_asset_manager::Asset;
use corrosive_asset_manager_macro::{static_hasher, Asset};
use corrosive_ecs_renderer_backend::assets::{BindGroupLayoutAsset, ShaderAsset};
use corrosive_ecs_renderer_backend::color::Color;
use corrosive_ecs_renderer_backend::helper::{
    create_bind_group, create_bind_group_layout, create_buffer_init, create_shader_module,
    get_queue, BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferUsages,
    PipelineLayoutDescriptor, ShaderModule, ShaderStage, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexStepMode,
};
use corrosive_ecs_renderer_backend::material::{Material, MaterialDesc};

#[derive(Asset, Clone)]
pub struct Image2DMaterial {
    pub overlay_color: Color,
    overlay_color_buffer: Buffer,
    bind_group: BindGroup,
    shader: Asset<ShaderAsset>,
}
impl Image2DMaterial {
    pub fn new(overlay_color: Color) -> Self {
        let buffer = create_buffer_init(
            "Image2DMaterialOverlayColorBuffer",
            &overlay_color.to_bytes(),
            BufferUsages::UNIFORM,
        );
        Self {
            overlay_color,
            bind_group: create_bind_group(
                "Image2DMaterialBingGroup",
                &Image2DMaterial::get_bind_group_layout_desc().get().layout,
                &[BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            ),
            overlay_color_buffer: buffer,
            shader: Asset::load(static_hasher!("Image2DMaterialShader"), || ShaderAsset {
                shader: create_shader_module("Image2DMaterialShader", include_str!("image2d.wgsl")),
            }),
        }
    }
    pub fn update(&self) {
        get_queue().write_buffer(
            &self.overlay_color_buffer,
            0,
            &self.overlay_color.to_bytes(),
        );
    }
    pub fn set_color(&mut self, color: Color) {
        self.overlay_color = color;
        get_queue().write_buffer(
            &self.overlay_color_buffer,
            0,
            &self.overlay_color.to_bytes(),
        );
    }
}

impl MaterialDesc for Image2DMaterial {
    fn get_name_desc<'a>() -> &'a str {
        "Image2DMaterial"
    }

    fn get_bind_group_layout_desc() -> Asset<BindGroupLayoutAsset> {
        Asset::load(static_hasher!("Image2DMaterialBindGroupLayout"), || {
            BindGroupLayoutAsset {
                layout: create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: "Image2DMaterialBindGroupLayoutDescriptor".into(),
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
        })
    }
}

impl Material for Image2DMaterial {
    fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn get_shader(&self) -> &ShaderModule {
        &Self.shader.get().shader
    }
}
