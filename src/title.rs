use bevy::prelude::*;

use crate::{
    AppState,
    audio::music,
    ui::{self, options_ui, ui_root_despawn_on_exit},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, SubStates, Default)]
#[source(AppState = AppState::Title)]
#[states(scoped_entities)]
pub enum TitleState {
    #[default]
    Main,
    Option,
}

pub fn plugin(app: &mut App) {
    app.add_sub_state::<TitleState>();

    app.add_systems(OnEnter(AppState::Title), title_background.spawn());
    app.add_systems(OnEnter(TitleState::Main), setup_title);
    app.add_systems(OnEnter(TitleState::Option), options_ui.spawn());

    app.add_systems(
        Update,
        animated_background.run_if(in_state(AppState::Title)),
    );
}

fn setup_title(mut command: Commands) {
    command.spawn_scene_list(bsn_list! {
                                 (
                                     ui_root_despawn_on_exit(TitleState::Main)
                                     Children [ui::title_ui()]
                                 ),
                             });
}

fn title_background() -> impl SceneList {
    bsn_list! {
        (
            Sprite {
                image: "images/background-pattern.png",
                image_mode: SpriteImageMode::Tiled {
                    tile_x: true,
                    tile_y: true,
                    stretch_value: 0.25,
                },
            }
            AnimatedBackground
            template_value(DespawnOnExit(AppState::Title))
        ),
        (
            music("audio/music/theme.ogg")
            template_value(DespawnOnExit(AppState::Title))
        ),
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
