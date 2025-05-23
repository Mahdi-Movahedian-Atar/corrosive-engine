use crate::render_graph::GraphNode;
use crate::wgpu::BindGroupEntry;
use crate::STATE;
use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::Resource;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType,
    BufferUsages, ColorTargetState, ColorWrites, Device, Extent3d, FragmentState, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StoreOp, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDimension, VertexState,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

#[derive(Resource, Default)]
pub struct Renderer(pub Option<JoinHandle<()>>);

pub enum StarchMode {
    Keep(u32, u32),
    KeepWidth(u32),
    KeepHeight(u32),
    Starch,
    AspectRatio(f32),
}

#[derive(Resource)]
pub struct WindowOptions {
    pub window: Option<Arc<Window>>,
    pub starch_mode: StarchMode,
    pub func:
        Vec<fn(&mut App, event_loop: &ActiveEventLoop, window_id: &WindowId, event: &WindowEvent)>,
}
impl Default for WindowOptions {
    fn default() -> Self {
        WindowOptions {
            window: None,
            starch_mode: StarchMode::Starch,
            func: vec![|app,
                        _event_loop: &ActiveEventLoop,
                        _window_id: &WindowId,
                        event: &WindowEvent| {
                match event {
                    WindowEvent::Resized(t) => unsafe {
                        match &app.window_options.f_read().starch_mode {
                            StarchMode::Keep(w, h) => {
                                let aspect = *w as f32 / *h as f32;
                                if &t.width < w || &t.height < h {
                                    if aspect < t.width as f32 / t.height as f32 {
                                        if let Some(s) = &mut STATE {
                                            s.resize(
                                                &PhysicalSize::new(
                                                    (t.height as f32 * aspect) as u32,
                                                    t.height,
                                                ),
                                                t,
                                            );
                                        }
                                    } else {
                                        if let Some(s) = &mut STATE {
                                            s.resize(
                                                &PhysicalSize::new(
                                                    t.width,
                                                    (t.width as f32 / aspect) as u32,
                                                ),
                                                t,
                                            );
                                        }
                                    }
                                } else {
                                    if let Some(s) = &mut STATE {
                                        s.resize(&PhysicalSize::new(*w, *h), t);
                                    }
                                }
                            }
                            StarchMode::KeepWidth(w) => {
                                if &t.width > w {
                                    if let Some(s) = &mut STATE {
                                        s.resize(&PhysicalSize::new(*w, t.height), t);
                                    }
                                } else {
                                    if let Some(s) = &mut STATE {
                                        s.resize(t, t);
                                    }
                                }
                            }
                            StarchMode::KeepHeight(h) => {
                                if &t.height > h {
                                    if let Some(s) = &mut STATE {
                                        s.resize(&PhysicalSize::new(t.width, *h), t);
                                    }
                                } else {
                                    if let Some(s) = &mut STATE {
                                        s.resize(t, t);
                                    }
                                }
                            }
                            StarchMode::Starch => {
                                if let Some(s) = &mut STATE {
                                    s.resize(t, t)
                                }
                            }
                            StarchMode::AspectRatio(a) => {
                                if *a < t.width as f32 / t.height as f32 {
                                    if let Some(s) = &mut STATE {
                                        s.resize(
                                            &PhysicalSize::new(
                                                (t.height as f32 * *a) as u32,
                                                t.height,
                                            ),
                                            t,
                                        );
                                    }
                                } else {
                                    if let Some(s) = &mut STATE {
                                        s.resize(
                                            &PhysicalSize::new(
                                                t.width,
                                                (t.width as f32 / *a) as u32,
                                            ),
                                            t,
                                        );
                                    }
                                }
                            }
                        }
                    },
                    WindowEvent::RedrawRequested => {
                        if let Some(t) = &app.window_options.f_read().window {
                            t.request_redraw();
                            unsafe {
                                if let Some(t) = &STATE {
                                    t.render().unwrap()
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }],
        }
    }
}
impl WindowOptions {
    pub fn window(&self) -> &Window {
        self.window.as_ref().unwrap()
    }
}

#[derive(Resource, Default)]
pub struct RenderGraph {
    pub(crate) pass_names: HashMap<String, usize>,
    pub(crate) pass_nodes: HashMap<usize, GraphNode>,
    pub(crate) edges: Vec<(usize, usize)>,
    pub(crate) execution_levels: Vec<Vec<usize>>,
}

pub struct State<'a> {
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: Arc<RwLock<wgpu::SurfaceConfiguration>>,
    pub(crate) size: PhysicalSize<u32>,
    pub(crate) v_size: PhysicalSize<u32>,
    pub(crate) render_graph: Res<RenderGraph>,
    pub(crate) device: Device,
    pub(crate) resolution_buffer: Buffer,
    pub(crate) resolution_bind_group: BindGroup,
    pub(crate) resolution_bind_group_layout: BindGroupLayout,
    pub(crate) v_pipeline: RenderPipeline,
    pub(crate) v_bind_group_layout: BindGroupLayout,
    pub(crate) v_texture: Texture,
    pub(crate) v_texture_view: TextureView,
    pub(crate) v_sampler: Sampler,
    pub(crate) v_scale_buffer: Buffer,
    pub(crate) v_bind_group: BindGroup,
    pub depth_texture: Texture,
    pub depth_view: TextureView,
}
impl<'a> State<'a> {
    async fn new(window: Arc<Window>, render_graph: Res<RenderGraph>) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .enumerate_adapters(wgpu::Backends::all())
            .into_iter()
            .filter(|adapter| adapter.is_surface_supported(&surface))
            .next()
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = Arc::new(RwLock::new(wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }));

        surface.configure(&device, &config.read().unwrap());

        let resolution_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: "resolution_buffer".into(),
            contents: &bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let resolution_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: "resolution_buffer_layout".into(),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let resolution_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: "resolution_bind_group".into(),
            layout: &resolution_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: (resolution_buffer.as_entire_binding()),
            }],
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("PostProcess Shader"),
            source: ShaderSource::Wgsl(
                "
            struct ScaleUniform { scale: vec2<f32> };
            struct VertexOutput {
                @builtin(position) clip_position : vec4 <f32>,
                @location(0) uv : vec2 <f32>
            };
            @group(0) @binding(2) var<uniform> uni: ScaleUniform;
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
                let p = pos[vid] * uni.scale;
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

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("postprocess bind group layout"),
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
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("postprocess pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("postprocess pipeline"),
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
                    format: config.read().unwrap().format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("postprocess sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let scale_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("scale uniform"),
            contents: bytemuck::cast_slice(&[1.0f32, 1.0f32]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let dummy_view = device.create_texture(&TextureDescriptor {
            label: Some("dummy texture"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: config.read().unwrap().format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = dummy_view.create_view(&Default::default());

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("PostProcess bind group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: scale_buffer.as_entire_binding(),
                },
            ],
        });

        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        State {
            surface,
            queue,
            config,
            size,
            v_size: size,
            render_graph,
            device,
            resolution_buffer,
            resolution_bind_group_layout,
            resolution_bind_group,
            v_pipeline: pipeline,
            v_bind_group_layout: bind_group_layout,
            v_texture: dummy_view,
            v_texture_view: view,
            v_sampler: sampler,
            v_scale_buffer: scale_buffer,
            v_bind_group: bind_group,
            depth_texture,
            depth_view,
        }
    }
    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_graph.f_read().execute(
            &self.device,
            &self.queue,
            &self.v_texture_view,
            &self.depth_view,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Present Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Present Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.v_pipeline);
            render_pass.set_bind_group(0, &self.v_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>, absolute_size: &PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = absolute_size.clone();
            self.v_size = new_size.clone();
            self.config.write().unwrap().width = absolute_size.width.clone();
            self.config.write().unwrap().height = absolute_size.height.clone();
            self.surface
                .configure(&self.device, &self.config.read().unwrap());
            self.queue.write_buffer(
                &self.resolution_buffer,
                0,
                &bytemuck::cast_slice(&[new_size.width as f32, new_size.height as f32]),
            );
            self.queue.write_buffer(
                &self.v_scale_buffer,
                0,
                &bytemuck::cast_slice(&[
                    new_size.width as f32 / absolute_size.width as f32,
                    new_size.height as f32 / absolute_size.height as f32,
                ]),
            );

            self.v_texture = self.device.create_texture(&TextureDescriptor {
                label: Some("Proxy Render Texture (Resized)"),
                size: Extent3d {
                    width: new_size.width,
                    height: new_size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: self.v_texture.format(),
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            self.v_texture_view = self.v_texture.create_view(&Default::default());

            // Recreate bind group with new texture view
            self.v_bind_group = self.device.create_bind_group(&BindGroupDescriptor {
                label: Some("Proxy Texture Bind Group (Resized)"),
                layout: &self.v_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&self.v_texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&self.v_sampler),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: self.v_scale_buffer.as_entire_binding(),
                    },
                ],
            });

            self.depth_texture = self.device.create_texture(&TextureDescriptor {
                label: Some("Depth Texture"),
                size: Extent3d {
                    width: new_size.width,
                    height: new_size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            self.depth_view = self
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
        }
    }
}

pub struct App {
    render_graph: Res<RenderGraph>,
    pub window_options: Res<WindowOptions>,
}

impl App {
    pub(crate) fn new(window_options: Res<WindowOptions>, render_graph: Res<RenderGraph>) -> App {
        App {
            render_graph,
            window_options,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window_options.f_write().window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));
        self.render_graph.f_write().prepare();

        if let Some(t) = &self.window_options.f_read().window {
            let state = pollster::block_on(State::new(t.clone(), self.render_graph.clone()));
            unsafe {
                STATE = Some(state);
            }
            t.request_redraw();
        } else {
            panic!("failed to run renderer backend")
        };
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let funcs = self.window_options.f_read().func.clone();
        funcs.iter().for_each(|f| {
            f(self, event_loop, &window_id, &event);
        });
    }
}
