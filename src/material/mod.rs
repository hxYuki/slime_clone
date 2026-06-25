pub mod saturate_out_screen_material;

pub mod fullscreen_matertial_prelude {
    pub use bevy::{
        core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
        prelude::*,
        render::{extract_component::ExtractComponent, render_resource::ShaderType},
        shader::ShaderRef,
    };
}
