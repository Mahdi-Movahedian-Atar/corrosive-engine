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

pub enum ReserveStrategy {
    Fit,
    Fixed(u32),
    Align(u32),
}
pub trait OrderedSetTrait {
    const NAME: &'static str;
    fn empty<'a>() -> Vec<u8>
    where
        Self: Sized,
    {
        let buffer_size = size_of::<Self>() as u64;
        let zero_data = vec![0u8; buffer_size as usize];
        zero_data
    }
}
pub(crate) struct OrderedData<T: bytemuck::Pod + OrderedSetTrait> {
    reserve_strategy: ReserveStrategy,
    available_ids: Vec<u32>,
    pub len: u32,
    data: Buffer,
    len_buffer: Buffer,
    pub(crate) bind_group_compute: BindGroup,
    pub(crate) bind_group_fragment: BindGroup,
    pub(crate) bind_group_compute_layout: BindGroupLayout,
    pub(crate) bind_group_fragment_layout: BindGroupLayout,
    _phantom_data: std::marker::PhantomData<T>,
}
impl<T: bytemuck::Pod + OrderedSetTrait> OrderedData<T> {
    pub fn allocate(&mut self, size: u32) {
        let mut size = max(size, 1);
        match self.reserve_strategy {
            ReserveStrategy::Fit => {}
            ReserveStrategy::Fixed(t) => {
                size = t;
            }
            ReserveStrategy::Align(t) => {
                size = (size / t + 1) * t;
            }
        }
        for i in (self.len..size).rev() {
            self.available_ids.push(i);
        }
        let new_buffer = get_device().create_buffer(&BufferDescriptor {
            label: T::NAME.into(),
            size: (size * size_of::<T>() as u32) as BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        write_to_buffer(
            &new_buffer,
            0,
            self.data.slice(..).get_mapped_range().to_vec().as_slice(),
        );
        self.data = new_buffer;
        self.bind_group_compute = create_bind_group(
            "OrderedSetComputeBindGroup",
            &self.bind_group_compute_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.data.as_entire_binding(),
                },
            ],
        );
        self. bind_group_fragment = create_bind_group(
            "OrderedSetFragmentBindGroup",
            &self.bind_group_fragment_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.data.as_entire_binding(),
                },
            ],
        );
    }
    pub fn add(&mut self, data: T) -> u32 {
        if let Some(id) = self.available_ids.pop() {
            println!("{}",size_of::<T>());
            write_to_buffer(
                &self.data,
                (id * (size_of::<T>() as u32)) as BufferAddress,
                bytemuck::bytes_of(&data),
            );
            if self.len <= id {
                self.len = id + 1;
                write_to_buffer(&self.len_buffer, 0, bytemuck::bytes_of(&self.len));
            }
            id
        } else {
            self.allocate(self.len + 1);
            self.add(data)
        }
    }
    pub fn remove(&mut self, id: u32) {
        write_to_buffer(
            &self.data,
            (id * size_of::<T>() as u32) as BufferAddress,
            T::empty().as_slice(),
        );
        if self.len - 1 == id {
            self.len = id;
            write_to_buffer(&self.len_buffer, 0, bytemuck::bytes_of(&self.len));
        }
        self.available_ids.push(id);
    }
    pub fn new(reserve_strategy: ReserveStrategy) -> Self {
        let size = match reserve_strategy {
            ReserveStrategy::Fit => 1,
            ReserveStrategy::Fixed(t) => t,
            ReserveStrategy::Align(t) => t,
        };
        let vec: Vec<u32> = (0..size).rev().collect();
        let buffer = get_device().create_buffer(&BufferDescriptor {
            label: T::NAME.into(),
            size: (size * size_of::<T>() as u32) as BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_compute_layout = get_device().create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: "OrderedSetComputeBindGroupLayout".into(),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE ,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE ,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let bind_group_fragment_layout = get_device().create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: "OrderedSetFragmentBindGroupLayout".into(),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT ,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT ,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let len_buffer = create_buffer_init(
            "OrderedSetLen",
            bytemuck::bytes_of(&size),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let bind_group_compute = create_bind_group(
            "OrderedSetComputeBindGroup",
            &bind_group_compute_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffer.as_entire_binding(),
                },
            ],
        );
        let bind_group_fragment = create_bind_group(
            "OrderedSetFragmentBindGroup",
            &bind_group_fragment_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: len_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffer.as_entire_binding(),
                },
            ],
        );
        Self {
            reserve_strategy,
            available_ids: vec,
            len: 0,
            data: buffer,
            len_buffer,
            bind_group_compute,
            bind_group_compute_layout,
            bind_group_fragment,
            bind_group_fragment_layout,
            _phantom_data: Default::default(),
        }
    }
}

pub struct OrderedSet<T: bytemuck::Pod + OrderedSetTrait> {
    pub data: Mutex<OrderedData<T>>,
}
impl<T: bytemuck::Pod + OrderedSetTrait> OrderedSet<T> {
    pub fn new(reserve_strategy: ReserveStrategy) -> Self {
        Self {
            data: Mutex::new(OrderedData::new(reserve_strategy)),
        }
    }
    pub fn allocate(&self, size: u32) {
        self.data.lock().unwrap().allocate(size);
    }
    pub fn add(&self, data: T) -> u32 {
        self.data.lock().unwrap().add(data)
    }
    pub fn remove(&self, id: u32) {
        self.data.lock().unwrap().remove(id)
    }
}
