use crate::comp::light::directional_light::DirectionalLightData;
use crate::comp::light::point_light::PointLightData;
use crate::comp::light::spot_light::SpotLightData;
use corrosive_asset_manager::cache_server::CacheServer;
use corrosive_ecs_renderer_backend::assets::BindGroupLayoutAsset;
use corrosive_ecs_renderer_backend::public_functions::{create_bind_group, create_bind_group_layout, create_buffer_init, create_pipeline, create_pipeline_layout, create_sampler, create_shader_module, create_texture, get_device, read_shader, write_to_buffer};
use corrosive_ecs_renderer_backend::wgpu;
use corrosive_ecs_renderer_backend::wgpu::{BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor, BufferUsages, Extent3d, PipelineLayoutDescriptor, RenderPipeline, Sampler, ShaderStages, Texture, TextureView, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};
use std::cmp::max;
use std::sync::Mutex;
use glam::{vec2, vec4, Mat4, Vec3, Vec4, Vec4Swizzles};
use corrosive_ecs_renderer_backend::wgpu::hal::dx12::PipelineLayout;
use crate::helper_functions::{transform_bind_group_layout, VERTEX_BUFFER_LAYOUT};
use crate::mesh::Vertex;

const POINT_LIGHT_SIZE: u32 = 8u32;
const SPOT_LIGHT_SIZE: u32 = 8u32;
const DIRECTIONAL_LIGHT_SIZE: u32 = 2u32;
const AMBIENT_LIGHT_SIZE: u32 = 2u32;
const CASCADE_SPLITS: [f32; 4] = [0.0,0.05, 0.2, 1.0];

struct DirectionalLightShadowMapData{
    inv_projection:[f32;16],
    is_enables:f32,
    padding:f32,
}

