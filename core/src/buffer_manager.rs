use log::info;
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
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ProgramUniform {
    pub screen_width: f32,
    pub screen_height: f32,
}

// uniforms
pub struct UniformManager {
    pub program_uniform_data: ProgramUniform,
    pub program_uniform_buffer: Buffer,
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
            label: Some("Screen Size uniform buffer"),
            contents: bytemuck::bytes_of(&program_uniform_data),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let program_uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Program uniforms bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let program_uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Program uniforms bind group"),
            layout: &program_uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: program_uniform_buffer.as_entire_binding(),
            }],
        });
        Self {
            program_uniform_data,
            program_uniform_buffer,
            bind_group_layout: program_uniform_bind_group_layout,
            bind_group: program_uniform_bind_group,
        }
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
            },
            Vertex {
                pos: [-1.0, 1.0, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                pos: [1.0, 1.0, 0.0],
                color: [0.0, 0.0, 1.0],
            },
            Vertex {
                pos: [1.0, -1.0, 0.0],
                color: [1.0, 1.0, 1.0],
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
