use crate::color_palette::ColorPallet;
use crate::comp::camera::ActivePixilCamera;
use crate::comp::dynamic::{PixilDynamicObject, PixilDynamicObjectData};
use crate::comp::light::point_light::PointLightData;
use crate::comp::render::PixilRenderSettings;
use crate::helper_functions::view_bind_group_layout;
use crate::light_set::OrderedSet;
use crate::render_set::RenderSet;
use corrosive_asset_manager::cache_server::{Cache, CacheServer};
use corrosive_asset_manager_macro::static_hasher;
use corrosive_ecs_core::ecs_core::{Arch, Member, Reference, Res};
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::assets::BindGroupLayoutAsset;
use corrosive_ecs_renderer_backend::comp::{RenderGraph, WindowOptions};
use corrosive_ecs_renderer_backend::public_functions::{
    create_bind_group, create_bind_group_layout, create_buffer_init, get_device,
    get_surface_format, get_window_ratio, read_shader, write_to_buffer,
};
use corrosive_ecs_renderer_backend::render_graph::{CommandEncoder, Device, Queue, RenderNode};
use corrosive_ecs_renderer_backend::wgpu;
use corrosive_ecs_renderer_backend::wgpu::util::RenderEncoder;
use corrosive_ecs_renderer_backend::wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferAddress, BufferBindingType,
    BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Extent3d, FragmentState, IndexFormat, LoadOp,
    Operations, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderBundle,
    RenderBundleEncoder, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StoreOp, Texture, TextureDescriptor, TextureDimension, TextureSampleType,
    TextureUsages, TextureView, TextureViewDimension, VertexState,
};
use corrosive_ecs_renderer_backend::winit::event::WindowEvent;
use glam::Vec4;
use std::cell::{LazyCell, UnsafeCell};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;
use crate::comp::position_pixil::PositionPixil;

#[repr(align(16))]
struct Cluster {
    min_point: Vec4,
    max_point: Vec4,
    count: u32,
    light_indices: [u32; 100],
}
pub static DYNAMIC_OBJECTS: RenderSet<(PixilDynamicObjectData)> = RenderSet::new();
pub static DYNAMIC_LIGHTS: LazyLock<OrderedSet> =
    LazyLock::new(|| OrderedSet::new());

pub static COLOR_PALLET: LazyLock<ColorPallet> = LazyLock::new(|| ColorPallet::new());

struct RenderPixilNode {
    object_view_bind_group: BindGroup,
    render_bind_group: BindGroup,
    render_bind_group_layout: BindGroupLayout,
    render_pipeline: RenderPipeline,
    cluster_buffer: Buffer,
    cluster_bind_group: BindGroup,
    cluster_pipeline: ComputePipeline,
    light_bind_group: BindGroup,
    lights_pipeline: ComputePipeline,
    render_settings: Res<PixilRenderSettings>,
}
impl RenderNode for RenderPixilNode {
    fn name(&self) -> &str {
        "RenderPixilNode"
    }

    fn execute(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        depth_view: &TextureView,
    ) {
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Pixil Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.cluster_pipeline);
            compute_pass.set_bind_group(0, &self.cluster_bind_group, &[]);
            let size = self.render_settings.f_write().grid_size;
            compute_pass.dispatch_workgroups(size[0], size[1], size[2]);

