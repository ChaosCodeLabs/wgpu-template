use std::path::Path;

use image::GenericImageView;
use log::info;
use wasm_bindgen::{JsCast, JsError, JsValue};
use web_sys::js_sys::{Array, Uint8Array};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    Extent3d, FilterMode, Queue, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
    TexelCopyBufferLayout, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDimension, hal::BindGroupLayoutFlags,
};

pub struct TextureHolder {
    pub texture: Texture,
    pub texture_view: TextureView,
}
pub struct TextureManager {
    pub textures: Vec<TextureHolder>,
    pub samplers: Vec<Sampler>,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl TextureManager {
    pub fn new(device: &Device, queue: &Queue, textures_data: JsValue) -> Result<Self, JsError> {
        let texture_buffers = Self::load_texture_buffers(textures_data).map_err(|err| {
            JsError::new(&format!("Error Occured while loading textures: {:?}", err))
        })?;
        let textures = texture_buffers
            .iter()
            .map(|tex_buffer: &Vec<u8>| Self::create_texture(device, queue, tex_buffer))
            .collect::<Result<Vec<_>, JsError>>()?;
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                // sampler
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // texture(s)
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&textures[0].texture_view),
                },
            ],
        });

        Ok(Self {
            textures,
            samplers: vec![sampler],
            bind_group_layout,
            bind_group,
        })
    }

    pub fn create_texture(
        device: &Device,
        queue: &Queue,
        tex_buffer: &Vec<u8>,
    ) -> Result<TextureHolder, JsError> {
        let img = image::load_from_memory(&tex_buffer)
            .map_err(|err| JsError::new(&format!("Problem occured reading texture: {:?}", err)))?;
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();
        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            // all textures stored as 3d, we just present our 2d
            // texture by setting depth 1
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Texture"),
            size: size.clone(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            &rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(TextureHolder {
            texture,
            texture_view: view,
        })
    }

    fn load_texture_buffers(data: JsValue) -> Result<Vec<Vec<u8>>, JsError> {
        // Convert the JsValue to a js_sys::Array
        let array = Array::from(&data);

        let mut textures: Vec<Vec<u8>> = Vec::new();

        for i in 0..array.length() {
            let element = array.get(i);

            // Ensure the element is a Uint8Array
            if let Ok(typed_array) = element.dyn_into::<Uint8Array>() {
                // Convert Uint8Array to a Vec<u8>
                let mut texture_data = vec![0; typed_array.length() as usize];
                typed_array.copy_to(&mut texture_data[..]);
                textures.push(texture_data);

                info!("Texture {} length: {}", i, typed_array.length() as usize);
            } else {
                return Err(JsError::new(&format!(
                    "Element at index {} is not a Uint8Array",
                    i
                )));
            }
        }
        return Ok(textures);
    }
}
