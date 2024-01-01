use wgpu::util::DeviceExt;

use crate::gpu_types;
use crate::gpu_types::QuadVertex;
use crate::texture;

pub struct QuadRenderer {
    render_pipeline: wgpu::RenderPipeline,
    triangles: Vec<gpu_types::QuadVertex>,
    vertex_buffer: wgpu::Buffer,
}

impl QuadRenderer {
    pub fn new(device: &wgpu::Device, format: &wgpu::TextureFormat) -> Self {

        //Shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("quad_shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture::Texture::get_texture_bind_group_layout(&device),
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[gpu_types::QuadVertex::desc(),],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: *format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let triangles = Vec::new();

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: bytemuck::cast_slice(&triangles),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        Self { render_pipeline, triangles, vertex_buffer }
    }

    pub fn get_render_pipeline(&self) -> &wgpu::RenderPipeline {

        &self.render_pipeline
    }

    pub fn render_quad(&mut self, top_left: glam::f32::Vec3, bottom_right: glam::f32::Vec3) {

        self.triangles.push(gpu_types::QuadVertex { position: top_left.into(), texture_coordinates: [0.0, 1.0]});
        self.triangles.push(gpu_types::QuadVertex { position: bottom_right.into() , texture_coordinates: [1.0, 0.0]});
        self.triangles.push(gpu_types::QuadVertex { position: [bottom_right.x, top_left.y, top_left.z], texture_coordinates: [1.0, 1.0]});

        self.triangles.push(gpu_types::QuadVertex { position: top_left.into(), texture_coordinates: [0.0, 1.0]});
        self.triangles.push(gpu_types::QuadVertex { position: [top_left.x, bottom_right.y, top_left.z], texture_coordinates: [0.0, 0.0]});
        self.triangles.push(gpu_types::QuadVertex { position: bottom_right.into(), texture_coordinates: [1.0, 0.0]});
    }

    pub fn render_quad_aspect_corrected(&mut self, top_left: glam::f32::Vec3, bottom_right: glam::f32::Vec3) {

        let mut aspect_corrected_top_left = top_left;
        let mut aspect_corrected_bottom_right = bottom_right;

        aspect_corrected_top_left.x = top_left.x * 0.9;
        aspect_corrected_bottom_right.x = bottom_right.x * 0.9;
        aspect_corrected_top_left.y = top_left.y * 1.6;
        aspect_corrected_bottom_right.y = bottom_right.y * 1.6;

        self.render_quad(aspect_corrected_top_left, aspect_corrected_bottom_right);
    }

    pub fn generate_vertex_buffer(&mut self, device: &wgpu::Device) {

        self.vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: bytemuck::cast_slice(&self.triangles),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
    }

    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {

        &self.vertex_buffer
    }

    pub fn clear_triangles(&mut self) {

        self.triangles.clear();
    }
}