mod assets_tracker;
mod audio;
mod bsn_asset_with_settings;
mod game;
mod gameobjects;
mod resource_notify;
mod sprite_animation;
mod title;
mod ui;

use bevy::app::AppExit;
use bevy::asset::AssetMetaCheck;
use bevy::ecs::template::template;
use bevy::prelude::*;

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
                    // meta_check: AssetMetaCheck::Never,
                    mode: AssetMode::Processed,
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Dungeon Slime".to_string(),
                        resizable: false,
                        resolution: (1280, 720).into(),
                        fit_canvas_to_parent: true,
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        );

        app.init_state::<AppState>();

        // functional plugins
        app.add_plugins((
            assets_tracker::plugin,
            audio::plugin,
            ui::plugin,
            sprite_animation::plugin,
        ));

        // content plugins
        app.add_plugins((title::plugin, game_plugin));

        app.add_systems(Startup, spawn_camera);

        // app.add_systems(
        //     Update,
        //     (
        //         switch_state,
        //         // #[cfg(feature = "dev")]
        //         // bevy::dev_tools::states::log_transitions::<AppState>,
        //     ),
        // );
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

pub fn change_state_to_game(mut next_state: ResMut<NextState<AppState>>) {
    NextState::set_if_neq(&mut next_state, AppState::Game);
}

pub fn change_state_to_title(mut next_state: ResMut<NextState<AppState>>) {
    NextState::set_if_neq(&mut next_state, AppState::Title);
}
