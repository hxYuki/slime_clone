use super::fullscreen_matertial_prelude::*;

pub fn plugins(app: &mut App) {
    app.add_plugins(FullscreenMaterialPlugin::<SaturateOutScreenMaterial>::default());
}

#[derive(Component, Clone, Copy, Default, ExtractComponent, ShaderType)]
pub struct SaturateOutScreenMaterial {
    pub saturation: f32,
    _padding: Vec3,
}
impl SaturateOutScreenMaterial {
    pub fn new(saturation: f32) -> Self {
        Self {
            saturation,
            _padding: Vec3::ZERO,
        }
    }
}

impl FullscreenMaterial for SaturateOutScreenMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/saturate_out.wesl".into()
    }

    fn schedule() -> impl bevy::ecs::schedule::ScheduleLabel + Clone {
        bevy::core_pipeline::Core2d
    }

    fn schedule_configs(
        system: bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::BoxedSystem>,
    ) -> bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::BoxedSystem> {
        system
            .in_set(bevy::core_pipeline::Core2dSystems::PostProcess)
            .before(bevy::core_pipeline::tonemapping::tonemapping)
    }
}
