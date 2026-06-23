use std::sync::Arc;

use wgpu::{BufferUsages, util::DeviceExt};
use winit::window::Window;

use crate::nes::NES_PALETTE;

pub struct Renderer {
    pub window:         Arc<Window>,
    pub surface:        wgpu::Surface<'static>,
    pub device:         wgpu::Device,
    pub queue:          wgpu::Queue,
    pub config:         wgpu::SurfaceConfiguration,

    pub bind_group:     wgpu::BindGroup,
    pub pipeline:       wgpu::ComputePipeline,

    pub framebuffer:    wgpu::Buffer,
    pub out_tex:        wgpu::Texture
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Renderer {
        use futures::executor::block_on;
        
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = block_on(instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }))
            .unwrap();

        let (device, queue) = block_on(adapter
            .request_device(&wgpu::DeviceDescriptor::default()))
            .unwrap();

        let caps = surface.get_capabilities(&adapter);

        let format = caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("NES Rendering Shader"),

            source: wgpu::ShaderSource::Wgsl(include_str!("shader/render.wgsl").into())
        });

        let framebuffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("NES Framebuffer Bind Group"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                    },
                    count: None,
                }
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("NES Compute Pipeline Layout"),
            bind_group_layouts: &[Some(&framebuffer_bind_group_layout)],
            immediate_size: 0
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("NES Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None
        });

        let framebuffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("NES Framebuffer"),
            size: 61440,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let palette_vec_form: Vec<_> = NES_PALETTE.iter().map(|(r, g, b)| [*r as f32 / 255., *g as f32 / 255., *b as f32 / 255., 1.]).collect();
        let palette_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NES Palette"),
            contents: bytemuck::cast_slice(&palette_vec_form),
            usage: BufferUsages::UNIFORM,
        });

        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Renderer Output Image"),
            size: wgpu::Extent3d{ width: 256, height: 240, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("NES Framebuffer Bind Group"),
            layout: &framebuffer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: framebuffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: palette_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&output_view)
                }
            ],
        });
        
        Self { 
            window, 
            surface, 
            device, 
            queue, 
            config, 
            bind_group, 
            out_tex: output_texture, 
            framebuffer, 
            pipeline
        }
    }

    pub fn upload_framebuffer(&mut self, framebuffer: &[u8; 256 * 240]) {
        let mut packed = [0; 256 * 240 / 4];

        for i in 0..15360 {
            packed[i] =
                (framebuffer[i * 4] as u32)             |
                ((framebuffer[i * 4 + 1] as u32) << 8)  |
                ((framebuffer[i * 4 + 2] as u32) << 16) |
                ((framebuffer[i * 4 + 3] as u32) << 24);
        }

        self.queue.write_buffer(&self.framebuffer, 0, bytemuck::cast_slice(&packed));
    }

    pub fn render(&mut self) {
        use wgpu::CurrentSurfaceTexture;

        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                self.config.width = self.window.inner_size().width;
                self.config.height = self.window.inner_size().height;

                self.surface.configure(&self.device, &self.config);

                surface_texture
            }

            _ => return
        };

        let mut encoder =
        self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            }
        );

        {
            let mut compute_pass = encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor{
                label: Some("NES Compute Pass"),
                timestamp_writes: None,
            });
            
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(64, 240, 1);
        }

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.out_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &frame.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: 256,
                height: 240,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

}