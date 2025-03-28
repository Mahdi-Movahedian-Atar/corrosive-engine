/*pub mod uniform {
    pub type BindGroupLayoutDescriptor = wgpu::BindGroupLayoutDescriptor<'static>;
    pub type BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry;
    pub type ShaderStages = wgpu::ShaderStages;
    pub type BindingType = wgpu::BindingType;
    pub type BufferBindingType = wgpu::BufferBindingType;
    pub type BufferSize = wgpu::BufferSize;
    pub type BindGroup = wgpu::BindGroup;

    pub fn create_bind_group_layout_descriptor(
        name: Option<&'static str>,
        entries: &'static [wgpu::BindGroupLayoutEntry],
    ) -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: name,
            entries,
        }
    }

    pub trait UniformRenderable {
        fn desc() -> BindGroupLayoutDescriptor;
    }
}
pub mod vertex {
    pub type VertexBufferLayout = wgpu::VertexBufferLayout<'static>;
    pub type VertexStepMode = wgpu::VertexStepMode;
    pub type VertexAttribute = wgpu::VertexAttribute;
    pub type VertexFormat = wgpu::VertexFormat;

    pub trait VertexRenderable {
        fn desc() -> VertexBufferLayout;
    }
    pub fn create_vertex_buffer_layout<V>(
        step_mode: VertexStepMode,
        attributes: &'static [VertexAttribute],
    ) -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<V>() as wgpu::BufferAddress,
            step_mode,
            attributes,
        }
    }
}
pub mod general {
    use crate::DEVICE;
    use wgpu::util::DeviceExt;

    pub type Buffer = wgpu::Buffer;
    pub type BufferUsages = wgpu::BufferUsages;
    pub type BufferInitDescriptor = wgpu::util::BufferInitDescriptor<'static>;
    pub type IndexFormat = wgpu::IndexFormat;
    pub type BlendState = wgpu::BlendState;
    pub type BlendComponent = wgpu::BlendComponent;
    pub type ColorWrites = wgpu::ColorWrites;
    pub type ShaderModule = wgpu::ShaderModule;

    pub fn create_buffer(data: &BufferInitDescriptor) -> Buffer {
        unsafe {
            if let Some(t) = &DEVICE {
                t.create_buffer_init(data)
            } else {
                panic!("create buffer must be called after run_renderer task.")
            }
        }
    }
}
pub mod pipeline {
    use crate::DEVICE;
    use std::collections::HashMap;
    use std::num::NonZeroU32;
    use wgpu::{
        BindGroupLayout, ColorTargetState, CompareFunction, DepthBiasState, DepthStencilState,
        Face, FragmentState, FrontFace, IndexFormat, MultisampleState, PipelineCache,
        PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor, PolygonMode,
        PrimitiveState, PrimitiveTopology, ShaderModule, StencilState, TextureFormat,
        VertexBufferLayout,
    };

    pub type RenderPipeline = wgpu::RenderPipeline;

    pub struct RenderPipelineBuilder<'a> {
        label: &'a str,
        vertex_shader_module: &'a ShaderModule,
        vertex_entry_point: Option<&'a str>,
        compilation_options: PipelineCompilationOptions<'a>,
        vertex_buffers: Vec<VertexBufferLayout<'a>>,
        primitive: PrimitiveState,
        multisample: MultisampleState,
        layout: Option<PipelineLayout>,
        color_format: TextureFormat,
        depth_stencil: Option<DepthStencilState>,
        fragment_state: Option<FragmentState<'a>>,
        multiview: Option<NonZeroU32>,
    }

    impl<'a> RenderPipelineBuilder<'a> {
        pub fn new(label: &'a str, vertex_shader_module: &'a ShaderModule) -> Self {
            Self {
                vertex_shader_module,
                vertex_entry_point: None,
                label,
                vertex_buffers: Vec::new(),
                compilation_options: Default::default(),
                primitive: PrimitiveState::default(),
                multisample: MultisampleState::default(),
                layout: None,
                color_format: TextureFormat::Rgba8UnormSrgb,
                depth_stencil: None,
                fragment_state: None,
                multiview: None,
            }
        }
        pub fn with_vertex_entry_point(mut self, vertex_entry_point: &'a str) -> Self {
            self.vertex_entry_point = vertex_entry_point.into();
            self
        }
        pub fn with_vertex_buffer(mut self, layout: VertexBufferLayout<'a>) -> Self {
            self.vertex_buffers.push(layout);
            self
        }
        pub fn with_primitive(
            mut self,
            topology: PrimitiveTopology,
            strip_index_format: Option<IndexFormat>,
            front_face: FrontFace,
            cull_mode: Option<Face>,
            unclipped_depth: bool,
            polygon_mode: PolygonMode,
            conservative: bool,
        ) -> Self {
            self.primitive = PrimitiveState {
                topology,
                strip_index_format,
                front_face,
                cull_mode,
                unclipped_depth,
                polygon_mode,
                conservative,
            };
            self
        }
        pub fn with_color_format(mut self, format: wgpu::TextureFormat) -> Self {
            self.color_format = format;
            self
        }
        pub fn with_depth_stencil(
            mut self,
            format: TextureFormat,
            depth_write_enabled: bool,
            depth_compare: CompareFunction,
            stencil: StencilState,
            bias: DepthBiasState,
        ) -> Self {
            self.depth_stencil = Some(DepthStencilState {
                format,
                depth_write_enabled,
                depth_compare,
                stencil,
                bias,
            });
            self
        }
        pub fn with_layout(
            mut self,
            label: &'a str,
            bind_group_layouts: &'a [&'a BindGroupLayout],
            push_constant_ranges: &'a [wgpu::PushConstantRange],
        ) -> Self {
            unsafe {
                if let Some(t) = &DEVICE {
                    self.layout = Some(t.create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: Some(label),
                        bind_group_layouts,
                        push_constant_ranges,
                    }));
                    self
                } else {
                    panic!("create render pipeline must be called after run_renderer task.")
                }
            }
        }
        pub fn with_compilation_options(
            mut self,
            constants: &'a HashMap<String, f64>,
            zero_initialize_workgroup_memory: bool,
        ) -> Self {
            self.compilation_options = PipelineCompilationOptions {
                constants,
                zero_initialize_workgroup_memory,
            };
            self
        }
        pub fn with_multisample(
            mut self,
            count: u32,
            mask: u64,
            alpha_to_coverage_enabled: bool,
        ) -> Self {
            self.multisample = MultisampleState {
                count,
                mask,
                alpha_to_coverage_enabled,
            };
            self
        }
        pub fn with_fragment_state(
            mut self,
            fragment_shader_module: &'a ShaderModule,
            entry_point: &'a str,
        ) -> Self {
            self.fragment_state = Some(FragmentState {
                module: fragment_shader_module,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                targets: &[],
            });
            self
        }
        pub fn with_fragment_compilation_options(
            mut self,
            constants: &'a HashMap<String, f64>,
            zero_initialize_workgroup_memory: bool,
        ) -> Self {
            if let Some(t) = &mut self.fragment_state {
                t.compilation_options = PipelineCompilationOptions {
                    constants,
                    zero_initialize_workgroup_memory,
                }
            };
            self
        }
        pub fn with_fragment_targets(mut self, targets: &'a [Option<ColorTargetState>]) -> Self {
            if let Some(t) = &mut self.fragment_state {
                t.targets = targets
            };
            self
        }
        pub fn with_multiview(mut self, multiview: u32) -> Self {
            if multiview == 0 {
                panic!("multiview cant be 0");
            }
            self.multiview = Some(NonZeroU32::try_from(multiview).unwrap());
            self
        }

        pub fn build(self) -> RenderPipeline {
            unsafe {
                if let Some(t) = &DEVICE {
                    t.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(self.label),
                        layout: self.layout.as_ref(),
                        vertex: wgpu::VertexState {
                            module: self.vertex_shader_module,
                            entry_point: self.vertex_entry_point,
                            compilation_options: self.compilation_options,
                            buffers: &self.vertex_buffers,
                        },
                        primitive: self.primitive,
                        depth_stencil: self.depth_stencil,
                        multisample: self.multisample,
                        fragment: self.fragment_state,
                        multiview: self.multiview,
                        cache: None,
                    })
                } else {
                    panic!("create render pipeline must be called after run_renderer task.")
                }
            }
        }
    }

    /*pub fn create_render_pipeline<'a>(
        window_options: Res<WindowOptions>,
        sh: ShaderModule,
    ) -> RenderPipeline {
        unsafe {
            if let Some(t) = &DEVICE {
                let shader = t.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
                });

                let a = t.create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("UIRenderPipeline"),
                    layout: None,
                    vertex: VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(FragmentState {
                        module: &shader,
                        entry_point: Some("fg_main"),
                        compilation_options: Default::default(),
                        targets: &[Some(ColorTargetState {
                            format: window_options.f_read().config().format,
                            blend: Some(BlendState {
                                color: BlendComponent::REPLACE,
                                alpha: BlendComponent::REPLACE,
                            }),
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                    cache: None,
                });

                a
            } else {
                panic!("create render pipeline must be called after run_renderer task.")
            }
        }
    }*/
}*/
/*pub mod shader {
    pub fn create_shader_module(label: &str, source: &str) -> wgpu::ShaderModule {
        unsafe {
            if let Some(t) = &DEVICE {
                t.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(label),
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                })
            } else {
                panic!("create buffer must be called after run_renderer task.")
            }
        }
    }
}*/
use crate::STATE;
use wgpu::util::DeviceExt;

