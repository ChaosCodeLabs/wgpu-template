use anyhow::{anyhow, Result};
use wasm_bindgen::prelude::*;
use log::{error, info};
use web_sys::{console::info, HtmlCanvasElement};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, wgt::{CommandEncoderDescriptor, DeviceDescriptor}, BufferAddress, BufferUsages, Color, Features, FragmentState, Limits, MultisampleState, Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, RenderPipelineDescriptor, RequestAdapterOptionsBase, ShaderModuleDescriptor, SurfaceTarget, VertexBufferLayout, VertexState};

#[wasm_bindgen]
pub async fn run(canvas: HtmlCanvasElement) {
    match run_with_result(canvas).await {
        Err(err) => error!("Error occurred: {:?}", err),
        Ok(()) => return,
    }
}

async fn run_with_result(canvas: HtmlCanvasElement) -> Result<()> {
    console_log::init().map_err(|err| anyhow!("Could not init logger: {:?}", err))?;
    info!("Running webgpu!!!");

    let width = canvas.width();
    let height = canvas.height();
    info!("window::width={width}, window::height={height}");

    // gpu context
    info!("Making context variables");
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());
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
    let vertex_data: [[f32; 2]; 3] = [
        [-0.5, -0.5],
        [0.0, 0.5],
        [0.5, -0.5],
    ];
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertex_data),
        usage: BufferUsages::VERTEX,
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
        bind_group_layouts: &[],
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
                    array_stride: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2] // matches @position(0)
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
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..vertex_data.len() as u32, 0..1);
    }
    queue.submit(Some(encoder.finish()));
    frame.present();
    Ok(())
}
