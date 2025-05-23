use corrosive_ecs_core_macro::Resource;
use corrosive_ecs_renderer_backend::public_functions::{
    get_device, get_surface_format, get_window_ratio,
};
use corrosive_ecs_renderer_backend::wgpu::{
    Extent3d, RenderPipeline, Texture, TextureDescriptor, TextureDimension, TextureUsages,
    TextureView,
};
use std::cell::LazyCell;
use std::time::Instant;

#[derive(Resource)]
pub struct PixilRenderSettings {
    pub(crate) render_size: u32,
    pub(crate) texture: Option<(Texture, TextureView)>,
}
impl Default for PixilRenderSettings {
    fn default() -> Self {
        Self {
            render_size: 320,
            texture: None,
        }
    }
}
impl PixilRenderSettings {
    pub fn set_new_render_size(&mut self, render_size: u32) {
        self.render_size = render_size;
        self.texture = None;
    }
    pub fn get_view(&mut self) -> &TextureView {
        if self.texture.is_none() {
            let texture = get_device().create_texture(&TextureDescriptor {
                label: Some("Proxy Render Texture (Resized)"),
                size: Extent3d {
                    width: (self.render_size as f32 / get_window_ratio()) as u32,
                    height: self.render_size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: get_surface_format(),
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&Default::default());
            self.texture = Some((texture, view));
        }
        &self.texture.as_ref().unwrap().1
    }
}
unsafe impl Send for PixilRenderSettings {}
unsafe impl Sync for PixilRenderSettings {}
