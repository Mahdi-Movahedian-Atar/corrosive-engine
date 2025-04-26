use crate::render_graph::Queue;
use crate::STATE;
use std::{env, fs, io};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    Buffer, BufferAddress, BufferUsages, Extent3d, PipelineLayout, PipelineLayoutDescriptor,
    RenderPipeline, RenderPipelineDescriptor, Sampler, SamplerDescriptor, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, TextureDescriptor, TextureFormat, VertexBufferLayout,
};
use wgpu::Device;

pub trait VertexRenderable {
    fn desc<'a>() -> VertexBufferLayout<'a>;
}
pub trait BindGroupRenderable {
    fn desc<'a>() -> BindGroupLayoutDescriptor<'a>;
}

pub fn create_shader_module(label: &str, source: &str) -> wgpu::ShaderModule {
    unsafe {
        if let Some(t) = &STATE {
            t.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
        } else {
            panic!("create_shader_module must be called after run_renderer task.")
        }
    }
}
pub fn create_pipeline(descriptor: &RenderPipelineDescriptor) -> RenderPipeline {
    unsafe {
        if let Some(t) = &STATE {
            t.device.create_render_pipeline(descriptor)
        } else {
            panic!("create_pipeline must be called after run_renderer task.")
        }
    }
}
pub fn create_pipeline_layout(descriptor: &PipelineLayoutDescriptor) -> PipelineLayout {
    unsafe {
        if let Some(t) = &STATE {
            t.device.create_pipeline_layout(descriptor)
        } else {
            panic!("create_pipeline_layout must be called after run_renderer task.")
        }
    }
}
pub fn create_bind_group_layout(descriptor: &BindGroupLayoutDescriptor) -> BindGroupLayout {
    unsafe {
        if let Some(t) = &STATE {
            t.device.create_bind_group_layout(descriptor)
        } else {
            panic!("create_bind_group_layout must be called after run_renderer task.")
        }
    }
}
pub fn create_buffer_init<'a>(label: &str, contents: &'a [u8], usage: BufferUsages) -> Buffer {
    unsafe {
        if let Some(t) = &STATE {
            t.device.create_buffer_init(&BufferInitDescriptor {
                label: label.into(),
                contents,
                usage,
            })
        } else {
            panic!("get_surface_format must be called after run_renderer task.")
        }
    }
}
pub fn create_bind_group<'a>(
    label: &str,
    layout: &'a BindGroupLayout,
    entries: &'a [BindGroupEntry<'a>],
) -> BindGroup {
    unsafe {
        if let Some(t) = &STATE {
            t.device.create_bind_group(&BindGroupDescriptor {
                label: label.into(),
                layout,
                entries,
            })
        } else {
            panic!("get_surface_format must be called after run_renderer task.")
        }
    }
}
pub fn get_surface_format() -> TextureFormat {
    unsafe {
        if let Some(t) = &STATE {
            t.config.read().unwrap().format
        } else {
            panic!("get_surface_format must be called after run_renderer task.")
        }
    }
}
pub fn get_window_ratio() -> f32 {
    unsafe {
        if let Some(t) = &STATE {
            t.config.read().unwrap().width as f32 / t.config.read().unwrap().height as f32
        } else {
            panic!("get_surface_format must be called after run_renderer task.")
        }
    }
}
pub fn get_window_resolution() -> (u32, u32) {
    unsafe {
        if let Some(t) = &STATE {
            (
                t.config.read().unwrap().width.clone(),
                t.config.read().unwrap().height.clone(),
            )
        } else {
            panic!("get_surface_format must be called after run_renderer task.")
        }
    }
}
pub fn get_resolution_bind_group<'a>() -> &'a BindGroup {
    unsafe {
        if let Some(t) = &STATE {
            &t.resolution_bind_group
        } else {
            panic!("get_surface_format must be called after run_renderer task.")
        }
    }
}
pub fn get_resolution_bind_group_layout<'a>() -> &'a BindGroupLayout {
    unsafe {
        if let Some(t) = &STATE {
            &t.resolution_bind_group_layout
        } else {
            panic!("get_surface_format must be called after run_renderer task.")
        }
    }
}
pub fn get_queue<'a>() -> &'a Queue {
    unsafe {
        if let Some(t) = &STATE {
            &t.queue
        } else {
            panic!("get_surface_format must be called after run_renderer task.")
        }
    }
}
pub fn get_device<'a>() -> &'a Device {
    unsafe {
        if let Some(t) = &STATE {
            &t.device
        } else {
            panic!("get_surface_format must be called after run_renderer task.")
        }
    }
}
pub fn write_to_buffer(buffer: &Buffer, offset: BufferAddress, data: &[u8]) {
    unsafe {
        if let Some(t) = &STATE {
            t.queue.write_buffer(buffer, offset, data)
        } else {
            panic!("write_buffer must be called after run_renderer task.")
        }
    }
}
pub fn read_shader(path: &str) -> io::Result<String> {
    if path.ends_with(".slang") {
        #[cfg(debug_assertions)]{
            return fs::read_to_string(format!("{}/{}", env::var("CORROSIVE_APP_ROOT").unwrap_or(".".to_string()),path).as_str());
        }
        fs::read_to_string(format!("./assets/{}.wgsl", path))
    } else {
        #[cfg(debug_assertions)]{
            return fs::read_to_string(format!("{}/assets/{}", env::var("CORROSIVE_APP_ROOT").unwrap_or(".".to_string()),path).as_str());
        }
        fs::read_to_string(format!("./assets/{}", path))
    }
}
pub fn create_texture(texture_descriptor: &TextureDescriptor) -> Texture {
    unsafe {
        if let Some(t) = &STATE {
            t.device.create_texture(texture_descriptor)
        } else {
            panic!("write_buffer must be called after run_renderer task.")
        }
    }
}
pub fn write_texture(
    texture: TexelCopyTextureInfo<'_>,
    data: &[u8],
    data_layout: TexelCopyBufferLayout,
    size: Extent3d,
) {
    unsafe {
        if let Some(t) = &STATE {
            t.queue.write_texture(texture, data, data_layout, size)
        } else {
            panic!("write_buffer must be called after run_renderer task.")
        }
    }
}
pub fn create_sampler(descriptor: &SamplerDescriptor) -> Sampler {
    unsafe {
        if let Some(t) = &STATE {
            t.device.create_sampler(descriptor)
        } else {
            panic!("write_buffer must be called after run_renderer task.")
        }
    }
}
