use crate::comp::Mesh2D;
use crate::mesh2d::Vertex2D;
use corrosive_asset_manager::asset_server::{Asset, AssetServer};
use corrosive_asset_manager_macro::static_hasher;
use corrosive_ecs_core::trait_for;
use corrosive_ecs_core_macro::Component;
use corrosive_ecs_renderer_backend::assets::{BindGroupLayoutAsset, TextureAsset};
use corrosive_ecs_renderer_backend::helper::{
    create_bind_group, create_bind_group_layout, create_buffer_init, create_sampler, AddressMode,
    BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
    BindingType, Buffer, BufferUsages, FilterMode, RenderPass, SamplerBindingType,
    SamplerDescriptor, ShaderStages, TextureSampleType, TextureViewDescriptor,
    TextureViewDimension,
};

#[derive(Component)]
pub struct Sprite2D {
    offset: [f32; 2],
    texture: Asset<TextureAsset>,
    bind_group_layout_asset: Asset<BindGroupLayoutAsset>,
    bind_group: BindGroup,
    vertex_buffer: Buffer,
}
trait_for!(trait Mesh2D => Sprite2D);
impl Sprite2D {
    pub fn new(texture: Asset<TextureAsset>, offset: [f32; 2]) -> Self {
        let diffuse_texture_view = texture
            .get()
            .texture
            .create_view(&TextureViewDescriptor::default());
        let diffuse_sampler = create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group_layout_asset = AssetServer::add_sync(static_hasher!("Rect2D"), || {
            Ok(BindGroupLayoutAsset {
                layout: create_bind_group_layout(&BindGroupLayoutDescriptor {
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                view_dimension: TextureViewDimension::D2,
                                sample_type: TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("rect_2d_bind_group_layout"),
                }),
            })
        });
        Sprite2D {
            offset,
            texture,
            bind_group_layout_asset: bind_group_layout_asset.clone(),
            bind_group: create_bind_group(
                "rect_2d_bind_group",
                &bind_group_layout_asset.get().layout,
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&diffuse_texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&diffuse_sampler),
                    },
                ],
            ),
            vertex_buffer: create_buffer_init(
                "rect_2d_vertex_buffer",
                bytemuck::cast_slice(&[
                    Vertex2D {
                        position: [0f32, 1f32, 0f32],
                        uv: [0f32, 1f32],
                    },
                    Vertex2D {
                        position: [1f32, 1f32, 0f32],
                        uv: [1f32, 1f32],
                    },
                    Vertex2D {
                        position: [0f32, 0f32, 0f32],
                        uv: [0f32, 0f32],
                    },
                    Vertex2D {
                        position: [1f32, 0f32, 0f32],
                        uv: [1f32, 0f32],
                    },
                ]),
                BufferUsages::VERTEX,
            ),
        }
    }
}
impl Mesh2D for Sprite2D {
    fn draw(&self, render_pass: &mut RenderPass) {
        render_pass.set_bind_group(2, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..4, 0..1)
    }

    fn update(&self, render_pass: &mut RenderPass) {
        render_pass.set_bind_group(0, &self.bind_group, &[]);
    }

    fn name<'a>(&self) -> &'a str {
        "Rect2D"
    }

    fn get_bind_group_layout_desc(&self) -> &Asset<BindGroupLayoutAsset> {
        &self.bind_group_layout_asset
    }
}
