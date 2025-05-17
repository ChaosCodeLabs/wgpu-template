use anyhow::{anyhow, Result};
use wasm_bindgen::prelude::*;
use log::{error, info};
use web_sys::{console::info, HtmlCanvasElement};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, wgt::{CommandEncoderDescriptor, DeviceDescriptor}, BackendOptions, Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferAddress, BufferUsages, Color, Features, FragmentState, Limits, MultisampleState, Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, RenderPipelineDescriptor, RequestAdapterOptionsBase, ShaderModuleDescriptor, ShaderStages, SurfaceTarget, VertexBufferLayout, VertexState};

#[wasm_bindgen]
pub async fn run(canvas: HtmlCanvasElement) {
    match run_with_result(canvas).await {
        Err(err) => error!("Error occurred: {:?}", err),
        Ok(()) => return,
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenSize {
    width: f32,
    height: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
}

async fn run_with_result(canvas: HtmlCanvasElement) -> Result<()> {
    console_log::init().map_err(|err| anyhow!("Could not init logger: {:?}", err))?;
    info!("Running webgpu!!!");

    let width = canvas.width();
    let height = canvas.height();
    info!("window::width={width}, window::height={height}");

    // gpu context
    info!("Making context variables");
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::BROWSER_WEBGPU,
        ..Default::default()
    });
    let surface_target = SurfaceTarget::Canvas(canvas);
    let surface = instance
        .create_surface(surface_target)
        .map_err(|err| anyhow!("Failed to use canvas as webgpu surface: {:?}", err))?;
    let adapter = instance.request_adapter(&RequestAdapterOptionsBase {
            power_preference: Default::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .map_err(|err| anyhow!("Failed to use canvas as webgpu surface: {:?}", err))?;
    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            label: Some("WGPU Device request"),
            required_features: Features::default(),
            required_limits: Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
        })
        .await
        .map_err(|err| anyhow!("Failed to create device: {:?}", err))?;
    let config = surface
        .get_default_config(&adapter, width, height)
        .ok_or_else(|| format!("Unable to get default config for surface"))
        .map_err(|err| anyhow!(err))?;
    surface.configure(&device, &config);
    
    // buffers
    info!("Creating vertex buffer");
    let vertex_data: [Vertex; 3] = [
        Vertex { pos: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0]},
        Vertex { pos: [0.0, 0.5, 0.0], color: [0.0, 1.0, 0.0]},
        Vertex { pos: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0]},
    ];
    let screen_size = ScreenSize {
        width: width as f32,
        height: height as f32,
    };
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::bytes_of(&vertex_data),
        usage: BufferUsages::VERTEX,
    });
    let screen_size_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Screen Size uniform buffer"),
        contents: bytemuck::bytes_of(&screen_size),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
    });

    let screen_size_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Screen Size bind group layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None,
            }
        ]
    });

    let screen_size_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Screen Size bind group"),
        layout: &screen_size_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: screen_size_buffer.as_entire_binding()
            }
        ]
    });

    // shaders
    info!("Getting shader code");
    let shader_code = include_str!("./shader/default.wgsl");
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Triangle Shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(shader_code))
    });

    // pipeline
    info!("Creating pipeline");
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[&screen_size_bind_group_layout],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0]; // or Bgra8Unorm

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            buffers: &[
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3] // matches @position(0), and @position(1)
                }
            ],
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[
                Some(swapchain_format.into())
            ],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    // rendering
    info!("Rendering webgpu");
    let frame = surface
        .get_current_texture()
        .map_err(|err| anyhow!("Unable to create frame to render. {:?}", err))?;
    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Command Encoder")
    });
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                Some(
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: wgpu::LoadOp::Clear(Color::BLACK),
                            store: wgpu::StoreOp::Store
                        }
                    }
                )
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &screen_size_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..vertex_data.len() as u32, 0..1);
    }
    queue.submit(Some(encoder.finish()));
    frame.present();
    Ok(())
}
