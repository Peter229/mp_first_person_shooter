use winit::window::Window;
use wgpu::util::DeviceExt;
use std::rc::Rc;

use crate::resource_manager;
use crate::texture;
use crate::render_commands::{RenderCommands, RenderTransform};
use crate::gpu_types;

const VERTICES: &[gpu_types::Vertex] = &[
    gpu_types::Vertex { position: [0.0, 0.5, 0.0], texture_coordinates: [1.0, 0.0] },
    gpu_types::Vertex { position: [-0.5, -0.5, 0.0], texture_coordinates: [0.0, 1.0] },
    gpu_types::Vertex { position: [0.5, -0.5, 0.0], texture_coordinates: [0.0, 0.0] },
];

pub struct RenderState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    render_transforms: Vec<RenderTransform>,
}

impl RenderState {

    pub async fn new(window: Window) -> Self {

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } 
                else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter().copied().filter(|f| f.is_srgb()).next().unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        //Textures
        let depth_texture = texture::Texture::create_depth_texture(&device, &config);

        //Camera
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera buffer"),
                contents: bytemuck::cast_slice(&[glam::Mat4::IDENTITY.to_cols_array_2d()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let vertex_uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("Vertex uniform bind group layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &vertex_uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("Vertex uniform bind group"),
        });

        //Shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Static Mesh Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("static_mesh_shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture::Texture::get_texture_bind_group_layout(&device),
                &vertex_uniform_bind_group_layout,
                &vertex_uniform_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[gpu_types::Vertex::desc(),],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let render_transforms = Vec::new();

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            depth_texture,
            camera_buffer,
            camera_bind_group,
            render_transforms,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {

        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config);
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update_transforms(&mut self, render_commands: &Vec<RenderCommands>) {
        //Have to do this before render or else the borrow checker gets mad 
        let mut render_transform_index = 0;
        for render_command in render_commands {
            match render_command {
                RenderCommands::Model(transform, _, _) => {
                    if self.render_transforms.len() <= render_transform_index {
                        self.render_transforms.push(RenderTransform::new(&self.device, transform));
                    }
                    else {
                        self.render_transforms[render_transform_index].update_transform(transform);
                    }
                    render_transform_index += 1;
                },
                _ => (),
            }
        }
    }

    pub fn render(&mut self, render_commands: &Vec<RenderCommands>, resource_manager: &resource_manager::ResourceManager) -> Result<(), wgpu::SurfaceError> {

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.get_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            let mut render_transform_index = 0;

            render_pass.set_pipeline(&self.render_pipeline);
            for render_command in render_commands {
                match render_command {
                    RenderCommands::Camera(matrix) => {
                        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(matrix));
                        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    },
                    RenderCommands::Model(transform, model_name, texture_name) => {
                        let model = resource_manager.get_model(model_name);
                        let texture = resource_manager.get_texture(texture_name);
                        self.queue.write_buffer(self.render_transforms[render_transform_index].get_buffer(), 0, bytemuck::cast_slice(&transform.to_cols_array_2d()));
                        render_pass.set_bind_group(2, self.render_transforms[render_transform_index].get_bind_group(), &[]);
                        render_pass.set_bind_group(0, texture.unwrap().get_bind_group().unwrap(), &[]);
                        render_pass.set_vertex_buffer(0, model.unwrap().get_vertex_buffer().slice(..));
                        render_pass.set_index_buffer(model.unwrap().get_index_buffer().slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..model.unwrap().get_indices_count(), 0, 0..1);

                        render_transform_index += 1;
                    },
                    _ => (),
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}