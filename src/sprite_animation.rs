use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (update_animation, update_animation_image_node));
}

#[derive(Clone, Reflect)]
pub struct AnimationConfig {
    name: String,
    first_sprite_index: usize,
    last_sprite_index: usize,
    interval: Duration,
}
impl AnimationConfig {
    pub fn new(name: String, index_range: std::ops::Range<usize>, interval: Duration) -> Self {
        Self {
            name,
            first_sprite_index: index_range.start,
            last_sprite_index: index_range.end,
            interval,
        }
    }

    pub fn get_next_index(&self, current_index: usize) -> usize {
        if current_index >= self.last_sprite_index - 1 {
            self.first_sprite_index
        } else {
            current_index + 1
        }
    }
}
#[derive(Component, Clone, Default)]
pub struct AnimationCollection {
    animations: HashMap<String, AnimationConfig>,
    current_animation: Option<String>,
    timer: Timer,
}
impl AnimationCollection {
    pub fn new(
        animations: impl IntoIterator<Item = AnimationConfig>,
        current_animation: Option<String>,
    ) -> Self {
        let timer = Timer::new(Duration::from_millis(200), TimerMode::Repeating);

        Self {
            animations: HashMap::from_iter(animations.into_iter().map(|a| (a.name.clone(), a))),
            current_animation,
            timer,
        }
    }

    pub fn ensure_animation(&mut self) {
        if self.current_animation.is_none() {
            let first_key = self.animations.keys().next();
            self.current_animation = first_key.cloned();
        }
    }
    pub fn get_current_animation(&self) -> &AnimationConfig {
        self.animations
            .get(self.current_animation.as_ref().unwrap())
            .unwrap()
    }
}

fn update_animation(mut query: Query<(&mut Sprite, &mut AnimationCollection)>, time: Res<Time>) {
    query.iter_mut().for_each(|(mut sprite, mut collection)| {
        collection.ensure_animation();
        let cur = collection.get_current_animation().interval;

        if collection.timer.duration() != cur {
            collection.timer.set_duration(cur);
        }

        collection.timer.tick(time.delta());

        if collection.timer.just_finished() {
            let animation = collection.get_current_animation();
            sprite.texture_atlas.as_mut().unwrap().index =
                animation.get_next_index(sprite.texture_atlas.as_ref().unwrap().index);
        }
    });
}

fn update_animation_image_node(
    mut query: Query<(&mut ImageNode, &mut AnimationCollection)>,
    time: Res<Time>,
) {
    query
        .iter_mut()
        .for_each(|(mut image_node, mut collection)| {
            collection.ensure_animation();
            let cur = collection.get_current_animation().interval;

            if collection.timer.duration() != cur {
                collection.timer.set_duration(cur);
            }

            collection.timer.tick(time.delta());

            if collection.timer.just_finished() {
                let animation = collection.get_current_animation();
                image_node.texture_atlas.as_mut().unwrap().index =
                    animation.get_next_index(image_node.texture_atlas.as_ref().unwrap().index);
            }
        });
}
