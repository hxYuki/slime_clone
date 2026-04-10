use std::time::Duration;

use bevy::{
    color::palettes::css::{GREEN, RED, WHITE},
    ecs::template::template,
    input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest},
    platform::collections::HashMap,
    prelude::*,
    scene2::{CommandsSceneExt, Scene, bsn, on},
};

use crate::{
    AppState,
    assets_tracker::{AssetState, cleanup_resource, load_resource},
    audio::{music, sound_effect},
    gameobjects::{Bat, Room, Slime},
    ui::ui_root_despawn_on_exit,
};

const BASE_MOVE_SPEED: f32 = 100.0;

pub fn game_plugin(app: &mut App) {
    // Register assets
    // `app.load_resource` will execute on launch, which makes you lose the timing assets are
    // ready. While `init_asset` cannot be called from `World`. So you have initialize the assets here.
    app.init_asset::<LevelAssets>()
        .init_asset::<Bat>()
        .init_asset::<Slime>()
        .init_asset::<Room>();

    // Load Assets
    app.add_systems(OnEnter(AppState::Game), (load_resource::<LevelAssets>,));

    // Setup game when assets are ready
    app.add_systems(
        OnEnter(AssetState::Done),
        setup_game.run_if(in_state(AppState::Game)),
    );

    // Game logic
    app.add_systems(
        Update,
        (
            handle_keyboad_control,
            handle_gamepad_control,
            update_positions,
            update_transforms,
            collision_check,
        ),
    );

    // Cleanup assets
    app.add_systems(OnExit(AppState::Game), (cleanup_resource::<LevelAssets>,));
}

#[derive(Resource)]
pub struct GameState {
    pub player_score: u32,
    pub room_size: UVec2,
    pub grid_size: u32,
    pub room_bound: Rect,
}

#[derive(Asset, Resource, Clone, Reflect)]
#[reflect(Resource)]
struct LevelAssets {
    #[dependency]
    bat: Bat,
    #[dependency]
    slime: Slime,
    #[dependency]
    room: Room,
    #[dependency]
    theme_music: Handle<AudioSource>,
    #[dependency]
    bounce_sound: Handle<AudioSource>,
    #[dependency]
    collect_sound: Handle<AudioSource>,
    #[dependency]
    score_font: Handle<Font>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let bat = Bat::from_world(world);
        let slime = Slime::from_world(world);
        let room = Room::from_world(world);

        let asset_server = world.resource::<AssetServer>();

        // let bat = asset_server.add(bat);
        // let slime = asset_server.add(slime);
        // let room = asset_server.add(room);

        let theme_music = asset_server.load("audio/music/theme.ogg");
        let bounce_sound = asset_server.load("audio/sound_effects/bounce.wav");
        let collect_sound = asset_server.load("audio/sound_effects/collect.wav");

        let score_font = asset_server.load("fonts/04B_30.ttf");

        Self {
            bat,
            slime,
            room,
            theme_music,
            bounce_sound,
            collect_sound,
            score_font,
        }
    }
}

#[derive(Component)]
pub struct VelocityDir(Vec2);
impl std::ops::Deref for VelocityDir {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for VelocityDir {
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

#[derive(Component)]
pub struct Mob;

fn setup_game(mut commands: Commands, level: Res<LevelAssets>) {
    let bound = Rect::from_center_half_size(Vec2::ZERO, Vec2::new(16. - 2., 9. - 2.) * 20. / 2.);
    commands.insert_resource(GameState {
        player_score: 0,
        room_size: UVec2::new(16, 9),
        grid_size: 20,
        room_bound: bound,
    });
    let room = &level.room;
    let slime = &level.slime;
    let bat = &level.bat;
    commands.spawn((
        music(level.theme_music.clone()),
        DespawnOnExit(AppState::Game),
    ));

    commands.spawn_scene(bsn! {
        ui_root_despawn_on_exit(AppState::Game)

        Children [
            score_text()
        ]
    });

    commands.spawn((
        room.bundle(),
        DespawnOnExit(AppState::Game),
        Transform::from_scale(Vec3::splat(1.0)),
    ));
    commands
        .spawn((
            slime.bundle(),
            VelocityDir(Vec2::ZERO),
            Position(Vec2::ZERO),
            Speed(100.0),
            Player,
            DespawnOnExit(AppState::Game),
        ))
        .observe(player_score);

    commands
        .spawn((
            bat.bundle(),
            VelocityDir(get_random_velocity().normalize()),
            Position(Vec2::new(100.0, 0.0)),
            Speed(100.0),
            Mob,
            DespawnOnExit(AppState::Game),
        ))
        .observe(mob_relocate);
}

fn score_text() -> impl Scene {
    bsn! {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            top: px(20),
            left: px(80),
        }

        Text::new("Score: ")
        TextColor(WHITE)
        template(|context| {
            Ok(TextFont {
                font: context.resource::<LevelAssets>().score_font.clone().into(),
                font_size: 35.0.into(),
                ..default()
            })
        })

        Children [
        (
            ScoreText
            TextSpan::new("0")
            template(|context| {
                Ok(TextFont {
                    font: context.resource::<LevelAssets>().score_font.clone().into(),
                    font_size: 35.0.into(),
                    ..default()
                })
            })
        )]
    )}
}

