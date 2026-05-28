use std::time::Duration;

use bevy::{
    ecs::template::{SystemIdTemplate, system_value, template},
    image::TextureAtlasTemplate,
    prelude::*,
    scene::{Scene, bsn},
    text::FontSourceTemplate,
};

use crate::{
    change_state_to_game,
    sprite_animation::{AnimationCollection, AnimationConfig},
    title::change_title_state_to_option,
    ui::{
        ButtonFocusedEvent, ButtonGroupFocused, Callback, InFocusGroup, StyledButton,
        change_button_focus, press_trigger_callback,
    },
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
                        @StyledButton {
                            @caption: { bsn_list! {
                                Text::new("Start") button_text_style()
                            } },
                            @callback: system_value(change_state_to_game),
                        }
                        ButtonGroupFocused(true)
                        InFocusGroup(#MainButtonGroup)
                    ),
                    (
                        @StyledButton {
                            @caption: { bsn_list! {
                                Text::new("Options") button_text_style()
                            } },
                            @callback: system_value(change_title_state_to_option),
                        }
                        InFocusGroup(#MainButtonGroup)
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

pub fn font_04B_30(font_size: f32) -> impl Scene {
    bsn! {
        TextFont {
            font: FontSourceTemplate::Handle("fonts/04B_30.ttf"),
            font_size: px(font_size),
        }
    }
}
