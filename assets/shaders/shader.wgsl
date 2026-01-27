// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    fog_start: f32,
    fog_end: f32,
    _pad: vec2<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) view_dist: f32,
}

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vertex.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(vertex.position, 1.0);
    out.view_dist = distance(vertex.position, camera.camera_pos.xyz);
    return out;
}
// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // World-space distance based fog.
    let fog_factor = clamp(
        (in.view_dist - camera.fog_start) / (camera.fog_end - camera.fog_start),
        0.0,
        1.0,
    );
    // Цвет тумана совпадает с цветом неба/clear (см. renderer), чтобы шов не выделялся.
    let fog_color = vec3<f32>(0.10, 0.20, 0.30);
    return vec4<f32>(mix(base_color.rgb, fog_color, fog_factor), base_color.a);
}
 