#[derive(Component, FromTemplate)]
struct ScoreText;

fn handle_keyboad_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut VelocityDir, &mut Speed), With<Player>>,
) {
    for (mut velocity, mut speed) in query.iter_mut() {
        let mut direction = Vec2::ZERO;
        let mut move_speed = BASE_MOVE_SPEED;
        if keyboard_input.pressed(KeyCode::Space) {
            move_speed *= 1.5;
        }

        speed.0 = move_speed;

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

        velocity.0 = direction.normalize_or_zero();
    }
}

fn handle_gamepad_control(
    gamepads: Query<(Entity, &Gamepad)>,
    mut rumble_requests: MessageWriter<GamepadRumbleRequest>,
    mut player: Query<(&mut VelocityDir, &mut Speed), With<Player>>,
) {
    for (entity, gamepad) in gamepads.iter() {
        info_once!("Gamepad {:?} is connected", gamepad);

        let vel_dir = gamepad.left_stick();
        let boost = gamepad.pressed(GamepadButton::South);

        let Some((mut velocity, mut speed)) = player.iter_mut().next() else {
            continue;
        };

        // Death zone
        velocity.0 = if vel_dir.length_squared() > 0.01 {
            vel_dir
        } else {
            Vec2::ZERO
        };
        speed.0 = BASE_MOVE_SPEED
            * if boost {
                rumble_requests.write(GamepadRumbleRequest::Add {
                    duration: Duration::from_secs_f32(0.1),
                    intensity: GamepadRumbleIntensity::strong_motor(0.01),
                    gamepad: entity,
                });
                1.5
            } else {
                1.0
            };
    }
}

