use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    AppState,
    sprite_animation::AnimationCollection,
    ui::{self, ui_root_despawn_on_exit},
};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Title), setup_title);

    app.add_systems(
        Update,
        animated_background.run_if(in_state(AppState::Title)),
    );

    app.add_observer(reset_button_animation)
        .add_observer(focus_button_animation);
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

#[derive(Component, Default, Clone, Copy)]
pub struct FocusedButton;

fn reset_button_animation(
    removal: On<Remove, FocusedButton>,
    mut buttons: Query<(&mut AnimationCollection, Has<FocusedButton>)>,
) {
    if let Ok((mut collection, _)) = buttons.get_mut(removal.entity) {
        collection.set_current_animation("default".to_string());
    }
}

fn focus_button_animation(
    addition: On<Add, FocusedButton>,
    mut buttons: Query<(&mut AnimationCollection, Has<FocusedButton>)>,

    mut command: Commands,
    focused: Query<(Entity), (With<FocusedButton>, With<Button>, With<AnimationCollection>)>,
) {
    focused
        .iter()
        .filter(|(entity)| entity != &addition.entity)
        .for_each(|(entity)| {
            command.entity(entity).remove::<FocusedButton>();
        });

    if let Ok((mut collection, _)) = buttons.get_mut(addition.entity) {
        collection.set_current_animation("focused".to_string());
    }
}
