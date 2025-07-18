use corrosive_ecs_core_macro::Resource;
use corrosive_ecs_renderer_backend::public_functions::{
    create_buffer_init, get_device, get_surface_format, get_window_ratio, write_to_buffer,
};
use corrosive_ecs_renderer_backend::wgpu::{
    Buffer, BufferUsages, Extent3d, RenderPipeline, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView,
};
use std::cell::LazyCell;

pub(crate) struct PixilRenderSettingsTextures {
    pub(crate) render_texture: Texture,
    pub(crate) depth_texture: Texture,
    pub(crate) render_texture_view: TextureView,
    pub(crate) depth_texture_view: TextureView,
}
#[derive(Resource)]
pub struct PixilRenderSettings {
    pub(crate) render_size: u32,
    pub(crate) texture: Option<PixilRenderSettingsTextures>,
    pub(crate) size_buffer: LazyCell<Buffer>,
    pub(crate) grid_size_buffer: LazyCell<Buffer>,
    pub(crate) grid_size: [u32; 3],
}
impl Default for PixilRenderSettings {
    fn default() -> Self {
        Self {
            render_size: 320,
            texture: None,
            size_buffer: LazyCell::new(|| {
                create_buffer_init(
                    "SizeBuffer",
                    bytemuck::cast_slice(&[(320.0 * get_window_ratio()) as u32, 320u32]),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                )
            }),
            grid_size_buffer: LazyCell::new(|| {
                create_buffer_init(
                    "GridParams",
                    bytemuck::cast_slice(&[12u32, 12u32, 24u32]),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                )
            }),
            grid_size: [12u32, 12u32, 24u32],
        }
    }
}
impl PixilRenderSettings {
    pub fn set_new_render_size(&mut self, render_size: u32) {
        self.render_size = render_size;
        write_to_buffer(
            &self.size_buffer,
            0,
            bytemuck::cast_slice(&[
                (self.render_size as f32 * get_window_ratio()) as u32,
                self.render_size,
            ]),
        );
        self.texture = None;
    }
    pub fn set_view(&mut self) {
        if self.texture.is_none() {
            let render_texture = get_device().create_texture(&TextureDescriptor {
                label: Some("Pixil Render Texture"),
                size: Extent3d {
                    width: (self.render_size as f32 * get_window_ratio()) as u32,
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
            let depth_texture = get_device().create_texture(&TextureDescriptor {
                label: Some("Pixil Depth Texture"),
                size: Extent3d {
                    width: (self.render_size as f32 * get_window_ratio()) as u32,
                    height: self.render_size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let render_texture_view = render_texture.create_view(&Default::default());
            let depth_texture_view = depth_texture.create_view(&Default::default());
            self.texture = Some(PixilRenderSettingsTextures {
                render_texture,
                depth_texture,
                render_texture_view,
                depth_texture_view,
            });
        }
    }
    pub fn get_render_view(&self) -> &TextureView {
        &self.texture.as_ref().unwrap().render_texture_view
    }
    pub fn get_depth_view(&self) -> &TextureView {
        &self.texture.as_ref().unwrap().depth_texture_view
    }
    pub fn update_texture(&mut self) {
        if let Some(PixilRenderSettingsTextures {
            render_texture,
            depth_texture,
            ..
        }) = &mut self.texture
        {
            *render_texture = get_device().create_texture(&TextureDescriptor {
                label: Some("Pixil Render Texture"),
                size: Extent3d {
                    width: (self.render_size as f32 * get_window_ratio()) as u32,
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
            *depth_texture = get_device().create_texture(&TextureDescriptor {
                label: Some("pixil Depth Texture"),
                size: Extent3d {
                    width: (self.render_size as f32 * get_window_ratio()) as u32,
                    height: self.render_size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
        }
    }
    pub fn update_grid_size(&mut self, x: u32, y: u32, z: u32) {
        self.grid_size = [x, y, z];
        write_to_buffer(&self.grid_size_buffer, 0, bytemuck::cast_slice(&[x, y, z]))
    }
}
unsafe impl Send for PixilRenderSettings {}
unsafe impl Sync for PixilRenderSettings {}
