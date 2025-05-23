use corrosive_asset_manager::asset_server::AssetFile;
use corrosive_asset_manager_macro::Asset;
use corrosive_ecs_renderer_backend::public_functions::get_device;
use corrosive_ecs_renderer_backend::wgpu;
use corrosive_ecs_renderer_backend::wgpu::util::DeviceExt;
use std::error::Error;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Asset)]
pub struct Mesh {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) index_count: u32,
}
impl AssetFile for Mesh {
    fn load_file(file: &str) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        if file.ends_with(".obj") {
            let (models, _materials) = tobj::load_obj(
                file,
                &tobj::LoadOptions {
                    triangulate: true,
                    single_index: true,
                    ..Default::default()
                },
            )?;

            let mesh = &models[0].mesh;
            let mut vertices = Vec::new();

            for i in 0..(mesh.positions.len() / 3) {
                let pos = [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                ];

                let normal = if mesh.normals.is_empty() {
                    [0.0, 0.0, 0.0]
                } else {
                    [
                        mesh.normals[i * 3],
                        mesh.normals[i * 3 + 1],
                        mesh.normals[i * 3 + 2],
                    ]
                };

                vertices.push(Vertex {
                    position: pos,
                    normal,
                });
            }

            let device = get_device();

            Ok(Mesh {
                vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                }),
                index_count: mesh.indices.len() as u32,
            })
        } else {
            Err(Box::from("Not a mesh asset"))
        }
    }
}