            compute_pass.set_pipeline(&self.lights_pipeline);
            compute_pass.set_bind_group(0, &self.light_bind_group, &[]);
            compute_pass.set_bind_group(
                1,
                &DYNAMIC_LIGHTS.data.lock().unwrap().bind_group_compute,
                &[],
            );
            compute_pass.dispatch_workgroups(size[0], size[1], size[2]);
        }
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Pixil Low Resolutions"),
                color_attachments: &[Option::from(RenderPassColorAttachment {
                    view: self.render_settings.f_write().get_view(),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            for i in DYNAMIC_OBJECTS.data.lock().unwrap().enabled.iter() {
                render_pass.set_pipeline(i.pipeline);
                render_pass.set_bind_group(0, &self.object_view_bind_group, &[]);
                render_pass.set_bind_group(1, &i.transform_bind_group, &[]);
                render_pass.set_bind_group(
                    2,
                    &DYNAMIC_LIGHTS.data.lock().unwrap().bind_group_fragment,
                    &[],
                );
                render_pass.set_bind_group(3, i.material_bind_group, &[]);
                render_pass.set_vertex_buffer(0, i.vertex_buffer.slice(..));
                render_pass.set_index_buffer(i.index_buffer.slice(..), IndexFormat::Uint32);
                render_pass.draw_indexed(0..*i.count, 0, 0..1);
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Pixil high resolutions"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}

#[task]
pub fn start_pixil_renderer(
    render_setting: Res<PixilRenderSettings>,
    active_pixil_camera: Res<ActivePixilCamera>,
    graph: Res<RenderGraph>,
    window: Res<WindowOptions>,
) {
    let device = get_device();

    //window
    let setting_clone = render_setting.clone();
    window.f_write().func.push(Arc::new(move |_, _, _, s| {
        if let WindowEvent::Resized(_) = s {
            {
                setting_clone.f_write().update_texture();
            }
        }
    }));

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("pixil renderer Shader"),
        source: ShaderSource::Wgsl(
            "
            struct VertexOutput {
                @builtin(position) clip_position : vec4 <f32>,
                @location(0) uv : vec2 <f32>
            };
            @group(0) @binding(0) var tex: texture_2d<f32>;
            @group(0) @binding(1) var samp: sampler;
            @group(0) @binding(2) var<uniform> resolution : vec2<u32>;

            @vertex fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOutput {
                var output: VertexOutput;
                var pos = array<vec2<f32>, 3>(
                    vec2<f32>(-1.0, 1.0),
                    vec2<f32>( 3.0, 1.0),
                    vec2<f32>(-1.0,  -3.0),
                );
                var uv = array<vec2<f32>, 3>(
                    vec2<f32>(0.0, 0.0),
                    vec2<f32>(2.0, 0.0),
                    vec2<f32>(0.0, 2.0),
                );
                let p = pos[vid];
                output.clip_position = vec4<f32>(p, 0.0, 1.0);
                output.uv = uv[vid];
                return output;
            }

            @fragment fn fs_main(coord: VertexOutput) -> @location(0) vec4<f32> {
                let tex_size = vec2<f32>(f32(resolution.x), f32(resolution.y));

                let snapped_uv = (floor(coord.uv * tex_size) + vec2<f32>(0.5)) / tex_size;

                return textureSample(tex, samp, coord.uv);
            }"
            .into(),
        ),
    });

    let sampler = device.create_sampler(&SamplerDescriptor {
        label: Some("pixil renderer sampler"),
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

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("pixil renderer bind group layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2,
                    sample_type: TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
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
        ],
    });

    let bind_group = {
        render_setting.f_write().set_view();
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("pixil renderer bind group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        render_setting.f_read().get_view(),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: render_setting.f_read().size_buffer.as_entire_binding(),
                },
            ],
        })
    };

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("pixil renderer pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("pixil renderer pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main".into(),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main".into(),
            compilation_options: Default::default(),
            targets: &[Some(ColorTargetState {
                format: get_surface_format(),
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
        cache: None,
    });

    //clusters

    let cluster_buffer = device.create_buffer(&BufferDescriptor {
        label: "ClusterBuffer".into(),
        size: {
            let size = render_setting.f_read().grid_size;
            (size[0] * size[1] * size[2] * size_of::<Cluster>() as u32) as BufferAddress
        },
        usage: BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let cluster_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("cluster bind group layout"),
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
                    ty: BufferBindingType::Uniform,
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
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let cluster_bind_group = {
        let mut settings = render_setting.f_read();
        let mut camera = active_pixil_camera.f_read();
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("ClusterBindGroup"),
            layout: &cluster_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera.z_params_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: camera.inverse_projection_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: settings.grid_size_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: settings.size_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: cluster_buffer.as_entire_binding(),
                },
            ],
        })
    };

    let cluster_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: "Clustering pipeline".into(),
        layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: "Clustering pipeline layout".into(),
            bind_group_layouts: &[&cluster_bind_group_layout],
            push_constant_ranges: &[],
        })),
        module: &(device.create_shader_module(ShaderModuleDescriptor {
            label: Some("pixil clustering shader"),
            source: ShaderSource::Wgsl(
                read_shader("packages/pixil/shaders/clustering.wgsl")
                    .expect("failed to read shader")
                    .into(),
            ),
        })),
        entry_point: "main".into(),
        compilation_options: Default::default(),
        cache: None,
    });

    //lights

    let light_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("cluster bind group layout"),
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
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let light_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("LightBindGroup"),
        layout: &light_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: active_pixil_camera.f_read().view_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: render_setting.f_read().grid_size_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: cluster_buffer.as_entire_binding(),
            },
        ],
    });

    let lights_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: "Light pipeline".into(),
        layout: Some(
            &device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: "Light pipeline layout".into(),
                bind_group_layouts: &[
                    &light_bind_group_layout,
                    &DYNAMIC_LIGHTS
                        .data
                        .lock()
                        .unwrap()
                        .bind_group_compute_layout,
                ],
                push_constant_ranges: &[],
            }),
        ),
        module: &(device.create_shader_module(ShaderModuleDescriptor {
            label: Some("pixil lights_to_clusters shader"),
            source: ShaderSource::Wgsl(
                read_shader("packages/pixil/shaders/lights_to_clusters.wgsl")
                    .expect("failed to read shader")
                    .into(),
            ),
        })),
        entry_point: "main".into(),
        compilation_options: Default::default(),
        cache: None,
    });

    //test

    DYNAMIC_LIGHTS.add_point_light(PointLightData {
        position: [-1.0, 0.0, 0.0, 1.0],
        radius: 2.0,
        intensity: 1.0,
        color_index: 0,
        shade_mask: 0,
        cast_shadow_mask: 0,
    });
    DYNAMIC_LIGHTS.add_point_light(PointLightData {
        position: [1.0, 0.0, 0.0, 1.0],
        radius: 2.0,
        intensity: 1.0,
        color_index: 1,
        shade_mask: 0,
        cast_shadow_mask: 0,
    });
    DYNAMIC_LIGHTS.add_point_light(PointLightData {
        position: [0.0, 0.0, 2.0, 1.0],
        radius: 2.0,
        intensity: 1.0,
        color_index: 2,
        shade_mask: 0,
        cast_shadow_mask: 0,
    });

    //object

    let view_layout: Cache<BindGroupLayoutAsset> = view_bind_group_layout();

    let render_setting_clone = render_setting.clone();

    graph.f_write().add_node(Box::new(RenderPixilNode {
        object_view_bind_group: create_bind_group(
            "PixilViewBindGroup",
            &view_layout.get().layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: render_setting_clone
                        .f_read()
                        .size_buffer
                        .as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: active_pixil_camera.f_read().view_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: active_pixil_camera
                        .f_read()
                        .projection_buffer
                        .as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: render_setting_clone
                        .f_read()
                        .grid_size_buffer
                        .as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: cluster_buffer.as_entire_binding(),
                },
            ],
        ),
        render_bind_group: bind_group,
        render_bind_group_layout: bind_group_layout,
        render_pipeline: pipeline,
        render_settings: render_setting,
        cluster_pipeline,
        cluster_bind_group,
        cluster_buffer,
        lights_pipeline,
        light_bind_group,
    }));
    graph.f_write().prepare();
}

#[task]
pub fn update_camera(
    active_camera: Res<ActivePixilCamera>,
    renderer_settings: Res<PixilRenderSettings>,
) {
    let active_camera = active_camera.f_read();
    active_camera.update_view_matrix();
}
#[task]
pub fn update_pixil_position(pixil_dynamic_object: Arch<(&PixilDynamicObject, &Member<PositionPixil>)>){
    for (k,(i,j)) in pixil_dynamic_object.iter().enumerate() {
        let mut lock = j.dry_f_write();
        if let Reference::Some(t) = &mut *lock{
            if t.dirty{
                write_to_buffer(&i.transform_data,0,bytemuck::bytes_of(&t.uniform()));
                t.dirty = false;
            }
        }else { pixil_dynamic_object.remove(k) }
    }
}