use bevy::{
    audio::Volume,
    ecs::system::system_value,
    prelude::*,
    ui_widgets::{SliderValue, ValueChange},
};

use crate::audio::ChannelVolume;
use crate::resource_notify::subscribe_to;
use crate::title::{TitleState, change_title_state_to_main};
use crate::ui::{
    InFocusGroup, StyledButton, StyledSlider, button_text_style, font_04B_30,
    ui_root_despawn_on_exit,
};

pub fn options_ui() -> impl Scene {
    bsn! {
        @ui_root_despawn_on_exit(TitleState::Option)
        Children [
            #Group
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
            }
            Children [
                Node {
                    width: percent(90),
                }
                Children [
                    Text::new("options")
                    @font_04B_30(px(80.0))
                ]
                --
                @StyledSlider {
                    @caption: "MUSIC",
                    @resource_change_handler: system_value(|
                        self_entity: In<Entity>,
                        channel_volume: Res<ChannelVolume>,
                        mut commands: Commands|
                    {
                        commands
                            .entity(self_entity.0)
                            .insert(SliderValue(channel_volume.music_volume.to_linear() * 100.0));
                    }),
                    @slider_modifiers: bsn! {
                        on(|value_changed: On<ValueChange<f32>>, mut channel_volume: ResMut<ChannelVolume>| {
                            let value = value_changed.value;
                            channel_volume.music_volume = Volume::Linear(value / 100.0);
                        })
                        @subscribe_to::<ChannelVolume>()
                    },
                }
                InFocusGroup(#Group)
                ~Node {
                    width: percent(80),
                }
                --
                @StyledSlider {
                    @caption: "SFX",
                    @resource_change_handler: system_value(|
                        self_entity: In<Entity>,
                        channel_volume: Res<ChannelVolume>,
                        mut commands: Commands|
                    {
                        commands
                            .entity(self_entity.0)
                            .insert(SliderValue(channel_volume.sound_effect_volume.to_linear() * 100.0));
                    }),
                    @slider_modifiers: bsn! {
                        on(|value_changed: On<ValueChange<f32>>, mut channel_volume: ResMut<ChannelVolume>| {
                            let value = value_changed.value;
                            channel_volume.sound_effect_volume = Volume::Linear(value / 100.0);
                        })
                        @subscribe_to::<ChannelVolume>()
                    },
                }
                InFocusGroup(#Group)
                ~Node {
                    width: percent(80),
                }
                --
                Node {
                    width: percent(80),
                    flex_direction: FlexDirection::RowReverse,
                }
                Children [
                    @StyledButton {
                        @caption: bsn_list! {
                            Text::new("Back")
                            @button_text_style()
                        },
                        @callback: system_value(change_title_state_to_main),
                    }
                    InFocusGroup(#Group)
                ]
            ]
        ]
    }
}