fn update_transforms(mut query: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in query.iter_mut() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn update_positions(mut query: Query<(&mut Position, &VelocityDir, &Speed)>, time: Res<Time>) {
    for (mut position, velocity, speed) in query.iter_mut() {
        position.0 += velocity.0 * speed.0 * time.delta_secs();
    }
}

#[derive(EntityEvent)]
pub struct PlayerScoce(Entity);

#[derive(EntityEvent)]
pub struct MobDestroyed(Entity);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomEdge {
    Left,
    Top,
    Right,
    Bottom,
}

fn collision_check(
    mut commands: Commands,
    mut player: Query<(Entity, &mut Position, &mut VelocityDir), (With<Player>, Without<Mob>)>,
    mut mob: Query<(Entity, &mut Position, &mut VelocityDir), (With<Mob>, Without<Player>)>,

    game_state: If<Res<GameState>>,
    level_assets: If<Res<LevelAssets>>,
    // mut gizmos: Gizmos,
) {
    let room_bound = game_state.room_bound;

    let Ok((player, mut player_pos, mut player_vel)) = player.single_mut() else {
        return;
    };
    let player_bound = Rect::from_center_half_size(player_pos.0, Vec2::splat(10.));

    let Ok((mob, mut mob_pos, mut mob_vel)) = mob.single_mut() else {
        return;
    };
    let mob_bound = Rect::from_center_half_size(mob_pos.0, Vec2::splat(10.));

    // gizmos.rect_2d(room_bound.center(), room_bound.size(), RED);
    // gizmos.rect_2d(player_bound.center(), player_bound.size(), GREEN);
    // gizmos.rect_2d(mob_bound.center(), mob_bound.size(), GREEN);

    let mut play_bounce_sound = false;

    for edge in touch_room_edge(mob_bound, room_bound) {
        play_bounce_sound = true;

        mob_vel.0 = match edge {
            RoomEdge::Left | RoomEdge::Right => Vec2::new(-mob_vel.x, mob_vel.y),
            RoomEdge::Top | RoomEdge::Bottom => Vec2::new(mob_vel.x, -mob_vel.y),
        };

        match edge {
            RoomEdge::Left => mob_pos.x = f32::max(mob_pos.x, game_state.room_bound.min.x + 10.),
            RoomEdge::Right => mob_pos.x = f32::min(mob_pos.x, game_state.room_bound.max.x - 10.),
            RoomEdge::Bottom => mob_pos.y = f32::max(mob_pos.y, game_state.room_bound.min.y + 10.),
            RoomEdge::Top => mob_pos.y = f32::min(mob_pos.y, game_state.room_bound.max.y - 10.),
        }
    }

    for edge in touch_room_edge(player_bound, room_bound) {
        match edge {
            RoomEdge::Left => {
                player_vel.x = f32::max(player_vel.x, 0.);
                player_pos.x = f32::max(player_pos.x, game_state.room_bound.min.x + 10.)
            }
            RoomEdge::Right => {
                player_vel.x = f32::min(player_vel.x, 0.);
                player_pos.x = f32::min(player_pos.x, game_state.room_bound.max.x - 10.)
            }
            RoomEdge::Bottom => {
                player_vel.y = f32::max(player_vel.y, 0.);
                player_pos.y = f32::max(player_pos.y, game_state.room_bound.min.y + 10.)
            }
            RoomEdge::Top => {
                player_vel.y = f32::min(player_vel.y, 0.);
                player_pos.y = f32::min(player_pos.y, game_state.room_bound.max.y - 10.)
            }
        }
    }

    if play_bounce_sound {
        commands.spawn(sound_effect(level_assets.bounce_sound.clone()));
    }

    if !player_bound.intersect(mob_bound).is_empty() {
        commands.trigger(PlayerScoce(player));
        commands.trigger(MobDestroyed(mob));

        commands.spawn(sound_effect(level_assets.collect_sound.clone()));
    }

    fn touch_room_edge(item: Rect, room_bound: Rect) -> Vec<RoomEdge> {
        let mut touches = Vec::with_capacity(4);
        if item.min.x <= room_bound.min.x {
            touches.push(RoomEdge::Left);
        }
        if item.min.y <= room_bound.min.y {
            touches.push(RoomEdge::Bottom);
        }
        if item.max.x >= room_bound.max.x {
            touches.push(RoomEdge::Right);
        }
        if item.max.y >= room_bound.max.y {
            touches.push(RoomEdge::Top);
        }
        touches
    }
}

fn mob_relocate(
    mob_destroyed: On<MobDestroyed>,
    mut mob: Query<(&mut VelocityDir, &mut Position), With<Mob>>,
    game_state: Res<GameState>,
) {
    let room_bound = game_state.room_bound;

    let (mut mob_vel, mut mob_pos) = mob.get_mut(mob_destroyed.0).unwrap();

    mob_vel.0 = get_random_velocity();

    mob_pos.0 = get_random_vec2_in_rect(room_bound.inflate(-10.));
}

fn get_random_vec2_in_rect(rect: Rect) -> Vec2 {
    Vec2::new(
        rand::random::<f32>() * (rect.max.x - rect.min.x) + rect.min.x,
        rand::random::<f32>() * (rect.max.y - rect.min.y) + rect.min.y,
    )
}
fn get_random_velocity() -> Vec2 {
    let r = rand::random::<f32>();
    Vec2::new(
        f32::cos(r * std::f32::consts::TAU),
        f32::sin(r * std::f32::consts::TAU),
    )
    .normalize()
}

fn player_score(
    player_score: On<PlayerScoce>,
    mut game_state: ResMut<GameState>,
    mut query: Query<&mut TextSpan, With<ScoreText>>,
) {
    info!("{} player score", player_score.0);
    game_state.player_score += 1;

    let Ok(mut text) = query.single_mut() else {
        return;
    };
    **text = (game_state.player_score * 100).to_string();
}

fn update_score_text(
    game_state: If<Res<GameState>>,
    mut query: Query<&mut TextSpan, With<ScoreText>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    **text = game_state.player_score.to_string();
}
