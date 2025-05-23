use crate::comp::render::PixilRenderSettings;
use crate::render_set::RenderSet;
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::task;
use corrosive_ecs_renderer_backend::comp::RenderGraph;
use corrosive_ecs_renderer_backend::public_functions::{
    get_device, get_surface_format, get_window_ratio,
};
use corrosive_ecs_renderer_backend::render_graph::{CommandEncoder, Device, Queue, RenderNode};
use corrosive_ecs_renderer_backend::wgpu;
use corrosive_ecs_renderer_backend::wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Color, ColorTargetState, ColorWrites, Extent3d,
    FragmentState, LoadOp, Operations, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
    RenderBundle, RenderBundleEncoder, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, SamplerDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StoreOp, Texture, TextureDescriptor, TextureDimension,
    TextureSampleType, TextureUsages, TextureView, TextureViewDimension, VertexState,
};
use std::cell::{LazyCell, UnsafeCell};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

pub static DYNAMIC_OBJECTS: RenderSet<RenderBundle> = RenderSet::new();

struct RenderPixilNode {
    render_bind_group: BindGroup,
    render_bind_group_layout: BindGroupLayout,
    render_pipeline: RenderPipeline,
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
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Pixil Low Resolutions"),
                color_attachments: &[Option::from(RenderPassColorAttachment {
                    view: self.render_settings.f_write().get_view(),
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
            render_pass.execute_bundles(DYNAMIC_OBJECTS.data.lock().unwrap().enabled.iter());
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
pub fn start_pixil_renderer(render_setting: Res<PixilRenderSettings>, graph: Res<RenderGraph>) {
    let shader = get_device().create_shader_module(ShaderModuleDescriptor {
        label: Some("pixil renderer Shader"),
        source: ShaderSource::Wgsl(
            "
            struct VertexOutput {
                @builtin(position) clip_position : vec4 <f32>,
                @location(0) uv : vec2 <f32>
            };
            @group(0) @binding(0) var tex: texture_2d<f32>;
            @group(0) @binding(1) var samp: sampler;

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
                if (coord.uv.x > 1 || coord.uv.y > 1){
                    return vec4<f32>(0.0);
                }
                return textureSample(tex, samp, coord.uv);
            }"
            .into(),
        ),
    });

    let sampler = get_device().create_sampler(&SamplerDescriptor {
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

    let bind_group_layout = get_device().create_bind_group_layout(&BindGroupLayoutDescriptor {
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
                ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    let bind_group = get_device().create_bind_group(&BindGroupDescriptor {
        label: Some("pixil renderer bind group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(render_setting.f_write().get_view()),
            },
            BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let pipeline_layout = get_device().create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("pixil renderer pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = get_device().create_render_pipeline(&RenderPipelineDescriptor {
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

    graph.f_write().add_node(Box::new(RenderPixilNode {
        render_bind_group: bind_group,
        render_bind_group_layout: bind_group_layout,
        render_pipeline: pipeline,
        render_settings: render_setting,
    }));
    graph.f_write().prepare();
}
