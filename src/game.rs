use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    AppState,
    assets_tracker::{AssetState, cleanup_resource, load_resource},
    gameobjects::{Bat, Room, Slime},
};

pub fn game_plugin(app: &mut App) {
    // Register assets
    // `app.load_resource` will execute on launch, which makes you lose the timing assets are
    // ready. While `init_asset` cannot be called from `World`. So you have initialize the assets here.
    app.init_asset::<Slime>()
        .init_asset::<Bat>()
        .init_asset::<Room>();

    // Load Assets
    app.add_systems(
        OnEnter(AppState::Game),
        (
            load_resource::<Slime>,
            load_resource::<Bat>,
            load_resource::<Room>,
        ),
    );

    // Setup game when assets are ready
    app.add_systems(
        OnEnter(AssetState::Done),
        setup_game.run_if(in_state(AppState::Game)),
    );

    // Game logic
    app.add_systems(
        Update,
        (handle_keyboad_control, update_positions, update_transforms),
    );

    // Cleanup assets
    app.add_systems(
        OnExit(AppState::Game),
        (
            cleanup_resource::<Slime>,
            cleanup_resource::<Bat>,
            cleanup_resource::<Room>,
        ),
    );
}

pub struct GameRegion {}

#[derive(Component)]
pub struct Velocity(Vec2);
impl std::ops::Deref for Velocity {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Velocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[derive(Component)]
pub struct Position(Vec2);
impl std::ops::Deref for Position {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Position {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component)]
pub struct Speed(f32);

#[derive(Component)]
pub struct Player;

fn setup_game(mut commands: Commands, slime: Res<Slime>, bat: Res<Bat>, room: Res<Room>) {
    commands.spawn((
        room.bundle(),
        DespawnOnExit(AppState::Game),
        Transform::from_scale(Vec3::splat(4.0)),
    ));
    commands.spawn((
        slime.bundle(),
        Velocity(Vec2::ZERO),
        Position(Vec2::ZERO),
        Speed(400.0),
        Player,
        DespawnOnExit(AppState::Game),
    ));

    commands.spawn((
        bat.bundle(),
        Velocity(Vec2::ZERO),
        Position(Vec2::new(100.0, 0.0)),
        Speed(400.0),
        DespawnOnExit(AppState::Game),
    ));
}

fn handle_keyboad_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Speed), With<Player>>,
) {
    for (mut velocity, speed) in query.iter_mut() {
        let mut direction = Vec2::ZERO;
        let mut speed = speed.0;
        if keyboard_input.pressed(KeyCode::Space) {
            speed *= 1.5;
        }

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        velocity.0 = direction.normalize_or_zero() * speed;
    }
}

fn update_transforms(mut query: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in query.iter_mut() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn update_positions(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut position, velocity) in query.iter_mut() {
        position.0 += velocity.0 * time.delta_secs();
    }
}
