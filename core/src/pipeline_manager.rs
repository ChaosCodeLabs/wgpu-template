use log::info;
use wgpu::{
    BufferAddress, Device, FragmentState, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, TextureFormat, VertexBufferLayout, VertexState,
};

use crate::buffer_manager::{BufferManager, Vertex};

pub struct PipelineManager {
    pub pipeline: RenderPipeline,
}

impl PipelineManager {
    pub fn new(device: &Device, swapchain_format: TextureFormat, buffers: &BufferManager) -> Self {
        // shaders
        info!("Getting shader code");
        let shader_code = include_str!("./shader/default.wgsl");
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Default Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(shader_code)),
        });

        // pipeline
        info!("Creating pipeline");
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&buffers.uniform_manager.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        // matches @position(0), and @position(1)
                        0 => Float32x3,
                        1 => Float32x3
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        info!("Pipeline created successfully!!!");
        Self { pipeline }
    }
}
