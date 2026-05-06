use std::time::Duration;

use bevy::{
    ecs::template::{system_value, template},
    image::TextureAtlasTemplate,
    prelude::*,
    scene::{Scene, bsn},
    text::FontSourceTemplate,
};

use crate::{
    change_state_to_game,
    sprite_animation::{AnimationCollection, AnimationConfig},
    ui::{ButtonFocusedEvent, ButtonGroupFocused, Callback, InButtonGroup},
};

pub fn title_ui() -> impl Scene {
    bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(50.0)),
        }
        Children [
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    flex_grow: 1.0,
                }
                TextShadow
                Children [
                    (
                        Text::new("Dungeon")
                        font_04B_30(130.0)
                    ),
                    (
                        Text::new("Slime")
                        font_04B_30(130.0)
                        Node {
                            margin: UiRect::top(px(-40.0)),
                        }
                    ),
                ]
            ),
            (
                #MainButtonGroup
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceAround,
                    width: percent(100.0),
                }
                Children [
                    (
                        styled_button(
                            bsn_list! {
                                Text::new("Start") button_text_style()
                            },
                        )
                        ButtonGroupFocused(true)
                        InButtonGroup(#MainButtonGroup)
                        on(|ev: On<Pointer<Press>>, mut command: Commands, buttons: Query<&Callback>| {
                            buttons
                                .get(ev.entity)
                                .map(|action| {
                                    command.run_system(action.0);
                                })
                                .unwrap();
                        })
                        Callback(system_value(change_state_to_game))
                    ),
                    (
                        styled_button(
                            bsn_list! {
                                Text::new("Option") button_text_style()
                            },
                        )
                        InButtonGroup(#MainButtonGroup)
                    ),
                ]
            ),
        ]
    }
}
pub fn button_text_style() -> impl Scene {
    bsn! {
        TextColor(Color::srgba_u8(7, 14, 120, 255))
        TextLayout::justify(Justify::Center)
        Node {
            margin: UiRect::axes(px(35.0), px(5.0)),
        }
        font_04B_30(25.0)
    }
}
pub fn styled_button(children: impl SceneList) -> impl Scene {
    let slicer = TextureSlicer {
        border: BorderRect::axes(10.0, 6.0),
        max_corner_scale: 4.0,
        ..default()
    };

    let texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::new(65, 15), 1, 3, Some(UVec2::new(0, 1)), None);

    let animations = [
        AnimationConfig::new("default".into(), 0..1, Duration::from_millis(300)),
        AnimationConfig::new("focused".into(), 1..3, Duration::from_millis(300)),
    ];

    bsn! {//
        Button
        ImageNode {
            image: "images/button.png",
            image_mode: NodeImageMode::Sliced(slicer),
            texture_atlas: {TextureAtlasTemplate {
                layout: asset_value(texture_atlas),
                index: 0,
            }},
        }
        template_value(AnimationCollection::new(animations.clone(), Some("default".into())))
        on(|ev: On<Pointer<Enter>>, mut command: Commands| {
            command.trigger(ButtonFocusedEvent(ev.entity));
        })
        Node {
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            justify_items: JustifyItems::Center,
        }
        Children [
            {children}
        ]
    }
}

pub fn font_04B_30(font_size: f32) -> impl Scene {
    bsn! {
        TextFont {
            font: FontSourceTemplate::Handle("fonts/04B_30.ttf"),
            font_size: FontSize::Px(font_size),
        }
    }
}

pub fn image_node_with_texture_atlas(
    path: &'static str,
    layout: TextureAtlasLayout,
    image_mode: NodeImageMode,
) -> impl Scene {
    bsn! {
        template(move |ctx| {
            Ok(ImageNode {
                image: ctx.resource::<AssetServer>().load(path),
                texture_atlas: Some(TextureAtlas {
                    layout: ctx
                        .resource_mut::<Assets<TextureAtlasLayout>>()
                        .add(layout.clone()),
                    index: 0,
                }),
                image_mode: image_mode.clone(),
                ..default()
            })
        })
    }
}
