mod options;
mod title;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        observer::On,
        relationship::RelationshipSourceCollection,
        system::SystemId,
        template::{FromTemplate, system_value},
    },
    picking::hover::Hovered,
    prelude::*,
    scene::{Scene, bsn, template_value},
    state::{state::States, state_scoped::DespawnOnExit},
    ui::{Node, percent},
    ui_widgets::{Slider, SliderRange, SliderThumb, SliderValue, TrackClick, ValueChange},
};
pub use options::*;
pub use title::*;

use crate::sprite_animation::AnimationCollection;

pub fn ui_root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
        }
    }
}

pub fn ui_root_despawn_on_exit<S: States + Default + Clone + Unpin>(state: S) -> impl Scene {
    bsn! {
        ui_root()
        template_value(DespawnOnExit(state))
    }
}

pub fn plugin(app: &mut App) {
    // app.init_resource::<RegisterQuery>();

    app.add_observer(handle_button_group_focus)
        .add_systems(
            Update,
            (switch_button_focus_animation, trigger_focused_button),
        )
        .add_systems(Update, update_slider_thumb_position);
}

#[derive(Component, Debug, FromTemplate)]
#[relationship(relationship_target = ButtonGroup)]
#[require(ButtonGroupFocused)]
pub struct InButtonGroup(Entity);

#[derive(Component, Debug, FromTemplate)]
#[relationship_target(relationship = InButtonGroup)]
pub struct ButtonGroup(Vec<Entity>);

#[derive(Component, Debug, Clone, Default)]
pub struct ButtonGroupFocused(pub bool);

#[derive(Event, Debug)]
pub struct ButtonFocusedEvent(Entity);

#[derive(Component, FromTemplate)]
pub struct Callback(SystemId);

pub fn press_trigger_callback(
    ev: On<Pointer<Press>>,
    mut command: Commands,
    buttons: Query<&Callback>,
) {
    buttons
        .get(ev.entity)
        .map(|action| {
            command.run_system(action.0);
        })
        .expect(
            "You should only use on_press_trigger_callback on entities with a Callback component",
        );
}

fn handle_button_group_focus(
    focused: On<ButtonFocusedEvent>,
    button_groups: Query<(Entity, &ButtonGroup)>,
    mut buttons: Query<(Entity, &InButtonGroup, &mut ButtonGroupFocused)>,
) {
    let button = focused.0;
    let Ok((button, InButtonGroup(button_group), mut focused)) = buttons.get_mut(button) else {
        return;
    };
    if focused.0 {
        return;
    }

    focused.0 = true;

    button_groups
        .get(*button_group)
        .map(|(_, group)| &group.0)
        .unwrap_or(&Vec::new())
        .iter()
        .filter(|&b| b != button)
        .for_each(|b| {
            if let Ok((_, _, mut focused)) = buttons.get_mut(b) {
                focused.0 = false;
            }
        });
}

fn switch_button_focus_animation(
    mut animaiton_button: Query<
        (&mut AnimationCollection, &ButtonGroupFocused),
        Changed<ButtonGroupFocused>,
    >,
) {
    animaiton_button
        .iter_mut()
        .for_each(|(mut animation, ButtonGroupFocused(focused))| {
            if *focused {
                animation.set_current_animation("focused".to_string());
            } else {
                animation.set_current_animation("default".to_string());
            }
        });
}

fn trigger_focused_button(
    input: Res<ButtonInput<KeyCode>>,
    gamepads: Query<(Entity, &Gamepad)>,

    button_groups: Query<(Entity, &ButtonGroup, &InheritedVisibility)>,
    buttons: Query<(Entity, &InButtonGroup, &ButtonGroupFocused, &Callback)>,

    mut command: Commands,
) {
    let mut trigger = false;
    gamepads.iter().take(1).for_each(|(entity, gamepad)| {
        if gamepad.just_pressed(GamepadButton::South) {
            trigger = true;
        }
    });

    if input.just_pressed(KeyCode::Enter) {
        trigger = true;
    }

    if !trigger {
        return;
    }

    let active_groups = button_groups
        .iter()
        .filter(|(_, _, visibility)| visibility.get());
    match active_groups.clone().count() {
        0 => {}
        1 => {
            let (_, group, _) = active_groups.into_iter().next().unwrap();
            let focused_button = group
                .0
                .iter()
                .filter_map(|b| buttons.get(b).ok())
                .find(|(_, _, ButtonGroupFocused(focused), _)| *focused);

            if let Some((_, _, _, system_callback)) = focused_button {
                command.run_system(system_callback.0);
            }
        }
        _ => {
            warn!("Multiple active button groups, can't determine which one to trigger");
        }
    }
}

/// A slider component should listen a data source as SliderValue, and set it on ValueChange.
pub fn slider_ui() -> impl Scene {
    bsn! {
        #Slider
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            justify_items: JustifyItems::Center,
            column_gap: px(4),
            height: px(12),
            width: percent(30),
        }
        Hovered
        Slider {
            track_click: TrackClick::Snap,
        }
        SliderValue(100.0)
        SliderRange::new(0.0, 100.0)
        // on(slider_value_change_update_thumb_position)
        Children [
            (
                Node {
                    height: px(6.0),
                    border_radius: BorderRadius::all(px(3)),
                }
                BackgroundColor(Color::WHITE)
            ),
            (
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(12),
                    top: px(0),
                    bottom: px(0),
                }
                Children [(
                    Node {
                        display: Display::Flex,
                        width: px(12),
                        height: px(12),
                        position_type: PositionType::Absolute,
                        left: percent(0),
                        border_radius: BorderRadius::MAX,
                    }
                    SliderThumb
                    ThumbOfSlider(#Slider)
                    BackgroundColor(Color::BLACK)
                )]
            ),
        ]
    }
}

#[derive(Component, FromTemplate)]
#[relationship(relationship_target = SliderEntityThumb)]
struct ThumbOfSlider(Entity);
#[derive(Component, FromTemplate)]
#[relationship_target(relationship = ThumbOfSlider)]
struct SliderEntityThumb(Entity);

fn update_slider_thumb_position(
    sliders: Query<(&SliderValue, &SliderRange, &SliderEntityThumb), Changed<SliderValue>>,
    mut q_thumb: Query<&mut Node, With<SliderThumb>>,
) {
    sliders
        .iter()
        .for_each(|(value, range, SliderEntityThumb(thumb))| {
            if let Ok(mut node) = q_thumb.get_mut(*thumb) {
                node.left = percent(range.thumb_position(value.0) * 100.0);
            }
        });
}
