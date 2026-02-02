struct VSIn {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VSOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0) var atlas_tex: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

@group(1) @binding(0) var<uniform> globals: mat4x4<f32>;

@vertex
fn vs_gui(input: VSIn) -> VSOut {
    var out: VSOut;
    // positions for GUI already in clip space (-1..1)
    out.clip = vec4<f32>(input.pos, 1.0);
    out.uv = input.uv;
    out.color = input.color;
    return out;
}

@vertex
fn vs_world(input: VSIn) -> VSOut {
    var out: VSOut;
    out.clip = globals * vec4<f32>(input.pos, 1.0);
    out.uv = input.uv;
    out.color = input.color;
    return out;
}

@fragment
fn fs_gui(in: VSOut) -> @location(0) vec4<f32> {
    let alpha = textureSample(atlas_tex, atlas_sampler, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

@fragment
fn fs_world(in: VSOut) -> @location(0) vec4<f32> {
    let alpha = textureSample(atlas_tex, atlas_sampler, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}