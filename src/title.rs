use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    AppState,
    sprite_animation::AnimationCollection,
    ui::{self, ButtonGroupFocused, ui_root_despawn_on_exit},
};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Title), setup_title);

    app.add_systems(
        Update,
        animated_background.run_if(in_state(AppState::Title)),
    );
}

fn setup_title(mut command: Commands, asset_server: Res<AssetServer>) {
    command.spawn_scene(bsn! {
                            ui_root_despawn_on_exit(AppState::Title)
                            Children [ui::title_ui()]
                        });

    command.spawn((
        Sprite {
            image: asset_server.load("images/background-pattern.png"),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 0.25,
            },

            ..default()
        },
        AnimatedBackground,
        DespawnOnExit(AppState::Title),
    ));
}

#[derive(Component)]
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
