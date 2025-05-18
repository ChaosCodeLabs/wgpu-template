use log::info;
use wasm_bindgen::JsError;
use web_sys::HtmlCanvasElement;
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, Instance, Limits, Queue,
    RequestAdapterOptions, Surface, SurfaceTarget,
};

pub struct GpuContext<'window> {
    pub instance: Instance,
    pub surface: Surface<'window>,
    pub device: Device,
    pub queue: Queue,
    pub adapter: Adapter,
}

impl GpuContext<'_> {
    pub async fn new(canvas: HtmlCanvasElement, width: u32, height: u32) -> Result<Self, JsError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::BROWSER_WEBGPU,
            ..Default::default()
        });
        let surface_target = SurfaceTarget::Canvas(canvas);
        let surface = instance.create_surface(surface_target).map_err(|err| {
            JsError::new(&format!(
                "Failed to use canvas as webgpu surface: {:?}",
                err
            ))
        })?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: Default::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|err| {
                JsError::new(&format!(
                    "Failed to use canvas as webgpu surface: {:?}",
                    err
                ))
            })?;
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("WGPU Device request"),
                required_features: Features::default(),
                required_limits: Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|err| JsError::new(&format!("Failed to create device: {:?}", err)))?;
        let config = surface
            .get_default_config(&adapter, width, height)
            .ok_or_else(|| "Unable to get default config for surface")
            .map_err(|err| JsError::new(err))?;
        surface.configure(&device, &config);
        info!("WebGpu initialized successfully");
        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
        })
    }
}
