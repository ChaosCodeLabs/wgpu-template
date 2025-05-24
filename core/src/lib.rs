mod buffer_manager;
mod gpu_context;
mod pipeline_manager;
mod renderer;
mod texture_manager;

use buffer_manager::{BufferManager, MousePos};
use gpu_context::GpuContext;
use log::info;
use pipeline_manager::PipelineManager;
use renderer::Renderer;
use texture_manager::TextureManager;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, js_sys::Array};
use wgpu::wgc::device::queue;

#[wasm_bindgen]
pub struct App {
    gpu: GpuContext<'static>,
    buffers: BufferManager,
    textures: TextureManager,
    pipeline: PipelineManager,
    renderer: Renderer,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen]
    pub async fn setup(canvas: HtmlCanvasElement, textures_data: JsValue) -> Result<App, JsError> {
        console_log::init()
            .map_err(|err| JsError::new(&format!("Could not init logger: {:?}", err)))?;
        info!("Setting up webgpu!!!");

        let width = canvas.width();
        let height = canvas.height();
        info!("window::width={width}, window::height={height}");

        let gpu = GpuContext::new(canvas, width, height).await?;
        let buffer_manager = BufferManager::new(&gpu.device, width, height);
        let texture_manager = TextureManager::new(&gpu.device, &gpu.queue, Array::from(&textures_data))?;
        let swapchain_capabilities = gpu.surface.get_capabilities(&gpu.adapter);
        let swapchain_format = swapchain_capabilities.formats[0]; // should be Bgra8Unorm generally
        let pipeline_manager = PipelineManager::new(
            &gpu.device,
            swapchain_format,
            &buffer_manager,
            &texture_manager,
        );
        let renderer = Renderer::new();
        Ok(App {
            gpu,
            buffers: buffer_manager,
            textures: texture_manager,
            pipeline: pipeline_manager,
            renderer,
        })
    }

    #[wasm_bindgen]
    pub fn update(
        &mut self,
        time: Option<f32>,
        delta_time: Option<f32>,
        mouse: JsValue,
    ) -> Result<(), JsError> {
        let mouse_pos = if !mouse.is_null() && !mouse.is_undefined() {
            serde_wasm_bindgen::from_value::<MousePos>(mouse).ok()
        } else {
            None
        };
        self.buffers
            .uniform_manager
            .update(time, delta_time, mouse_pos);
        self.gpu.queue.write_buffer(
            &self.buffers.uniform_manager.per_frame_uniform_buffer,
            0,
            bytemuck::bytes_of(&self.buffers.uniform_manager.per_frame_uniform_data),
        );
        Ok(())
    }

    #[wasm_bindgen]
    pub fn render(&self) -> Result<(), JsError> {
        self.renderer
            .render(&self.gpu, &self.buffers, &self.textures, &self.pipeline)
    }
}
