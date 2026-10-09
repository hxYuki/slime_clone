import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;

struct RegionColorScreenMaterial {
    rects: array<vec4<f32>, 32>,
    color: vec4<f32>,
    rect_count: u32,
    saturation: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> material: RegionColorScreenMaterial;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(screen_texture, texture_sampler, in.uv);

    for (var i = 0u; i < material.rect_count; i = i + 1u) {
        let r = material.rects[i];
        if in.uv.x >= r.x && in.uv.x <= r.z && in.uv.y >= r.y && in.uv.y <= r.w {
            color *= material.color;
            break;
        }
    }

    return color;
}
