use std::{hash::Hash, ops::Range, time::Duration};

use bevy::{ecs::event::GlobalTrigger, platform::collections::HashMap, prelude::*};

pub const ANIMATION_EVENT_FINISHED: &str = "finished";

pub(super) fn plugin(app: &mut App) {
    for registration in inventory::iter::<AnimationRegistration> {
        (registration.register)(app);
    }
}

pub struct AnimationRegistration {
    pub register: fn(&mut App),
}

inventory::collect!(AnimationRegistration);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameAnimationSystems;

pub trait AnimationState: Copy + Eq + Hash + Send + Sync + 'static {
    fn animation_key(self) -> String;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackMode {
    Loop,
    OnceHold,
}

#[derive(Clone, Debug)]
pub struct FrameAnimation {
    key: String,
    frames: Range<usize>,
    frame_time: Duration,
    playback: PlaybackMode,
}

impl FrameAnimation {
    pub fn new(
        key: String,
        frames: Range<usize>,
        frame_time: Duration,
        playback: PlaybackMode,
    ) -> Self {
        assert!(
            !frames.is_empty(),
            "Frame animation '{key}' must contain at least one frame"
        );
        assert!(
            !frame_time.is_zero(),
            "Frame animation '{key}' must have a non-zero frame time"
        );

        Self {
            key,
            frames,
            frame_time,
            playback,
        }
    }

    fn first_frame(&self) -> usize {
        self.frames.start
    }

    fn last_frame(&self) -> usize {
        self.frames.end - 1
    }

    fn contains_frame(&self, frame: usize) -> bool {
        self.frames.contains(&frame)
    }
}

#[derive(Component, Clone)]
pub struct FrameAnimationSet<S: AnimationState> {
    animations: HashMap<String, FrameAnimation>,
    current_state: S,
    timer: Timer,
    needs_frame_sync: bool,
    held_once_finished: bool,
}

impl<S: AnimationState> FrameAnimationSet<S> {
    pub fn new(initial_state: S, animations: impl IntoIterator<Item = FrameAnimation>) -> Self {
        let animations = animations
            .into_iter()
            .fold(HashMap::new(), |mut map, animation| {
                let old = map.insert(animation.key.clone(), animation);
                assert!(old.is_none(), "Duplicate frame animation key");
                map
            });

        let initial_key = initial_state.animation_key();
        let initial_frame_time = animations
            .get(&initial_key)
            .unwrap_or_else(|| panic!("Initial frame animation key '{initial_key}' does not exist"))
            .frame_time;

        Self {
            animations,
            current_state: initial_state,
            timer: Timer::new(initial_frame_time, TimerMode::Repeating),
            needs_frame_sync: true,
            held_once_finished: false,
        }
    }

    pub fn set_state(&mut self, state: S) -> bool {
        if self.current_state == state {
            return false;
        }

        let key = state.animation_key();
        if !self.animations.contains_key(&key) {
            handle_missing_animation_key(&key);
            return false;
        }

        self.current_state = state;
        self.needs_frame_sync = true;
        self.held_once_finished = false;
        true
    }

    pub fn current_state(&self) -> S {
        self.current_state
    }

    fn current_animation(&self) -> &FrameAnimation {
        let key = self.current_state.animation_key();
        self.animations
            .get(&key)
            .unwrap_or_else(|| panic!("Current frame animation key '{key}' does not exist"))
    }
}