pub(crate) struct DynamicLightSet {
    point_light_available_ids: Vec<u32>,
    point_light_len: u32,
    point_light_data: Buffer,
    point_light_len_buffer: Buffer,
    spot_light_available_ids: Vec<u32>,
    spot_light_len: u32,
    spot_light_data: Buffer,
    spot_light_len_buffer: Buffer,
    directional_light_available_ids: Vec<u32>,
    directional_lights: Vec<DirectionalLightData>,
    directional_light_len: u32,
    directional_light_data: Buffer,
    directional_light_len_buffer: Buffer,
    pub(crate)directional_light_shadow_map_textures: [Texture;3],
    pub(crate)directional_light_shadow_map_whole_texture_views: [TextureView;3],
    directional_light_shadow_map_whole_sampler: Sampler,
    pub(crate) directional_light_shadow_map_texture_views: Vec<([TextureView;3])>,
    pub(crate)directional_light_shadow_map_enabled: Vec<(bool)>,
    pub(crate)directional_light_shadow_map_pipeline_layout: RenderPipeline,
    directional_light_shadow_map_bind_group_layout: BindGroupLayout,
    directional_light_shadow_map_buffers: Vec<[Buffer;3]>,
    pub(crate)directional_light_shadow_map_bind_groups: Vec<[BindGroup;3]>,
    frustum_corners: [[Vec3; 8];3],
    pub(crate) shadows_texture_size: u32,
    pub(crate) bind_group_compute: BindGroup,
    pub(crate) bind_group_fragment: BindGroup,
    pub(crate) bind_group_compute_layout: BindGroupLayout,
    pub(crate) bind_group_fragment_layout: BindGroupLayout,
}
impl DynamicLightSet {
    pub fn allocate_point_light(&mut self, size: u32) {
        let mut size = max((size / POINT_LIGHT_SIZE + 1) * POINT_LIGHT_SIZE, 1);
        for i in (self.point_light_len..size).rev() {
            self.point_light_available_ids.push(i);
        }
        let new_buffer = get_device().create_buffer(&BufferDescriptor {
            label: "PointLightBuffer".into(),
            size: (size * size_of::<PointLightData>() as u32) as BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        write_to_buffer(
            &new_buffer,
            0,
            self.point_light_data
                .slice(..)
                .get_mapped_range()
                .to_vec()
                .as_slice(),
        );
        self.point_light_data = new_buffer;
        self.recreate_bind_groups();
    }
    pub fn add_point_light(&mut self, data: &PointLightData) -> u32 {
        if let Some(id) = self.point_light_available_ids.pop() {
            write_to_buffer(
                &self.point_light_data,
                (id * (size_of::<PointLightData>() as u32)) as BufferAddress,
                bytemuck::bytes_of(data),
            );
            if self.point_light_len <= id {
                self.point_light_len = id + 1;
                write_to_buffer(
                    &self.point_light_len_buffer,
                    0,
                    bytemuck::bytes_of(&self.point_light_len),
                );
            }
            id
        } else {
            self.allocate_point_light(self.point_light_len + 1);
            self.add_point_light(data)
        }
    }
    pub fn update_point_light(&mut self, data: &PointLightData, id: u32) {
        write_to_buffer(
            &self.point_light_data,
            (id * (size_of::<PointLightData>() as u32)) as BufferAddress,
            bytemuck::bytes_of(data),
        );
    }
    pub fn remove_point_light(&mut self, id: u32) {
        write_to_buffer(
            &self.point_light_data,
            (id * size_of::<PointLightData>() as u32) as BufferAddress,
            vec![0u8; size_of::<PointLightData>()].as_slice(),
        );
        if self.point_light_len - 1 == id {
            self.point_light_len = id;
            write_to_buffer(
                &self.point_light_len_buffer,
                0,
                bytemuck::bytes_of(&self.point_light_len),
            );
        }
        self.point_light_available_ids.push(id);
    }
    pub fn allocate_spot_light(&mut self, size: u32) {
        let mut size = max((size / SPOT_LIGHT_SIZE + 1) * SPOT_LIGHT_SIZE, 1);
        for i in (self.spot_light_len..size).rev() {
            self.spot_light_available_ids.push(i);
        }
        let new_buffer = get_device().create_buffer(&BufferDescriptor {
            label: "SpotLightBuffer".into(),
            size: (size * size_of::<SpotLightData>() as u32) as BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        write_to_buffer(
            &new_buffer,
            0,
            self.spot_light_data
                .slice(..)
                .get_mapped_range()
                .to_vec()
                .as_slice(),
        );
        self.spot_light_data = new_buffer;
        self.recreate_bind_groups();
    }
    pub fn add_spot_light(&mut self, data: &SpotLightData) -> u32 {
        if let Some(id) = self.spot_light_available_ids.pop() {
            write_to_buffer(
                &self.spot_light_data,
                (id * (size_of::<SpotLightData>() as u32)) as BufferAddress,
                bytemuck::bytes_of(data),
            );
            if self.spot_light_len <= id {
                self.spot_light_len = id + 1;
                write_to_buffer(
                    &self.spot_light_len_buffer,
                    0,
                    bytemuck::bytes_of(&self.spot_light_len),
                );
            }
            id
        } else {
            self.allocate_point_light(self.spot_light_len + 1);
            self.add_spot_light(data)
        }
    }
    pub fn update_spot_light(&mut self, data: &SpotLightData, id: u32) {
        write_to_buffer(
            &self.spot_light_data,
            (id * (size_of::<SpotLightData>() as u32)) as BufferAddress,
            bytemuck::bytes_of(data),
        );
    }
    pub fn remove_spot_light(&mut self, id: u32) {
        write_to_buffer(
            &self.spot_light_data,
            (id * size_of::<SpotLightData>() as u32) as BufferAddress,
            vec![0u8; size_of::<SpotLightData>()].as_slice(),
        );
        if self.spot_light_len - 1 == id {
            self.spot_light_len = id;
            write_to_buffer(
                &self.spot_light_len_buffer,
                0,
                bytemuck::bytes_of(&self.spot_light_len),
            );
        }
        self.spot_light_available_ids.push(id);
    }
    pub fn allocate_directional_light(&mut self, size: u32) {
        let mut size = max(
            (size / DIRECTIONAL_LIGHT_SIZE + 1) * DIRECTIONAL_LIGHT_SIZE,
            1,
        );
        for i in (self.directional_light_len..size).rev() {
            self.directional_light_available_ids.push(i);
            self.directional_lights.push(DirectionalLightData::default())
        }
        let new_buffer = get_device().create_buffer(&BufferDescriptor {
            label: "DirectionalLightBuffer".into(),
            size: (size * size_of::<DirectionalLightData>() as u32) as BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        write_to_buffer(
            &new_buffer,
            0,
            self.directional_light_data
                .slice(..)
                .get_mapped_range()
                .to_vec()
                .as_slice(),
        );
        self.directional_light_data = new_buffer;
        self.recreate_directional_textures();
        self.recreate_bind_groups();
    }
    pub fn add_directional_light(&mut self, data: &DirectionalLightData) -> u32 {
        if let Some(id) = self.directional_light_available_ids.pop() {
            write_to_buffer(
                &self.directional_light_data,
                (id * (size_of::<DirectionalLightData>() as u32)) as BufferAddress,
                bytemuck::bytes_of(data),
            );
            if self.directional_light_len <= id {
                self.directional_light_len = id + 1;
                write_to_buffer(
                    &self.directional_light_len_buffer,
                    0,
                    bytemuck::bytes_of(&self.directional_light_len),
                );
            }
            if data.cast_shadow_mask != 0{
                self.directional_light_shadow_map_enabled[id as usize] = true;
            }
            self.directional_lights[id as usize] = data.clone();
            if self.directional_light_shadow_map_enabled[id as usize]{
                self.update_directional_cast_shadow_buffer(id);
            }
            id
        } else {
            self.allocate_directional_light(self.directional_light_len + 1);
            self.add_directional_light(data)
        }
    }
    pub fn update_directional_light(&mut self, data: &DirectionalLightData, id: u32) {
        if self.directional_light_shadow_map_enabled[id as usize]{
            self.update_directional_cast_shadow_buffer(id);
        }
        self.directional_lights[id as usize] = data.clone();
        write_to_buffer(
            &self.directional_light_data,
            (id * (size_of::<DirectionalLightData>() as u32)) as BufferAddress,
            bytemuck::bytes_of(&self.directional_lights[id as usize] ),
        );
        if data.cast_shadow_mask != 0{
            self.directional_light_shadow_map_enabled[id as usize] = true;
        }
        else { self.directional_light_shadow_map_enabled[id as usize] = false }
        self.directional_lights[id as usize] = data.clone();
    }
    pub fn remove_directional_light(&mut self, id: u32) {
        write_to_buffer(
            &self.directional_light_data,
            (id * size_of::<DirectionalLightData>() as u32) as BufferAddress,
            vec![0u8; size_of::<DirectionalLightData>()].as_slice(),
        );
        if self.directional_light_len - 1 == id {
            self.directional_light_len = id;
            write_to_buffer(
                &self.directional_light_len_buffer,
                0,
                bytemuck::bytes_of(&self.directional_light_len),
            );
        }
        self.directional_light_available_ids.push(id);
        self.directional_light_shadow_map_enabled[id as usize] = false
    }
    pub fn new(texture_size: u32) -> Self {
        //point_light
        let point_light_vec: Vec<u32> = (0..POINT_LIGHT_SIZE).rev().collect();
        let point_light_buffer = get_device().create_buffer(&BufferDescriptor {
            label: "PointLightBuffer".into(),
            size: (POINT_LIGHT_SIZE * size_of::<PointLightData>() as u32) as BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let point_light_len_buffer = create_buffer_init(
            "PointLightLen",
            bytemuck::bytes_of(&POINT_LIGHT_SIZE),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        //spot_light
        let spot_light_vec: Vec<u32> = (0..SPOT_LIGHT_SIZE).rev().collect();
        let spot_light_buffer = get_device().create_buffer(&BufferDescriptor {
            label: "SpotLightBuffer".into(),
            size: (SPOT_LIGHT_SIZE * size_of::<SpotLightData>() as u32) as BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let spot_light_len_buffer = create_buffer_init(
            "SpotLightLen",
            bytemuck::bytes_of(&SPOT_LIGHT_SIZE),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        //directional_light
        let directional_light_vec: Vec<u32> = (0..DIRECTIONAL_LIGHT_SIZE).rev().collect();
        let directional_light_buffer = get_device().create_buffer(&BufferDescriptor {
            label: "DirectionalLightBuffer".into(),
            size: (DIRECTIONAL_LIGHT_SIZE * size_of::<DirectionalLightData>() as u32)
                as BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let directional_light_len_buffer = create_buffer_init(
            "DirectionalLightLen",
            bytemuck::bytes_of(&DIRECTIONAL_LIGHT_SIZE),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let directional_light_shadow_map_textures = [
            create_texture(&wgpu::TextureDescriptor {
                label: Some("DirectionalTextureArray_1"),
                size: Extent3d {
                    width: texture_size,
                    height: texture_size,
                    depth_or_array_layers: DIRECTIONAL_LIGHT_SIZE,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
            create_texture(&wgpu::TextureDescriptor {
                label: Some("DirectionalTextureArray_2"),
                size: Extent3d {
                    width: texture_size,
                    height: texture_size,
                    depth_or_array_layers: DIRECTIONAL_LIGHT_SIZE,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
            create_texture(&wgpu::TextureDescriptor {
                label: Some("DirectionalTextureArray_3"),
                size: Extent3d {
                    width: texture_size,
                    height: texture_size,
                    depth_or_array_layers: DIRECTIONAL_LIGHT_SIZE,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
        ];
        let directional_light_shadow_map_whole_texture_views = [
            directional_light_shadow_map_textures
                [0]
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("DirectionalLightTextureArray_0"),
                    format: Some(wgpu::TextureFormat::Depth32Float),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                    aspect: wgpu::TextureAspect::DepthOnly,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                }),
            directional_light_shadow_map_textures
                [1]
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("DirectionalLightTextureArray_1"),
                    format: Some(wgpu::TextureFormat::Depth32Float),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                    aspect: wgpu::TextureAspect::DepthOnly,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                }),
            directional_light_shadow_map_textures
                [2]
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("DirectionalLightTextureArray_2"),
                    format: Some(wgpu::TextureFormat::Depth32Float),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                    aspect: wgpu::TextureAspect::DepthOnly,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                }),
        ];
        let directional_light_shadow_map_whole_sampler=
            create_sampler(&wgpu::SamplerDescriptor {
                label: Some("ShadowMapSampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                anisotropy_clamp: 1,
                border_color: None,
            });
        let directional_light_shadow_map_bind_group_layout = create_bind_group_layout(&BindGroupLayoutDescriptor{
            label: Some("DirectionalLightShadowMapBindGroupDescriptor"),
            entries: &[BindGroupLayoutEntry{
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let mut directional_lights = Vec::with_capacity(DIRECTIONAL_LIGHT_SIZE as usize);
        let mut directional_light_shadow_map_texture_views = Vec::with_capacity(DIRECTIONAL_LIGHT_SIZE as usize);
        let mut directional_light_shadow_map_buffers = Vec::with_capacity(DIRECTIONAL_LIGHT_SIZE as usize);
        let mut directional_light_shadow_map_bind_groups = Vec::with_capacity(DIRECTIONAL_LIGHT_SIZE as usize);
        for i in 0..DIRECTIONAL_LIGHT_SIZE {
            directional_lights.push(DirectionalLightData::default());
            directional_light_shadow_map_texture_views.push(
                [directional_light_shadow_map_textures[0].create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!("DirectionalLightRenderView_0_{}", i)),
                    format: Some(wgpu::TextureFormat::Depth32Float),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: i,
                    array_layer_count: Some(1),
                    ..Default::default()
                }),
                    directional_light_shadow_map_textures[1].create_view(&wgpu::TextureViewDescriptor {
                        label: Some(&format!("DirectionalLightRenderView_1_{}", i)),
                        format: Some(wgpu::TextureFormat::Depth32Float),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        base_array_layer: i,
                        array_layer_count: Some(1),
                        ..Default::default()
                    }),
                    directional_light_shadow_map_textures[2].create_view(&wgpu::TextureViewDescriptor {
                        label: Some(&format!("DirectionalLightRenderView_2_{}", i)),
                        format: Some(wgpu::TextureFormat::Depth32Float),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        base_array_layer: i,
                        array_layer_count: Some(1),
                        ..Default::default()
                    }),] );
            directional_light_shadow_map_buffers.push([
                create_buffer_init(&format!("DirectionalLightShadowMapBuffer_{}", i),bytemuck::bytes_of(&Mat4::IDENTITY.to_cols_array()),BufferUsages::UNIFORM| BufferUsages::COPY_DST),
                create_buffer_init(&format!("DirectionalLightShadowMapBuffer_{}", i),bytemuck::bytes_of(&Mat4::IDENTITY.to_cols_array()),BufferUsages::UNIFORM| BufferUsages::COPY_DST),
                create_buffer_init(&format!("DirectionalLightShadowMapBuffer_{}", i),bytemuck::bytes_of(&Mat4::IDENTITY.to_cols_array()),BufferUsages::UNIFORM| BufferUsages::COPY_DST),]
            );
            directional_light_shadow_map_bind_groups.push([create_bind_group(&format!("DirectionalLightShadowMapBindGroup_0_{}", i),&directional_light_shadow_map_bind_group_layout,&[BindGroupEntry{
                binding: 0,
                resource: directional_light_shadow_map_buffers.last().unwrap()[0].as_entire_binding(),
            }]),create_bind_group(&format!("DirectionalLightShadowMapBindGroup_1_{}", i),&directional_light_shadow_map_bind_group_layout,&[BindGroupEntry{
                binding: 0,
                resource: directional_light_shadow_map_buffers.last().unwrap()[1].as_entire_binding(),
            }]),create_bind_group(&format!("DirectionalLightShadowMapBindGroup_2_{}", i),&directional_light_shadow_map_bind_group_layout,&[BindGroupEntry{
                binding: 0,
                resource: directional_light_shadow_map_buffers.last().unwrap()[2].as_entire_binding(),
            }])])
        }
        //bind_groups
        let bind_group_compute_layout =
            get_device().create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: "LightSetComputeBindGroupLayout".into(),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let bind_group_fragment_layout =
            get_device().create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: "LightSetFragmentBindGroupLayout".into(),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 5,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 9,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            });
        let bind_group_compute = create_bind_group(
            "LightSetComputeBindGroup",
            &bind_group_compute_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: point_light_len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: point_light_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: spot_light_len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: spot_light_buffer.as_entire_binding(),
                },
            ],
        );
        let bind_group_fragment = create_bind_group(
            "OrderedSetFragmentBindGroup",
            &bind_group_fragment_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: point_light_len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: point_light_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: spot_light_len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: spot_light_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: directional_light_len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: directional_light_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&directional_light_shadow_map_whole_texture_views[0]),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&directional_light_shadow_map_whole_texture_views[1]),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&directional_light_shadow_map_whole_texture_views[2]),
                },
                BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::Sampler(&directional_light_shadow_map_whole_sampler),
                },
            ],
        );
        Self {
            point_light_available_ids: point_light_vec,
            point_light_len: 0,
            point_light_data: point_light_buffer,
            point_light_len_buffer,
            spot_light_available_ids: spot_light_vec,
            spot_light_len: 0,
            spot_light_data: spot_light_buffer,
            spot_light_len_buffer,
            directional_light_available_ids: directional_light_vec,
            directional_lights,
            directional_light_len: 0,
            directional_light_data: directional_light_buffer,
            directional_light_len_buffer,
            directional_light_shadow_map_textures,
            directional_light_shadow_map_whole_texture_views,
            directional_light_shadow_map_whole_sampler,
            directional_light_shadow_map_texture_views,
            directional_light_shadow_map_enabled: Vec::from([false; DIRECTIONAL_LIGHT_SIZE as usize]) ,
            directional_light_shadow_map_pipeline_layout: create_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("DirectionalShadowMapRenderPipeline"),
                layout: Some(&create_pipeline_layout(
                    &PipelineLayoutDescriptor {
                        label: Some("ShadowRenderPipelineLayout"),
                        bind_group_layouts: &[&directional_light_shadow_map_bind_group_layout,&transform_bind_group_layout().get().layout],
                        push_constant_ranges: &[],
                    }
                )),
                vertex: wgpu::VertexState {
                    module: &create_shader_module("directional_shadow_map.wgsl",&read_shader("packages/pixil/shaders/directional_shadow_map.wgsl").unwrap()),
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &VERTEX_BUFFER_LAYOUT,
                },
                fragment: None, // No fragment shader needed for depth-only pass
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // Match your scene's winding order
                    cull_mode: Some(wgpu::Face::Back), // Cull back faces for shadow maps
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            }),
            directional_light_shadow_map_bind_group_layout,
            directional_light_shadow_map_buffers,
            directional_light_shadow_map_bind_groups,
            frustum_corners: [[Vec3::ONE;8];3],
            shadows_texture_size: 0,
            bind_group_compute,
            bind_group_compute_layout,
            bind_group_fragment,
            bind_group_fragment_layout,
        }
    }
    pub fn set_frustum_corners_world(
        &mut self,
        inv_view_proj: Mat4,
        proj: Mat4,
        near: f32,
        far: f32,
    ) {
        fn project_view_z_to_clip_z(view_z: f32, proj: &Mat4) -> f32 {
            let v = *proj * vec4(0.0, 0.0, view_z, 1.0);
            v.z / v.w
        }

        let splits = CASCADE_SPLITS.map(|s| near + (far - near) * s);

        for i in 0..CASCADE_SPLITS.len() - 1 {
            let near = splits[i];
            let far = splits[i + 1];
            let near_clip_z = project_view_z_to_clip_z(-near,&proj);
            let far_clip_z = project_view_z_to_clip_z(-far,&proj);

            let clip_space_corners = [
                // Near plane
                Vec4::new(-1.0,  1.0, near_clip_z, 1.0), // Top-left near
                Vec4::new( 1.0,  1.0, near_clip_z, 1.0), // Top-right near
                Vec4::new( 1.0, -1.0, near_clip_z, 1.0), // Bottom-right near
                Vec4::new(-1.0, -1.0, near_clip_z, 1.0), // Bottom-left near
                // Far plane
                Vec4::new(-1.0,  1.0, far_clip_z, 1.0),  // Top-left far
                Vec4::new( 1.0,  1.0, far_clip_z, 1.0),  // Top-right far
                Vec4::new( 1.0, -1.0, far_clip_z, 1.0),  // Bottom-right far
                Vec4::new(-1.0, -1.0, far_clip_z, 1.0),  // Bottom-left far
            ];
            let mut world_corners = [Vec3::ZERO; 8];
            for (i, corner) in clip_space_corners.iter().enumerate() {
                let world_pos = inv_view_proj * *corner;
                world_corners[i] = world_pos.truncate() / world_pos.w;
            }

            self.frustum_corners[i] = world_corners;
        }

        for i in 0..self.directional_light_shadow_map_enabled.len() {
            if self.directional_light_shadow_map_enabled[i] {
                self.update_directional_cast_shadow_buffer(i as u32);
                write_to_buffer(
                    &self.directional_light_data,
                    (i * std::mem::size_of::<DirectionalLightData>()) as wgpu::BufferAddress,
                    bytemuck::bytes_of(&self.directional_lights[i]),
                );
            }
        }
    }
    pub fn update_directional_cast_shadow_buffer(&mut self,id:u32){
        let direction = self.directional_lights[id as usize].direction;
        let light_dir = -Vec3::new(direction[0],direction[1],direction[2]).normalize();

        for i in 0..CASCADE_SPLITS.len() -1 {
            let frustum_corners = self.frustum_corners[i];

            let center = frustum_corners.iter().copied().sum::<Vec3>() / 8.0;

            let mut radius:f32 = 0.0;
            for c in &frustum_corners {
                let r = (*c - center).length();
                radius = radius.max(r);
            }

            let distance = radius *  1.1;

            let light_view = Mat4::look_at_rh((center - light_dir) * distance, center, Vec3::Y);

            let light_space_corners: Vec<Vec3> = frustum_corners
                .iter()
                .map(|&c| (light_view * c.extend(1.0)).xyz())
                .collect();

            let mut min = Vec3::splat(f32::INFINITY);
            let mut max = Vec3::splat(f32::NEG_INFINITY);
            for c in &light_space_corners {
                min = min.min(*c);
                max = max.max(*c);
            }

            let light_proj = Mat4::orthographic_rh(min.x, max.x, min.y, max.y, -max.z - 10.0, -min.z + 10.0);

            self.directional_lights[id as usize].projections[i] = (light_proj * light_view).to_cols_array();
            write_to_buffer(&self.directional_light_shadow_map_buffers[id as usize][i],0,bytemuck::bytes_of(&(light_proj * light_view).to_cols_array()))
            //write_to_buffer(&self.directional_light_shadow_map_buffers[id as usize][i],0,bytemuck::bytes_of(&(Mat4::from_cols_array(&[0.2886751, 0.0, 0.0, 0.0, 0.0, 0.5773502, 0.0, 0.0, 0.0, 0.0, -1.0526316, -1.0, 0.0, 0.0, -0.105263166, 0.0])).to_cols_array()))
        }
    }
    pub fn recreate_bind_groups(&mut self) {
        self.bind_group_compute = create_bind_group(
            "OrderedSetComputeBindGroup",
            &self.bind_group_compute_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.point_light_len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.point_light_data.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: self.spot_light_len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: self.spot_light_data.as_entire_binding(),
                },
            ],
        );
        self.bind_group_fragment = create_bind_group(
            "OrderedSetFragmentBindGroup",
            &self.bind_group_fragment_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.point_light_len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.point_light_data.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: self.spot_light_len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: self.spot_light_data.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: self.directional_light_len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: self.directional_light_data.as_entire_binding(),
                },
            ],
        );
    }
    pub fn recreate_directional_textures(&mut self) {
        let directional_light_shadow_map_textures = [
            create_texture(&wgpu::TextureDescriptor {
                label: Some("DirectionalTextureArray_1"),
                size: Extent3d {
                    width: self.shadows_texture_size,
                    height: self.shadows_texture_size,
                    depth_or_array_layers: self.directional_light_len,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
            create_texture(&wgpu::TextureDescriptor {
                label: Some("DirectionalTextureArray_2"),
                size: Extent3d {
                    width: self.shadows_texture_size,
                    height: self.shadows_texture_size,
                    depth_or_array_layers: self.directional_light_len,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
            create_texture(&wgpu::TextureDescriptor {
                label: Some("DirectionalTextureArray_3"),
                size: Extent3d {
                    width: self.shadows_texture_size,
                    height: self.shadows_texture_size,
                    depth_or_array_layers: self.directional_light_len,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
        ];
        let directional_light_shadow_map_whole_texture_views = [
            directional_light_shadow_map_textures
                [0]
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("DirectionalLightTextureArray_0"),
                    format: None,
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                    aspect: wgpu::TextureAspect::DepthOnly,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                }),
            directional_light_shadow_map_textures
                [1]
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("DirectionalLightTextureArray_1"),
                    format: None,
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                    aspect: wgpu::TextureAspect::DepthOnly,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                }),
            directional_light_shadow_map_textures
                [2]
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("DirectionalLightTextureArray_2"),
                    format: None,
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                    aspect: wgpu::TextureAspect::DepthOnly,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                }),
        ];
        let mut directional_light_shadow_map_texture_views = Vec::with_capacity(self.directional_light_len as usize);
        for i in 0..self.directional_light_len {
            directional_light_shadow_map_texture_views.push(
                [directional_light_shadow_map_textures[0].create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!("DirectionalLightRenderView_0_{}", i)),
                    format: Some(wgpu::TextureFormat::Depth32Float),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: i,
                    array_layer_count: Some(1),
                    ..Default::default()
                }),
                    directional_light_shadow_map_textures[1].create_view(&wgpu::TextureViewDescriptor {
                        label: Some(&format!("DirectionalLightRenderView_1_{}", i)),
                        format: Some(wgpu::TextureFormat::Depth32Float),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        base_array_layer: i,
                        array_layer_count: Some(1),
                        ..Default::default()
                    }),
                    directional_light_shadow_map_textures[2].create_view(&wgpu::TextureViewDescriptor {
                        label: Some(&format!("DirectionalLightRenderView_2_{}", i)),
                        format: Some(wgpu::TextureFormat::Depth32Float),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        base_array_layer: i,
                        array_layer_count: Some(1),
                        ..Default::default()
                    }),] )
        }
        self.directional_light_shadow_map_textures = directional_light_shadow_map_textures;
        self.directional_light_shadow_map_whole_texture_views = directional_light_shadow_map_whole_texture_views;
        self.directional_light_shadow_map_texture_views = directional_light_shadow_map_texture_views;
        for i in self.directional_light_shadow_map_enabled.len()..self.directional_light_len as usize {
            self.directional_light_shadow_map_enabled.push(false)
        }
    }
}

pub struct OrderedSet {
    pub data: Mutex<DynamicLightSet>,
}
impl OrderedSet {
    pub fn new(shadow_size:u32) -> Self {
        Self {
            data: Mutex::new(DynamicLightSet::new(shadow_size)),
        }
    }
    pub fn point_allocate(&self, size: u32) {
        self.data.lock().unwrap().allocate_point_light(size);
    }
    pub fn add_point_light(&self, data: &PointLightData) -> u32 {
        self.data.lock().unwrap().add_point_light(data)
    }
    pub fn remove_point_light(&self, id: u32) {
        self.data.lock().unwrap().remove_point_light(id)
    }
    pub fn update_point_light(&self, data: &PointLightData, id: u32) {
        self.update_point_light(data, id);
    }
    pub fn spot_allocate(&self, size: u32) {
        self.data.lock().unwrap().allocate_spot_light(size);
    }
    pub fn add_spot_light(&self, data: &SpotLightData) -> u32 {
        self.data.lock().unwrap().add_spot_light(data)
    }
    pub fn remove_spot_light(&self, id: u32) {
        self.data.lock().unwrap().remove_spot_light(id)
    }
    pub fn update_spot_light(&self, data: &SpotLightData, id: u32) {
        self.update_spot_light(data, id);
    }
    pub fn directional_allocate(&self, size: u32) {
        self.data.lock().unwrap().allocate_directional_light(size);
    }
    pub fn add_directional_light(&self, data: &DirectionalLightData) -> u32 {
        self.data.lock().unwrap().add_directional_light(data)
    }
    pub fn update_directional_light(&self, data: &DirectionalLightData, id: u32) {
        self.update_directional_light(data, id);
    }
    pub fn remove_directional_light(&self, id: u32) {
        self.data.lock().unwrap().remove_directional_light(id)
    }
    pub fn set_frustum_corners_world(
        & self,
        inv_view_proj: Mat4,
        proj: Mat4,
        near: f32,
        far: f32,
    ){
        self.data.lock().unwrap().set_frustum_corners_world(inv_view_proj,proj,near,far)
    }
}
