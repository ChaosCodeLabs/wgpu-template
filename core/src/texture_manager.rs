use image::GenericImageView;
use log::info;
use wasm_bindgen::{JsCast, JsError, JsValue};
use web_sys::js_sys::{Array, Uint8Array};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    Extent3d, FilterMode, Queue, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
    Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};

pub struct TextureHolder {
    pub texture: Texture,
    pub texture_view: TextureView,
}

pub struct TextureManager {
    pub textures: Vec<TextureHolder>,
    pub sampler: Sampler,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl TextureManager {
    pub fn new(device: &Device, queue: &Queue, textures_data: Array) -> Result<Self, JsError> {
        let mut textures = Vec::new();

        // Include predefined texture
        // let predefined = include_bytes!("./textures/download20250505175626.png");
        // textures.push(Self::create_texture(device, queue, predefined)?);

        // Load JS-supplied textures
        for js_value in textures_data.iter() {
            if js_value.is_null() || js_value.is_undefined() {
                continue;
            }

            let u8_array = Uint8Array::new(&js_value);
            let mut buffer = vec![0u8; u8_array.length() as usize];
            u8_array.copy_to(&mut buffer[..]);

            let texture = Self::create_texture(device, queue, &buffer)?;
            textures.push(texture);
        }

        // Create sampler
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

        // Create bind group layout
        let mut layout_entries: Vec<BindGroupLayoutEntry> = vec![];
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: if textures.is_empty() {
                &layout_entries
            } else {
                layout_entries.push(
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    }
                );
                layout_entries.extend(
                    (0..textures.len()).map(|index| {
                        BindGroupLayoutEntry {
                            binding: 1 + index as u32,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                view_dimension: TextureViewDimension::D2,
                                sample_type: TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        }
                    })
                );
                &layout_entries
            }
        });

        // Create bind group using first texture as representative
        let mut entries: Vec<BindGroupEntry> = vec![];
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &bind_group_layout,
            entries: if textures.is_empty() {
                &entries
            } else {
                entries.push(
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Sampler(&sampler),
                    }
                );
                entries.extend(
                    textures.iter().enumerate().map(|(i, texture)| BindGroupEntry {
                        binding: 1 + i as u32, // binding index increases for each texture
                        resource: BindingResource::TextureView(&texture.texture_view),
                    })
                );
                &entries
            }
        });

        Ok(Self {
            textures,
            sampler,
            bind_group_layout,
            bind_group,
        })
    }

    fn create_texture(device: &Device, queue: &Queue, data: &[u8]) -> Result<TextureHolder, JsError> {
        let img = image::load_from_memory(data)
            .map_err(|e| JsError::new(&format!("Failed to load image: {e}")))?;

        let rgba = img.to_rgba8();
        let (width, height) = img.dimensions();
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Texture"),
            size,
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
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&TextureViewDescriptor::default());

        Ok(TextureHolder {
            texture,
            texture_view: view,
        })
    }
}
