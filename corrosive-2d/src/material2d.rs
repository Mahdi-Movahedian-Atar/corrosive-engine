use corrosive_asset_manager::asset_server::{Asset, AssetServer};
use corrosive_asset_manager_macro::{static_hasher, Asset};
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_renderer_backend::assets::BindGroupLayoutAsset;
use corrosive_ecs_renderer_backend::color::Color;
use corrosive_ecs_renderer_backend::helper::{
    create_bind_group, create_bind_group_layout, create_buffer_init, create_shader_module,
    get_queue, read_shader, BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferUsages,
    PipelineLayoutDescriptor, ShaderModule, ShaderStage, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexStepMode,
};
use corrosive_ecs_renderer_backend::material::{Material, MaterialDesc};

pub trait Material2DWrapper {
    fn get_bind_group(&self) -> &'static BindGroup;
}
pub trait Material2D: Material {
    fn generate_wrapper(&self, asset: Asset<Self>) -> Box<dyn Material2DWrapper + Send + Sync>
    where
        Self: Sized;
}

#[derive(Clone, Asset)]
pub struct StandardMaterial2D {
    pub overlay_color: Color,
    overlay_color_buffer: Buffer,
    bind_group: BindGroup,
}
impl StandardMaterial2D {
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
                &StandardMaterial2D::get_bind_group_layout_desc()
                    .get()
                    .layout,
                &[BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            ),
            overlay_color_buffer: buffer,
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

impl MaterialDesc for StandardMaterial2D {
    fn get_name_desc<'a>() -> &'a str {
        "Image2DMaterial"
    }

    fn get_bind_group_layout_desc<'a>() -> Asset<BindGroupLayoutAsset> {
        AssetServer::add(
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

impl Material for StandardMaterial2D {
    fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn get_shader(&self) -> (&str, String) {
        (
            "StandardMaterial2D",
            read_shader("corrosive-2d/shaders/image2d.wgsl").expect("failed to read shader"),
        )
    }

    fn get_name(&self) -> &str {
        StandardMaterial2D::get_name_desc()
    }

    fn get_bind_group_layout(&self) -> Asset<BindGroupLayoutAsset> {
        StandardMaterial2D::get_bind_group_layout_desc()
    }
}

struct StandardMaterial2DWrapper {
    asset: Asset<StandardMaterial2D>,
}
impl Material2DWrapper for StandardMaterial2DWrapper {
    fn get_bind_group(&self) -> &'static BindGroup {
        self.asset.get().get_bind_group()
    }
}
impl Material2D for StandardMaterial2D {
    fn generate_wrapper(&self, asset: Asset<Self>) -> Box<dyn Material2DWrapper + Send + Sync> {
        Box::new(StandardMaterial2DWrapper { asset })
    }
}
