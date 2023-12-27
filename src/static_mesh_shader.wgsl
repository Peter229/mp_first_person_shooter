@group(1) @binding(0)
var<uniform> camera_view_projection: mat4x4<f32>;
@group(2) @binding(0)
var<uniform> model_transform: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texture_coordinates: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) fragment_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texture_coordinates: vec2<f32>,
};

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera_view_projection * model_transform * vec4<f32>(vertex.position, 1.0);
    out.fragment_position = vec3<f32>((model_transform * vec4<f32>(vertex.position, 1.0)).xyz);
    //Need to change this to the mat3x3(transpose(inverse(model))) but inverse needs to be done on the cpu and uploaded by uniform, this is to handle non uniform scale
    out.normal = vec3<f32>((model_transform * vec4<f32>(vertex.normal, 0.0)).xyz);
    out.texture_coordinates = vertex.texture_coordinates;
    return out;
}

@group(0) @binding(0)
var diffuse_texture: texture_2d<f32>;
@group(0) @binding(1)
var diffuse_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let sun_direction = normalize(vec3<f32>(1.0, 1.0, -1.0));
    let sun_strength = max(dot(sun_direction, in.normal), 0.1);
    let sun_colour = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let sun_light = sun_colour * sun_strength;

    return textureSample(diffuse_texture, diffuse_sampler, in.texture_coordinates) * sun_light;
}