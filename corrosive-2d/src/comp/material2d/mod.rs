use corrosive_asset_manager::comp::{Asset, AssetServer, AssetTrait};
use corrosive_asset_manager_macro::{static_hasher, Asset};
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_renderer_backend::color::Color;
use corrosive_ecs_renderer_backend::comp::assets::{BindGroupLayoutAsset, ShaderAsset};
use corrosive_ecs_renderer_backend::helper::{
    create_bind_group, create_bind_group_layout, create_buffer_init, create_shader_module,
    get_queue, BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferUsages,
    PipelineLayoutDescriptor, ShaderModule, ShaderStage, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexStepMode,
};
use corrosive_ecs_renderer_backend::material::{Material, MaterialDesc};

#[derive(Clone)]
pub struct Image2DMaterial {
    pub overlay_color: Color,
    overlay_color_buffer: Buffer,
    bind_group: BindGroup,
    shader: Asset<ShaderAsset>,
}
impl Image2DMaterial {
    pub fn new(
        overlay_color: Color,
        shader_asset_server: &Res<AssetServer<ShaderAsset>>,
        bind_group_asset_server: &Res<AssetServer<BindGroupLayoutAsset>>,
    ) -> Self {
        let buffer = create_buffer_init(
            "Image2DMaterialOverlayColorBuffer",
            &overlay_color.to_bytes(),
            BufferUsages::UNIFORM,
        );
        Self {
            overlay_color,
            bind_group: create_bind_group(
                "Image2DMaterialBingGroup",
                &Image2DMaterial::get_bind_group_layout_desc(bind_group_asset_server)
                    .get()
                    .layout,
                &[BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            ),
            overlay_color_buffer: buffer,
            shader: shader_asset_server.load(static_hasher!("Image2DMaterialShader"), || {
                ShaderAsset {
                    shader: create_shader_module(
                        "Image2DMaterialShader",
                        include_str!("../../image2d.wgsl"),
                    ),
                }
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

    fn get_bind_group_layout_desc(
        asset_server: &Res<AssetServer<BindGroupLayoutAsset>>,
    ) -> Asset<BindGroupLayoutAsset> {
        asset_server.load(
            static_hasher!("Image2DMaterialBindGroupLayout"),
            move || BindGroupLayoutAsset {
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
            },
        )
    }
}

impl Material for Image2DMaterial {
    fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn get_shader(&self) -> &ShaderModule {
        &self.shader.get().shader
    }
}
