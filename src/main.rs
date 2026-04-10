mod assets_tracker;
mod audio;
mod game;
mod gameobjects;
mod sprite_animation;
mod ui;

use bevy::app::AppExit;
use bevy::asset::AssetMetaCheck;
use bevy::ecs::template::template;
use bevy::prelude::*;
use bevy::scene2::{CommandsSceneExt, bsn};

use crate::game::game_plugin;
use crate::ui::ui_root_despawn_on_exit;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Title,
    Game,
}

pub struct AppPlugin;
impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Dungeon Slime".to_string(),
                        resizable: false,
                        resolution: (1280, 720).into(),
                        fit_canvas_to_parent: true,
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        );

        app.init_state::<AppState>();

        app.add_plugins((assets_tracker::plugin, audio::plugin));

        app.add_plugins((game_plugin, sprite_animation::plugin));

        app.add_systems(Startup, spawn_camera);

        app.add_systems(OnEnter(AppState::Title), setup_title);

        app.add_systems(
            Update,
            (
                switch_state,
                // #[cfg(feature = "dev")]
                // bevy::dev_tools::states::log_transitions::<AppState>,
            ),
        );
    }
}

fn spawn_camera(mut command: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 0.25;
    command.spawn((
        Name::new("Camera"),
        Camera2d,
        Projection::Orthographic(projection),
    ));
}

fn setup_title(mut command: Commands) {
    command.spawn_scene(bsn! {
        ui_root_despawn_on_exit(AppState::Title)

        Children [
            ui::title_ui()
        ]
    });
}

fn switch_state(
    mut next_state: ResMut<NextState<AppState>>,
    input: Res<ButtonInput<KeyCode>>,
    gamepads: Query<(Entity, &Gamepad)>,
) {
    gamepads.iter().take(1).for_each(|(entity, gamepad)| {
        if gamepad.just_pressed(GamepadButton::South) {
            NextState::set_if_neq(&mut next_state, AppState::Game);
        }

        if gamepad.just_pressed(GamepadButton::Start) {
            NextState::set_if_neq(&mut next_state, AppState::Title);
        }
    });

    if input.just_pressed(KeyCode::Enter) {
        NextState::set_if_neq(&mut next_state, AppState::Game);
        // next_state.set(AppState::Game);
    }

    if input.just_pressed(KeyCode::Escape) {
        NextState::set_if_neq(&mut next_state, AppState::Title);
    }
}
