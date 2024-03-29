use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use winit::window::Window;
use wgpu::util::DeviceExt;

use crate::console::Console;
use crate::quad_renderer;
use crate::resource_manager;
use crate::texture;
use crate::render_commands::*;
use crate::gpu_types;

pub struct RenderState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    static_mesh_render_pipeline: wgpu::RenderPipeline,
    skeleton_mesh_render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    render_transforms: Vec<RenderTransform>,
    quad_renderer: quad_renderer::QuadRenderer,
    console: Rc<RefCell<Console>>,
}

impl RenderState {

    pub async fn new(window: Window, console: Rc<RefCell<Console>>) -> Self {

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
        let static_mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Static Mesh Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("static_mesh_shader.wgsl").into()),
        });

        let skeleton_mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Skeleton Mesh Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("skeleton_mesh_shader.wgsl").into()),
        });

        let static_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture::Texture::get_texture_bind_group_layout(&device),
                &vertex_uniform_bind_group_layout,
                &vertex_uniform_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let skeleton_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture::Texture::get_texture_bind_group_layout(&device),
                &vertex_uniform_bind_group_layout,
                &vertex_uniform_bind_group_layout,
                &vertex_uniform_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let static_mesh_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Static Mesh Render Pipeline"),
            layout: Some(&static_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &static_mesh_shader,
                entry_point: "vs_main",
                buffers: &[gpu_types::Vertex::desc(),],
            },
            fragment: Some(wgpu::FragmentState {
                module: &static_mesh_shader,
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
                front_face: wgpu::FrontFace::Ccw,
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

        let skeleton_mesh_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skeleton Mesh Render Pipeline"),
            layout: Some(&skeleton_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &skeleton_mesh_shader,
                entry_point: "vs_main",
                buffers: &[gpu_types::SkeletonVertex::desc(),],
            },
            fragment: Some(wgpu::FragmentState {
                module: &skeleton_mesh_shader,
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
                front_face: wgpu::FrontFace::Ccw,
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

        //Quad renderer
        let quad_renderer = quad_renderer::QuadRenderer::new(&device, &config.format);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            static_mesh_render_pipeline,
            skeleton_mesh_render_pipeline,
            depth_texture,
            camera_buffer,
            camera_bind_group,
            render_transforms,
            quad_renderer,
            console,
        }
    }

    pub fn get_window(&self) -> &Window {
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

    pub fn get_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
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

    //Do not parallelize this function or the rendering calls, render order is required for transform to stay correct
    pub fn update_transforms(&mut self, render_commands: &mut Vec<RenderCommands>) {
        //Have to do this before render or else the borrow checker gets mad 
        let start = Instant::now();
        self.quad_renderer.clear_triangles();
        let mut render_transform_index = 0;
        for render_command in render_commands {
            match render_command {
                RenderCommands::Model(mrc) => {
                    mrc.render_transform_index = render_transform_index;
                    if self.render_transforms.len() <= render_transform_index {
                        self.render_transforms.push(RenderTransform::new(&self.device, &mrc.model_matrix));
                    }
                    else {
                        self.render_transforms[render_transform_index].update_transform(&mrc.model_matrix);
                    }
                    render_transform_index += 1;
                },
                RenderCommands::SkeletonModel(smrc) => {
                    smrc.render_transform_index = render_transform_index;
                    if self.render_transforms.len() <= render_transform_index {
                        self.render_transforms.push(RenderTransform::new(&self.device, &smrc.model_matrix));
                    }
                    else {
                        self.render_transforms[render_transform_index].update_transform(&smrc.model_matrix);
                    }
                    render_transform_index += 1;
                },
                RenderCommands::Quad(top_left, bottom_right, _) => {
                    self.quad_renderer.render_quad_aspect_corrected(*top_left, *bottom_right);
                },
                _ => (),
            }
        }
        self.render_transforms.drain(render_transform_index..);
        self.quad_renderer.generate_vertex_buffer(&self.device);

        let milli_time = start.elapsed().as_micros() as f32 / 1000.0;
        self.console.borrow_mut().insert_timing("Renderer preprocess", milli_time);
    }

    pub fn render(&mut self, render_commands: &Vec<RenderCommands>, resource_manager: &resource_manager::ResourceManager, renderer: &mut egui_wgpu::Renderer, paint_jobs: &Vec<egui::ClippedPrimitive>, ppp: f32, texture_deltas: &egui::TexturesDelta) -> Result<(), wgpu::SurfaceError> {

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut start = Instant::now();

        //Main scene render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Render Pass"),
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

            render_pass.set_pipeline(&self.static_mesh_render_pipeline);
            //Force do camera first
            for render_command in render_commands {
                match render_command {
                    RenderCommands::Camera(matrix) => {
                        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(matrix));
                        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    },
                    _ => (),
                }
            }
            for render_command in render_commands {
                match render_command {
                    RenderCommands::Model(mrc) => {
                        let model = resource_manager.get_model(&mrc.model_name).unwrap();
                        let texture = resource_manager.get_texture(&mrc.texture_name).unwrap();
                        self.queue.write_buffer(self.render_transforms[mrc.render_transform_index].get_buffer(), 0, bytemuck::cast_slice(&mrc.model_matrix.to_cols_array_2d()));
                        render_pass.set_bind_group(2, self.render_transforms[mrc.render_transform_index].get_bind_group(), &[]);
                        render_pass.set_bind_group(0, texture.get_bind_group().unwrap(), &[]);
                        render_pass.set_vertex_buffer(0, model.get_vertex_buffer().slice(..));
                        render_pass.set_index_buffer(model.get_index_buffer().slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..model.get_indices_count(), 0, 0..1);
                    },
                    _ => (),
                }
            }

            //Skeleton animation pass
            render_pass.set_pipeline(&self.skeleton_mesh_render_pipeline);
            //Force do camera first
            for render_command in render_commands {
                match render_command {
                    RenderCommands::Camera(matrix) => {
                        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(matrix));
                        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    },
                    _ => (),
                }
            }
            for render_command in render_commands {
                match render_command {
                    RenderCommands::SkeletonModel(smrc) => {
                        let model = resource_manager.get_skeleton_model(&smrc.model_name).unwrap();
                        model.write_skeleton_buffer(&self.queue);
                        render_pass.set_bind_group(3, model.get_joints_bind_group(), &[]);
                        let texture = resource_manager.get_texture(&smrc.texture_name).unwrap();
                        self.queue.write_buffer(self.render_transforms[smrc.render_transform_index].get_buffer(), 0, bytemuck::cast_slice(&smrc.model_matrix.to_cols_array_2d()));
                        render_pass.set_bind_group(2, self.render_transforms[smrc.render_transform_index].get_bind_group(), &[]);
                        render_pass.set_bind_group(0, texture.get_bind_group().unwrap(), &[]);
                        render_pass.set_vertex_buffer(0, model.get_vertex_buffer().slice(..));
                        render_pass.set_index_buffer(model.get_index_buffer().slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..model.get_indices_count(), 0, 0..1);
                    },
                    _ => (),
                }
            }
        }

        let mut milli_time = start.elapsed().as_micros() as f32 / 1000.0;
        self.console.borrow_mut().insert_timing("Scene renderer", milli_time);
        start = Instant::now();

        //Render pass for quads, used for crosshair atm
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Quad Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.quad_renderer.get_render_pipeline());
            let mut offset = 0;
            for render_command in render_commands {
                match render_command {
                    RenderCommands::Quad(_, _, texture_name) => {
                        let texture = resource_manager.get_texture(texture_name).unwrap();
                        render_pass.set_bind_group(0, texture.get_bind_group().unwrap(), &[]);
                        render_pass.set_vertex_buffer(0, self.quad_renderer.get_vertex_buffer().slice(..));
                        render_pass.draw(offset..(offset + 6), 0..1);
                        offset += 6;
                    },
                    _ => (),
                }
            }
        }

        milli_time = start.elapsed().as_micros() as f32 / 1000.0;
        self.console.borrow_mut().insert_timing("Quad renderer", milli_time);
        start = Instant::now();

        //EGUI render pass
        {
            let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: ppp,
            };
    
            renderer.update_buffers(&self.device, &self.queue, &mut encoder, &paint_jobs, &screen_descriptor);
    
            for (tex_id, img_delta) in &texture_deltas.set {
                renderer.update_texture(&self.device, &self.queue, *tex_id, img_delta);
            }
    
            for tex_id in &texture_deltas.free {
                renderer.free_texture(tex_id);
            }

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("EGUI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            renderer.render(&mut render_pass, paint_jobs, &screen_descriptor);
        }

        milli_time = start.elapsed().as_micros() as f32 / 1000.0;
        self.console.borrow_mut().insert_timing("EGUI renderer", milli_time);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}