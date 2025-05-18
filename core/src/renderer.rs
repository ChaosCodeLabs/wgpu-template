use log::info;
use wasm_bindgen::JsError;
use wgpu::{Color, CommandEncoderDescriptor, Operations};

use crate::{
    buffer_manager::BufferManager, gpu_context::GpuContext, pipeline_manager::PipelineManager,
};

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        info!("Renderer created successfully!");
        Self {}
    }

    pub fn render(
        &self,
        gpu: &GpuContext,
        buffers: &BufferManager,
        pipeline: &PipelineManager,
    ) -> Result<(), JsError> {
        info!("Rendering webgpu");
        let frame = gpu
            .surface
            .get_current_texture()
            .map_err(|err| JsError::new(&format!("Unable to create frame to render. {:?}", err)))?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });
        // render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&pipeline.pipeline);
            render_pass.set_bind_group(0, &buffers.uniform_manager.bind_group, &[]);
            render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..buffers.index_length as u32, 0, 0..1);
        }
        gpu.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}
