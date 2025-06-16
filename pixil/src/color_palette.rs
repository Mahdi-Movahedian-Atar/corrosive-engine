use std::sync::Mutex;
use corrosive_asset_manager_macro::{Asset, Cache};
use corrosive_ecs_renderer_backend::color::Color;
use corrosive_ecs_renderer_backend::public_functions::{create_sampler, create_texture, get_surface_format, write_texture};
use corrosive_ecs_renderer_backend::wgpu;
use corrosive_ecs_renderer_backend::wgpu::{Sampler, SamplerDescriptor, Texture, TextureView, TextureViewDescriptor};


#[derive(Default)]
pub enum ColorTransition {
    #[default]
    CutOff,
    Curve([f32; 4]),
}
#[derive(Default)]
pub struct ColorRange {
    pub size: u32,
    pub color: Color,
    pub transition_type: ColorTransition,
}
#[derive(Asset, Cache)]
pub struct ColorPallet {/*
    pub name: Option<&'static str>,
    pub colors: Vec<ColorRange>,
    pub len: usize,*/
    data: Mutex<[u8;256 * 256 * 4]>,
    pub(crate) texture: Texture,
    pub(crate) texture_view: TextureView,
    pub(crate) texture_sampler: Sampler,
}

impl ColorPallet {
    pub fn new() -> Self {
        let data = [0u8;256 * 256 * 4];
        let texture = create_texture(&wgpu::TextureDescriptor {
            label: Some("PalletTexture"),
            size: wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        write_texture(
            texture.as_image_copy(),
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256 * 4),
                rows_per_image: Some(256),
            },
            wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
        );

        let texture_view = texture.create_view(&TextureViewDescriptor{
            label: Some("PalletTextureView"),
            ..Default::default()
        });

        let sampler = create_sampler(&SamplerDescriptor {
            label: Some("PalletTextureSampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        Self {
            data: Mutex::new(data),
            texture,
            texture_view,
            texture_sampler:sampler
        }
    }
    pub fn set_palette(&self, row: u8, colors: Vec<ColorRange>) {
        let mut data = self.data.lock().unwrap();
        let mut x: u32 = 0;
        for (seg_idx, seg) in colors.iter().enumerate() {
            let seg_start = x;
            let seg_end = (x + seg.size).min(256);
            let span = (seg_end - seg_start).max(1);
            for i in 0..span {
                let t = i as f32 / (span - 1) as f32;
                let col = if seg_idx == 0 {
                    seg.color
                } else {
                    let prev = colors[seg_idx - 1].color;
                    let weight = match seg.transition_type {
                        ColorTransition::CutOff => 1.0,
                        ColorTransition::Curve(v) => {
                            let u = 1.0 - t;
                             u * u * u * v[0]
                                + 3.0 * u * u * t * v[1]
                                + 3.0 * u * t * t * v[2]
                                + t * t * t * v[3]
                        }
                    };
                    prev.mix(&seg.color,weight)
                };
                let px = (seg_start + i) as usize;
                write_px(&mut *data, px, row as usize, col);
            }
            x = seg_end;
        }
        write_texture(
            self.texture.as_image_copy(),
            &*data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256 * 4),
                rows_per_image: Some(256),
            },
            wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
        );
    }

}

fn write_px(data: &mut[u8], x: usize, y: usize, c: Color) {
    let idx = (y * 256 + x) * 4;
    let c = c.to_array_u8();
    data[idx + 0] = c[0];
    data[idx + 1] = c[1];
    data[idx + 2] = c[2];
    data[idx + 3] = c[3];
}

/*fn get_colors(color_ranges: &Vec<ColorRange>, len: &usize) -> Vec<u8> {
    let mut colors = Vec::with_capacity(*len);
    for color in 0..color_ranges.len() {
        for i in 0..color_ranges[color].size {
            match color_ranges[color].transition_type {
                ColorTransition::CutOff => colors.extend(color_ranges[color].color.to_array_u8()),
                ColorTransition::Curve(v) => {
                    if i != color_ranges[color].size - 1 {
                        let t = i as f32 / (color_ranges[color].size - 1) as f32;
                        let u = 1.0 - t;
                        let v = u * u * u * v[0]
                            + 3.0 * u * u * t * v[1]
                            + 3.0 * u * t * t * v[2]
                            + t * t * t * v[3];
                        let color_a = color_ranges[color].color.to_array();
                        let color_b = color_ranges[color + 1].color.to_array();
                        colors.extend([
                            ((color_a[0] * (1.0 - v) + color_b[0] * v) * 255.0) as u8,
                            ((color_a[1] * (1.0 - v) + color_b[1] * v) * 255.0) as u8,
                            ((color_a[2] * (1.0 - v) + color_b[2] * v) * 255.0) as u8,
                            ((color_a[3] * (1.0 - v) + color_b[3] * v) * 255.0) as u8,
                        ]);
                    }
                    colors.extend(color_ranges[color].color.to_array_u8())
                    //todo: better mixing
                }
            }
        }
    }
    colors
}*/
