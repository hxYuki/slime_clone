mod title;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        observer::On,
        relationship::RelationshipSourceCollection,
        system::{BoxedSystem, SystemId},
        template::FromTemplate,
    },
    prelude::*,
    scene::{Scene, bsn, template_value},
    state::{state::States, state_scoped::DespawnOnExit},
    ui::{Node, percent},
};
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
    app.init_resource::<RegisterQuery>();

    app.add_observer(handle_button_group_focus)
        .add_systems(
            Update,
            (switch_button_focus_animation, trigger_focused_button),
        )
        .add_systems(
            Update,
            (collect_unregistered_systems, register_callback_system),
        );
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

// #[derive(Component, Debug, Clone, FromTemplate)]
// pub struct Callback(SystemId);

#[derive(Component)]

pub enum SystemCache {
    Unregistered(Option<BoxedSystem<(), ()>>),
    Registered(SystemId<(), ()>),
}

impl SystemCache {
    pub fn new_system<M>(system: impl IntoSystem<(), (), M>) -> Self {
        Self::Unregistered(Some(Box::new(IntoSystem::into_system(system))))
    }

    pub fn run(&self, commands: &mut Commands) {
        match self {
            SystemCache::Unregistered(_) => {
                warn!("Trying to run an unregistered system, this is likely a bug");
            }
            SystemCache::Registered(id) => {
                commands.run_system(*id);
            }
        }
    }
}

#[derive(Resource, Default)]
struct RegisterQuery {
    new_add_entities: Vec<Entity>,
}

fn collect_unregistered_systems(
    systems: Query<(Entity, &SystemCache), Added<SystemCache>>,
    mut q: ResMut<RegisterQuery>,
) {
    q.new_add_entities
        .extend_from_iter(systems.iter().map(|(e, _)| e));
}

fn register_callback_system(
    world: &mut World,
    // systems: Query<(Entity, &SystemCache), Added<SystemCache>>,
) {
    let mut q = world.get_resource_mut::<RegisterQuery>().unwrap();
    let entities = q.new_add_entities.drain(..).collect::<Vec<_>>();

    entities.into_iter().for_each(|e| {
        let mut comp = world.get_mut::<SystemCache>(e).unwrap();

        let system = match &mut *comp {
            SystemCache::Registered(_) => return,
            SystemCache::Unregistered(system) => system,
        };
        let system = system.take().unwrap();
        let id = world.register_boxed_system(system);

        if let Some(mut cache) = world.get_mut::<SystemCache>(e) {
            *cache = SystemCache::Registered(id);
        }
    });
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
    buttons: Query<(Entity, &InButtonGroup, &ButtonGroupFocused, &SystemCache)>,

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
                system_callback.run(&mut command);
            }
        }
        _ => {
            warn!("Multiple active button groups, can't determine which one to trigger");
        }
    }
}
