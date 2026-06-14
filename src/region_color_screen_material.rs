use bevy::{
    core_pipeline::{
        self,
        fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
    },
    ecs::template::FromTemplate,
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
    shader::ShaderRef,
    ui::{
        ComputedNode, ComputedUiRenderTargetInfo, ComputedUiTargetCamera, UiGlobalTransform,
        UiSystems,
    },
    ui_render,
};

const MAX_GREYOUT_RECTS: usize = 32;

pub fn plugins(app: &mut App) {
    app.add_plugins(FullscreenMaterialPlugin::<RegionColorScreenMaterial>::default());

    app.add_systems(
        PostUpdate,
        sync_greyout_regions_to_screen_material.after(UiSystems::Layout),
    );
}

fn sync_greyout_regions_to_screen_material(
    mut cameras: Query<(
        Entity,
        &Camera,
        Option<&RegionColor>,
        &mut RegionColorScreenMaterial,
    )>,
    greyouts: Query<(
        &ComputedNode,
        &UiGlobalTransform,
        &ComputedUiTargetCamera,
        &ComputedUiRenderTargetInfo,
        &InheritedVisibility,
        &UiGreyout,
    )>,
) {
    for (camera_entity, camera, region_color, mut screen_material) in cameras.iter_mut() {
        screen_material.rects = [Vec4::ZERO; MAX_GREYOUT_RECTS];
        screen_material.color = region_color
            .map(|region_color| region_color.color.to_linear().to_vec4())
            .unwrap_or(Vec4::ONE);
        screen_material.rect_count = 0;

        let mut truncated = false;

        for (node, transform, target_camera, target_info, visibility, greyout) in greyouts.iter() {
            if screen_material.rect_count as usize >= MAX_GREYOUT_RECTS {
                truncated = true;
                break;
            }

            if !greyout.enabled
                || !visibility.get()
                || node.is_empty()
                || target_camera.get() != Some(camera_entity)
            {
                continue;
            }

            let Some(uv_rect) = ui_node_uv_rect(node, transform, target_info, camera) else {
                continue;
            };

            let rect_index = screen_material.rect_count as usize;
            screen_material.rects[rect_index] = uv_rect;
            screen_material.rect_count += 1;
        }

        if truncated {
            warn!(
                "More than {MAX_GREYOUT_RECTS} UiGreyout regions target one camera; extra regions were ignored"
            );
        }
    }
}

fn ui_node_uv_rect(
    node: &ComputedNode,
    transform: &UiGlobalTransform,
    target_info: &ComputedUiRenderTargetInfo,
    camera: &Camera,
) -> Option<Vec4> {
    let mut target_size = target_info.physical_size().as_vec2();
    if target_size.cmple(Vec2::ZERO).any() {
        target_size = camera.physical_viewport_size()?.as_vec2();
    }

    if target_size.cmple(Vec2::ZERO).any() {
        return None;
    }

    let rect = Rect::from_center_size(transform.to_scale_angle_translation().2, node.size());
    let min = (rect.min / target_size).clamp(Vec2::ZERO, Vec2::ONE);
    let max = (rect.max / target_size).clamp(Vec2::ZERO, Vec2::ONE);

    if min.x >= max.x || min.y >= max.y {
        return None;
    }

    Some(Vec4::new(min.x, min.y, max.x, max.y))
}

#[derive(Component, Debug, Clone, Copy, FromTemplate)]
pub struct UiGreyout {
    pub enabled: bool,
}

impl Default for UiGreyout {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Component, Clone, Copy)]
#[component(immutable)]
#[require(RegionColorScreenMaterial)]
pub struct RegionColor {
    pub color: Color,
}

impl Default for RegionColor {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
        }
    }
}

#[derive(Component, Clone, Copy, Default, ExtractComponent, ShaderType)]
pub struct RegionColorScreenMaterial {
    rects: [Vec4; MAX_GREYOUT_RECTS],
    color: Vec4,
    rect_count: u32,
    _padding: Vec3,
}

impl FullscreenMaterial for RegionColorScreenMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/region_color_screen_material.wgsl".into()
    }

    fn schedule() -> impl bevy::ecs::schedule::ScheduleLabel + Clone {
        bevy::core_pipeline::Core2d
    }

    fn schedule_configs(
        system: bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::BoxedSystem>,
    ) -> bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::BoxedSystem> {
        system
            .after(ui_render::ui_pass)
            .before(core_pipeline::upscaling::upscaling)
    }
}
