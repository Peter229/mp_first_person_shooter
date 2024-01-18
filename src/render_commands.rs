use wgpu::util::DeviceExt;

pub enum RenderCommands {
    Camera([[f32; 4]; 4]),
    Model(ModelRenderCommand),
    SkeletonModel(SkeletonModelRenderCommand),
    Quad(glam::f32::Vec3, glam::f32::Vec3, String),
}

pub struct ModelRenderCommand {
    pub model_matrix: glam::f32::Mat4,
    pub model_name: String,
    pub texture_name: String,
    pub render_transform_index: usize,
}

impl ModelRenderCommand {

    pub fn new(model_matrix: glam::f32::Mat4, model_name: &str, texture_name: &str) -> Self {

        Self { model_matrix, model_name: model_name.to_string(), texture_name: texture_name.to_string(), render_transform_index: 0 }
    }
}

pub struct SkeletonModelRenderCommand {
    pub model_matrix: glam::f32::Mat4,
    pub model_name: String,
    pub texture_name: String,
    pub render_transform_index: usize,
}

impl SkeletonModelRenderCommand {

    pub fn new(model_matrix: glam::f32::Mat4, model_name: &str, texture_name: &str) -> Self {

        Self { model_matrix, model_name: model_name.to_string(), texture_name: texture_name.to_string(), render_transform_index: 0 }
    }
}

pub struct RenderTransform {
    transform: glam::f32::Mat4,
    render_transform_buffer: wgpu::Buffer,
    render_transform_bind_group: wgpu::BindGroup,
}

impl RenderTransform {

    pub fn new(device: &wgpu::Device, transform: &glam::f32::Mat4) -> Self {

        let render_transform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Render transform buffer"),
                contents: bytemuck::cast_slice(&[transform.to_cols_array_2d()]),
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

        let render_transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &vertex_uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: render_transform_buffer.as_entire_binding(),
                }
            ],
            label: Some("Render transform buffer bind group"),
        });

        Self { transform: transform.to_owned(), render_transform_buffer, render_transform_bind_group }
    }

    pub fn update_transform(&mut self, transform: &glam::f32::Mat4) {
        
        self.transform = transform.to_owned();
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {

        &self.render_transform_buffer
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {

        &self.render_transform_bind_group
    }
}