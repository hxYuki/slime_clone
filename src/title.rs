use bevy::prelude::*;

use crate::{
    AppState,
    audio::music,
    state_match::matches,
    ui::{self, despawn_on_exit, options_ui, ui_root_despawn_on_exit},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, SubStates, Default)]
#[source(AppState = AppState::Title)]
pub enum TitleState {
    #[default]
    Main,
    Option,
}

pub fn plugin(app: &mut App) {
    app.add_sub_state::<TitleState>();

    app.add_systems(OnEnter(AppState::Title), title_background.spawn());
    app.add_systems(OnEnter(AppState::Title), title_screens.spawn());

    app.add_systems(
        Update,
        animated_background.run_if(in_state(AppState::Title)),
    );
}

fn title_screens() -> impl Scene {
    bsn! {
        @ui_root_despawn_on_exit(AppState::Title)
        Children [
            @matches(TitleState::Main, ui::title_ui)
            --
            @matches(TitleState::Option, options_ui)
        ]
    }
}

fn title_background() -> impl SceneList {
    bsn_list! {
        Sprite {
            image: "images/background-pattern.png",
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 0.25,
            },
        }
        AnimatedBackground
        despawn_on_exit(AppState::Title)
        --
        @music("audio/music/theme.ogg")
        despawn_on_exit(AppState::Title)
    }
}

pub fn change_title_state_to_main(mut next_state: ResMut<NextState<TitleState>>) {
    next_state.set(TitleState::Main);
}
pub fn change_title_state_to_option(mut next_state: ResMut<NextState<TitleState>>) {
    next_state.set(TitleState::Option);
}

#[derive(Component, Clone, Default)]
struct AnimatedBackground;

fn animated_background(
    mut query: Query<&mut Sprite, With<AnimatedBackground>>,
    time: Res<Time>,
    mut acc: Local<f32>,
) {
    query.iter_mut().for_each(|mut sprite| {
        let view_port_size = Vec2::new(1280.0, 720.0);

        let image_size = view_port_size + Vec2::splat(164.0) * *acc;

        *acc -= time.delta_secs() / 5.0;
        if *acc < 0.0 {
            *acc += 1.0;
        }

        sprite.custom_size = Some(image_size);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{asset::AssetPlugin, scene::ScenePlugin, transform::TransformPlugin};

    #[test]
    fn title_screen_transform_hierarchy_is_complete() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                watch_for_changes_override: Some(false),
                use_asset_processor_override: Some(false),
                ..default()
            },
            ScenePlugin,
            TransformPlugin,
        ));
        let root = app.world_mut().spawn_scene(title_screens()).unwrap().id();
        app.world_mut().flush();

        let world = app.world_mut();
        let mut query = world.query_filtered::<(Entity, &ChildOf), With<GlobalTransform>>();
        let invalid: Vec<_> = query
            .iter(world)
            .filter(|(_, parent)| world.get::<GlobalTransform>(parent.parent()).is_none())
            .map(|(entity, parent)| {
                (
                    entity,
                    parent.parent(),
                    world
                        .get::<bevy::ui::experimental::GhostNode>(entity)
                        .is_some(),
                    world.get::<Node>(parent.parent()).is_some(),
                )
            })
            .collect();
        assert!(
            invalid.is_empty(),
            "B0004: root={root:?}; (child, parent, child is GhostNode, parent is Node): {invalid:?}"
        );
    }
}