pub type Buffer = wgpu::Buffer;
pub type BindGroup = wgpu::BindGroup;
pub type BufferUsages = wgpu::BufferUsages;
pub type BufferInitDescriptor<'a> = wgpu::util::BufferInitDescriptor<'a>;
pub type IndexFormat = wgpu::IndexFormat;
pub type BlendState = wgpu::BlendState;
pub type BlendComponent = wgpu::BlendComponent;
pub type ColorWrites = wgpu::ColorWrites;
pub type ShaderModule = wgpu::ShaderModule;
pub type VertexBufferLayout<'a> = wgpu::VertexBufferLayout<'a>;
pub type BufferAddress = wgpu::BufferAddress;
pub type VertexAttribute = wgpu::VertexAttribute;
pub type VertexFormat = wgpu::VertexFormat;
pub type VertexStepMode = wgpu::VertexStepMode;
pub type RenderPipeline = wgpu::RenderPipeline;
pub type RenderPipelineDescriptor<'a> = wgpu::RenderPipelineDescriptor<'a>;
pub type VertexState<'a> = wgpu::VertexState<'a>;
pub type PrimitiveState = wgpu::PrimitiveState;
pub type PrimitiveTopology = wgpu::PrimitiveTopology;
pub type FrontFace = wgpu::FrontFace;
pub type Face = wgpu::Face;
pub type PolygonMode = wgpu::PolygonMode;
pub type FragmentState<'a> = wgpu::FragmentState<'a>;
pub type TextureFormat = wgpu::TextureFormat;
pub type ColorTargetState = wgpu::ColorTargetState;
pub type RenderPassDescriptor<'a> = wgpu::RenderPassDescriptor<'a>;
pub type RenderPassColorAttachment<'a> = wgpu::RenderPassColorAttachment<'a>;
pub type TextureView = wgpu::TextureView;
pub type PipelineLayout = wgpu::PipelineLayout;
pub type PipelineLayoutDescriptor<'a> = wgpu::PipelineLayoutDescriptor<'a>;
pub type BindGroupLayoutDescriptor<'a> = wgpu::BindGroupLayoutDescriptor<'a>;
pub type BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry;
pub type ShaderStage = wgpu::ShaderStages;
pub type BindingType = wgpu::BindingType;
pub type BufferBindingType = wgpu::BufferBindingType;
pub type BindGroupLayout = wgpu::BindGroupLayout;
pub type BindGroupDescriptor<'a> = wgpu::BindGroupDescriptor<'a>;
pub type BindGroupEntry<'a> = wgpu::BindGroupEntry<'a>;
pub type Operations<V> = wgpu::Operations<V>;
pub type LoadOp<V> = wgpu::LoadOp<V>;
pub type Color = wgpu::Color;
pub type StoreOp = wgpu::StoreOp;
pub type BlendFactor = wgpu::BlendFactor;
pub type BlendOperation = wgpu::BlendOperation;

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
            t.config.format
        } else {
            panic!("get_surface_format must be called after run_renderer task.")
        }
    }
}
pub fn get_window_ratio() -> f32 {
    unsafe {
        if let Some(t) = &STATE {
            t.config.width as f32 / t.config.height as f32
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
pub fn write_to_buffer(buffer: &Buffer, offset: BufferAddress, data: &[u8]) {
    unsafe {
        if let Some(t) = &STATE {
            t.queue.write_buffer(buffer, offset, data)
        } else {
            panic!("write_buffer must be called after run_renderer task.")
        }
    }
}
