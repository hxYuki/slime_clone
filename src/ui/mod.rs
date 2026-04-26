mod title;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        observer::On,
        template::{FromTemplate, template},
    },
    prelude::*,
    scene::{Scene, bsn, template_value},
    state::{state::States, state_scoped::DespawnOnExit},
    ui::{Node, percent},
};
pub use title::*;

use crate::{AppState, sprite_animation::AnimationCollection};

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
    app.add_observer(handle_button_group_focus)
        .add_systems(Update, switch_button_focus_animation);
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
        .filter(|&&b| b != button)
        .for_each(|&b| {
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
