use corrosive_ecs_core_macro::Resource;
use corrosive_ecs_renderer_backend::public_functions::{
    create_buffer_init, get_device, get_surface_format, get_window_ratio, write_to_buffer,
};
use corrosive_ecs_renderer_backend::wgpu::{
    Buffer, BufferUsages, Extent3d, RenderPipeline, Texture, TextureDescriptor, TextureDimension,
    TextureUsages, TextureView,
};

#[derive(Resource)]
pub struct PixilRenderSettings {
    pub(crate) render_size: u32,
    pub(crate) texture: Option<(Texture, TextureView)>,
    pub(crate) buffers: Option<(Buffer, Buffer)>,
    pub(crate) grid_size: [u32; 3],
}
impl Default for PixilRenderSettings {
    fn default() -> Self {
        Self {
            render_size: 320,
            texture: None,
            buffers: None,
            grid_size: [12u32, 12u32, 24u32],
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
                label: Some("Proxy Render Texture"),
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
    pub fn get_buffers(&mut self) -> &(Buffer, Buffer) {
        if self.buffers.is_none() {
            self.buffers = Some((
                create_buffer_init(
                    "ZParam",
                    bytemuck::cast_slice(&[0.1, 1.0]),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                ),
                create_buffer_init(
                    "ZParam",
                    bytemuck::cast_slice(&[
                        self.grid_size[0],
                        self.grid_size[1],
                        self.grid_size[2],
                    ]),
                    BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                ),
            ));
        }
        &self.buffers.as_ref().unwrap()
    }
    pub fn update_texture(&mut self) {
        if let Some((texture, _)) = &mut self.texture {
            println!(
                "{} {}",
                (self.render_size as f32 * get_window_ratio()) as u32,
                self.render_size
            );
            *texture = get_device().create_texture(&TextureDescriptor {
                label: Some("Proxy Render Texture"),
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
        }
    }
    pub fn update_z_params(&self, near: f32, far: f32) {
        if let Some(b) = &self.buffers {
            write_to_buffer(&b.0, 0, bytemuck::cast_slice(&[near, far]))
        }
    }
    pub fn update_grid_size(&mut self, x: u32, y: u32, z: u32) {
        self.grid_size = [x, y, z];
        if let Some(b) = &self.buffers {
            write_to_buffer(&b.1, 0, bytemuck::cast_slice(&[x, y, z]))
        }
    }
}
unsafe impl Send for PixilRenderSettings {}
unsafe impl Sync for PixilRenderSettings {}
