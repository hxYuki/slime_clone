import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import super::utils::saturation_matrix;

struct SaturateOutScreenMaterial {
    saturation: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> material: SaturateOutScreenMaterial;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(screen_texture, texture_sampler, in.uv);

    return saturation_matrix(material.saturation) * color;
}

