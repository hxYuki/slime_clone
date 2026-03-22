use std::time::Duration;

use crate::sprite_animation::{AnimationCollection, AnimationConfig};
use bevy::prelude::*;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct Bat {
    #[dependency]
    pub(crate) texture: Handle<Image>,
    pub(crate) layout: Handle<TextureAtlasLayout>,
    pub(crate) animations: Vec<AnimationConfig>,
}

impl Bat {
    pub fn bundle(&self) -> impl Bundle {
        (
            Sprite {
                image: self.texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: self.layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            Transform::from_scale(Vec3::splat(4.0)).with_translation(Vec3::new(100.0, 0.0, 0.0)),
            AnimationCollection::new(self.animations.clone(), Some("idle".to_string())),
        )
    }
}

impl FromWorld for Bat {
    fn from_world(world: &mut World) -> Self {
        let texture = world.load_asset("images/atlas.png");

        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let layout =
            TextureAtlasLayout::from_grid(UVec2::splat(20), 2, 2, None, Some(UVec2::new(20, 0)));
        let layout = texture_atlas_layouts.add(layout);

        Self {
            texture,
            layout,
            animations: vec![AnimationConfig::new(
                "idle".to_string(),
                0..3,
                Duration::from_millis(200),
            )],
        }
    }
}
