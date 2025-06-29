use corrosive_asset_manager::cache_server::CacheServer;
use corrosive_ecs_renderer_backend::assets::BindGroupLayoutAsset;
use corrosive_ecs_renderer_backend::public_functions::{
    create_bind_group, create_buffer_init, get_device, write_to_buffer,
};
use corrosive_ecs_renderer_backend::wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor, BufferUsages,
    ShaderStages,
};
use std::cmp::max;
use std::sync::Mutex;
use crate::comp::light::point_light::PointLightData;
use crate::comp::light::spot_light::SpotLightData;

const POINT_LIGHT_SIZE: u32 = 8u32;
const SPOT_LIGHT_SIZE: u32 = 8u32;
const ARIAL_LIGHT_SIZE: u32 = 4u32;
const DIRECTIONAL_LIGHT_SIZE: u32 = 2u32;

pub(crate) struct DynamicLightSet {
    point_light_available_ids: Vec<u32>,
    point_light_len: u32,
    point_light_data: Buffer,
    point_light_len_buffer: Buffer,
    spot_light_available_ids: Vec<u32>,
    spot_light_len: u32,
    spot_light_data: Buffer,
    spot_light_len_buffer: Buffer,
    pub(crate) bind_group_compute: BindGroup,
    pub(crate) bind_group_fragment: BindGroup,
    pub(crate) bind_group_compute_layout: BindGroupLayout,
    pub(crate) bind_group_fragment_layout: BindGroupLayout,
}
impl DynamicLightSet {
    pub fn allocate_point_light(&mut self, size: u32) {
        let mut size = max((size / POINT_LIGHT_SIZE + 1) *POINT_LIGHT_SIZE, 1);
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
            self.point_light_data.slice(..).get_mapped_range().to_vec().as_slice(),
        );
        self.point_light_data = new_buffer;
        self.recreate_bind_groups();
    }
    pub fn add_point_light(&mut self, data: PointLightData) -> u32 {
        if let Some(id) = self.point_light_available_ids.pop() {
            write_to_buffer(
                &self.point_light_data,
                (id * (size_of::<PointLightData>() as u32)) as BufferAddress,
                bytemuck::bytes_of(&data),
            );
            if self.point_light_len <= id {
                self.point_light_len = id + 1;
                write_to_buffer(&self.point_light_len_buffer, 0, bytemuck::bytes_of(&self.point_light_len));
            }
            id
        } else {
            self.allocate_point_light(self.point_light_len + 1);
            self.add_point_light(data)
        }
    }
    pub fn remove_point_light(&mut self, id: u32) {
        write_to_buffer(
            &self.point_light_data,
            (id * size_of::<PointLightData>() as u32) as BufferAddress,
            vec![0u8; size_of::<PointLightData>()].as_slice(),
        );
        if self.point_light_len - 1 == id {
            self.point_light_len = id;
            write_to_buffer(&self.point_light_len_buffer, 0, bytemuck::bytes_of(&self.point_light_len));
        }
        self.point_light_available_ids.push(id);
    }
    pub fn allocate_spot_light(&mut self, size: u32) {
        let mut size = max((size / SPOT_LIGHT_SIZE + 1) *SPOT_LIGHT_SIZE, 1);
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
            self.spot_light_data.slice(..).get_mapped_range().to_vec().as_slice(),
        );
        self.spot_light_data = new_buffer;
        self.recreate_bind_groups();
    }
    pub fn add_spot_light(&mut self, data: SpotLightData) -> u32 {
        if let Some(id) = self.spot_light_available_ids.pop() {
            write_to_buffer(
                &self.spot_light_data,
                (id * (size_of::<SpotLightData>() as u32)) as BufferAddress,
                bytemuck::bytes_of(&data),
            );
            if self.spot_light_len <= id {
                self.spot_light_len = id + 1;
                write_to_buffer(&self.spot_light_len_buffer, 0, bytemuck::bytes_of(&self.spot_light_len));
            }
            id
        } else {
            self.allocate_point_light(self.spot_light_len + 1);
            self.add_spot_light(data)
        }
    }
    pub fn remove_spot_light(&mut self, id: u32) {
        write_to_buffer(
            &self.spot_light_data,
            (id * size_of::<SpotLightData>() as u32) as BufferAddress,
            vec![0u8; size_of::<SpotLightData>()].as_slice(),
        );
        if self.spot_light_len - 1 == id {
            self.spot_light_len = id;
            write_to_buffer(&self.spot_light_len_buffer, 0, bytemuck::bytes_of(&self.spot_light_len));
        }
        self.spot_light_available_ids.push(id);
    }
    pub fn new() -> Self {
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
            bind_group_compute,
            bind_group_compute_layout,
            bind_group_fragment,
            bind_group_fragment_layout,
        }
    }
    pub fn recreate_bind_groups(&mut self){
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
            ],
        );
    }
}

pub struct OrderedSet {
    pub data: Mutex<DynamicLightSet>,
}
impl OrderedSet {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(DynamicLightSet::new()),
        }
    }
    pub fn point_allocate(&self, size: u32) {
        self.data.lock().unwrap().allocate_point_light(size);
    }
    pub fn add_point_light(&self, data: PointLightData) -> u32 {
        self.data.lock().unwrap().add_point_light(data)
    }
    pub fn remove_point_light(&self, id: u32) {
        self.data.lock().unwrap().remove_point_light(id)
    }
    pub fn spot_allocate(&self, size: u32) {
        self.data.lock().unwrap().allocate_spot_light(size);
    }
    pub fn add_spot_light(&self, data: SpotLightData) -> u32 {
        self.data.lock().unwrap().add_spot_light(data)
    }
    pub fn remove_spot_light(&self, id: u32) {
        self.data.lock().unwrap().remove_spot_light(id)
    }
}
