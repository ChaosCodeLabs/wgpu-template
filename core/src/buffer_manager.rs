use log::info;
use serde::Deserialize;
use web_sys::js_sys::Math::abs;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferUsages, Device, ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
    tex_pos: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MousePos {
    x: f32,
    y: f32,
}
impl MousePos {
    fn zero() -> Self {
        MousePos { x: 0.0, y: 0.0 }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ProgramUniform {
    screen_width: f32,
    screen_height: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PerFrameUniform {
    time: f32,
    delta_time: f32,
    mouse_pos: MousePos,
}

// uniforms
pub struct UniformManager {
    pub program_uniform_data: ProgramUniform,
    pub program_uniform_buffer: Buffer,
    pub per_frame_uniform_data: PerFrameUniform,
    pub per_frame_uniform_buffer: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl UniformManager {
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        let program_uniform_data = ProgramUniform {
            screen_width: width as f32,
            screen_height: height as f32,
        };
        let program_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Program uniform buffer"),
            contents: bytemuck::bytes_of(&program_uniform_data),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let per_frame_uniform_data = PerFrameUniform {
            time: 0.0,
            delta_time: 0.0,
            mouse_pos: MousePos::zero(),
        };
        let per_frame_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Per Frame uniform buffer"),
            contents: bytemuck::bytes_of(&per_frame_uniform_data),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Program uniforms bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Program uniforms bind group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: program_uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: per_frame_uniform_buffer.as_entire_binding(),
                },
            ],
        });
        Self {
            program_uniform_data,
            program_uniform_buffer,
            per_frame_uniform_data,
            per_frame_uniform_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(
        &mut self,
        time: Option<f32>,
        delta_time: Option<f32>,
        mouse_pos: Option<MousePos>,
    ) {
        if time.is_none() && delta_time.is_none() && mouse_pos.is_none() {
            return;
        }
        let per_frame_uniform_data = PerFrameUniform {
            time: time.unwrap_or(self.per_frame_uniform_data.time),
            delta_time: delta_time.unwrap_or(self.per_frame_uniform_data.delta_time),
            mouse_pos: if let Some(mouse_pos) = mouse_pos {
                MousePos {
                    x: f32::min(
                        f32::max(mouse_pos.x, 0.0),
                        self.program_uniform_data.screen_width,
                    ),
                    y: f32::min(
                        f32::max(mouse_pos.y, 0.0),
                        self.program_uniform_data.screen_height,
                    ),
                }
            } else {
                self.per_frame_uniform_data.mouse_pos
            },
        };
        self.per_frame_uniform_data = per_frame_uniform_data;
    }
}

// Buffers
pub struct BufferManager {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_length: u32,
    pub uniform_manager: UniformManager,
}

impl BufferManager {
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        info!("Creating vertex buffer");
        let vertex_data: [Vertex; 4] = [
            Vertex {
                pos: [-1.0, -1.0, 0.0],
                color: [1.0, 0.0, 0.0],
                tex_pos: [0.0, 1.0],
            },
            Vertex {
                pos: [-1.0, 1.0, 0.0],
                color: [0.0, 1.0, 0.0],
                tex_pos: [0.0, 0.0],
            },
            Vertex {
                pos: [1.0, 1.0, 0.0],
                color: [0.0, 0.0, 1.0],
                tex_pos: [1.0, 0.0],
            },
            Vertex {
                pos: [1.0, -1.0, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_pos: [1.0, 1.0],
            },
        ];
        let index_data: [[u16; 3]; 2] = [[0, 1, 2], [0, 2, 3]];
        let index_length = index_data.len() as u32 * 3; // flattened size
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::bytes_of(&vertex_data),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::bytes_of(&index_data),
            usage: BufferUsages::INDEX,
        });
        let uniform_manager = UniformManager::new(&device, width, height);
        Self {
            vertex_buffer,
            index_buffer,
            index_length,
            uniform_manager,
        }
    }
}
