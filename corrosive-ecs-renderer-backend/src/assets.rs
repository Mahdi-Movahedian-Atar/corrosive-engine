use crate::public_functions::{create_texture, write_texture};
use corrosive_asset_manager;
use corrosive_asset_manager::asset_server::AssetFile;
use corrosive_asset_manager_macro::{Asset, Cache};
use image::ImageReader;
use std::error::Error;
use std::fs::File;
use wgpu::{RenderPipeline, TexelCopyTextureInfo, Texture};

#[derive(PartialEq, Asset , Cache)]
pub struct PipelineAsset {
    pub layout: RenderPipeline,
}
#[derive(PartialEq, Asset , Cache)]
pub struct BindGroupLayoutAsset {
    pub layout: wgpu::BindGroupLayout,
}
#[derive(PartialEq, Asset)]
pub struct TextureAsset {
    pub texture: Texture,
}
impl AssetFile for TextureAsset {
    fn load_file(file_name: &str) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let file = File::open(file_name)?;
        let reader = ImageReader::new(std::io::BufReader::new(file)).with_guessed_format()?;

        let diffuse_image = reader.decode()?;
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = create_texture(&wgpu::TextureDescriptor {
            label: file_name.into(),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        Ok(TextureAsset { texture })
    }
}
