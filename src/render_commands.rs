pub enum RenderCommands {
    Camera([[f32; 4]; 4]),
    Model([[f32; 4]; 4], String, String),
}