impl<S> Default for FrameAnimationSet<S>
where
    S: AnimationState + Default,
{
    fn default() -> Self {
        Self {
            animations: HashMap::default(),
            current_state: S::default(),
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            needs_frame_sync: false,
            held_once_finished: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnimationEvent<S: AnimationState> {
    pub entity: Entity,
    pub state: S,
    pub animation_key: String,
    pub event_type: String,
}

impl<S: AnimationState> Event for AnimationEvent<S> {
    type Trigger<'a> = GlobalTrigger;
}

pub fn register_sprite_animation<S: AnimationState>(app: &mut App) {
    app.add_systems(
        Update,
        update_sprite_animation::<S>.in_set(FrameAnimationSystems),
    );
}

pub fn register_image_node_animation<S: AnimationState>(app: &mut App) {
    app.add_systems(
        Update,
        update_image_node_animation::<S>.in_set(FrameAnimationSystems),
    );
}

pub fn animation_key_from_variant(variant: &str) -> String {
    let mut key = String::new();

    for (index, character) in variant.chars().enumerate() {
        if character.is_uppercase() {
            if index != 0 {
                key.push('_');
            }
            character.to_lowercase().for_each(|lower| key.push(lower));
        } else {
            key.push(character);
        }
    }

    key
}

fn update_sprite_animation<S: AnimationState>(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut FrameAnimationSet<S>)>,
    time: Res<Time>,
) {
    query
        .iter_mut()
        .for_each(|(entity, mut sprite, mut animations)| {
            let atlas = sprite
                .texture_atlas
                .as_mut()
                .expect("FrameAnimationSet<Sprite> requires Sprite.texture_atlas");
            update_frame_animation(
                entity,
                &mut atlas.index,
                &mut animations,
                time.delta(),
                &mut commands,
            );
        });
}

fn update_image_node_animation<S: AnimationState>(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ImageNode, &mut FrameAnimationSet<S>)>,
    time: Res<Time>,
) {
    query
        .iter_mut()
        .for_each(|(entity, mut image_node, mut animations)| {
            let atlas = image_node
                .texture_atlas
                .as_mut()
                .expect("FrameAnimationSet<ImageNode> requires ImageNode.texture_atlas");
            update_frame_animation(
                entity,
                &mut atlas.index,
                &mut animations,
                time.delta(),
                &mut commands,
            );
        });
}

fn update_frame_animation<S: AnimationState>(
    entity: Entity,
    frame_index: &mut usize,
    animation_set: &mut FrameAnimationSet<S>,
    delta: Duration,
    commands: &mut Commands,
) {
    let animation = animation_set.current_animation().clone();

    if animation_set.needs_frame_sync {
        *frame_index = animation.first_frame();
        animation_set.timer.set_duration(animation.frame_time);
        animation_set.timer.reset();
        animation_set.needs_frame_sync = false;
        animation_set.held_once_finished = false;
        return;
    }

    if animation.playback == PlaybackMode::OnceHold && animation_set.held_once_finished {
        *frame_index = animation.last_frame();
        return;
    }

    if !animation.contains_frame(*frame_index) {
        *frame_index = animation.first_frame();
        animation_set.timer.reset();
        return;
    }

    if animation_set.timer.duration() != animation.frame_time {
        animation_set.timer.set_duration(animation.frame_time);
    }

    animation_set.timer.tick(delta);

    if !animation_set.timer.just_finished() {
        return;
    }

    if *frame_index >= animation.last_frame() {
        commands.trigger(AnimationEvent {
            entity,
            state: animation_set.current_state,
            animation_key: animation.key.clone(),
            event_type: ANIMATION_EVENT_FINISHED.to_string(),
        });

        match animation.playback {
            PlaybackMode::Loop => {
                *frame_index = animation.first_frame();
            }
            PlaybackMode::OnceHold => {
                *frame_index = animation.last_frame();
                animation_set.held_once_finished = true;
            }
        }
    } else {
        *frame_index += 1;
    }
}

fn handle_missing_animation_key(key: &str) {
    if cfg!(debug_assertions) {
        panic!("Frame animation key '{key}' does not exist");
    } else {
        warn!("Frame animation key '{}' does not exist", key);
    }
}

#[macro_export]
macro_rules! animation_state {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $first_variant:ident $(, $variant:ident)* $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        $vis enum $name {
            $first_variant,
            $($variant),*
        }

        impl Default for $name {
            fn default() -> Self {
                Self::$first_variant
            }
        }

        impl $crate::sprite_animation::AnimationState for $name {
            fn animation_key(self) -> String {
                match self {
                    Self::$first_variant => {
                        $crate::sprite_animation::animation_key_from_variant(stringify!($first_variant))
                    }
                    $(Self::$variant => {
                        $crate::sprite_animation::animation_key_from_variant(stringify!($variant))
                    }),*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! register_frame_animation_state {
    ($state:ty, sprite) => {
        const _: () = {
            fn register(app: &mut bevy::prelude::App) {
                $crate::sprite_animation::register_sprite_animation::<$state>(app);
            }

            inventory::submit! {
                $crate::sprite_animation::AnimationRegistration { register }
            }
        };
    };
    ($state:ty, image_node) => {
        const _: () = {
            fn register(app: &mut bevy::prelude::App) {
                $crate::sprite_animation::register_image_node_animation::<$state>(app);
            }

            inventory::submit! {
                $crate::sprite_animation::AnimationRegistration { register }
            }
        };
    };
    ($state:ty, sprite, image_node) => {
        $crate::register_frame_animation_state!($state, sprite);
        $crate::register_frame_animation_state!($state, image_node);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    animation_state! {
        enum TestAnimation {
            Idle,
            OnceHold,
        }
    }

    #[test]
    fn converts_variant_names_to_animation_keys() {
        assert_eq!(TestAnimation::Idle.animation_key(), "idle");
        assert_eq!(TestAnimation::OnceHold.animation_key(), "once_hold");
    }

    #[test]
    fn setting_same_state_does_not_trigger_transition() {
        let mut animations = FrameAnimationSet::new(
            TestAnimation::Idle,
            [FrameAnimation::new(
                "idle".to_string(),
                0..2,
                Duration::from_millis(100),
                PlaybackMode::Loop,
            )],
        );

        assert!(!animations.set_state(TestAnimation::Idle));
    }
}
