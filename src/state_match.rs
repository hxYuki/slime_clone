use std::{marker::PhantomData, os::linux::raw::stat};

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::state_match::event_dispatcher::{EventWatcher, ForwardableEntityEvent, Watch};

pub fn matches<T, S>(
    state: T,
    make_scene: impl Fn() -> S + Clone + Send + Sync + 'static,
) -> impl Scene
where
    T: States + Default + Clone + Copy + Unpin,
    S: Scene,
{
    bsn! {
        MatchStateShowHide::<T>(state)
        on(move |ev: On<ShowScene>, mut commands: Commands| {
            commands
                .spawn_scene(
                    bsn! {
                        ChildOf({ ev.0 })
                        DespawnOnExit::<T>(state)
                        @{ make_scene() }
                    },
                );
        })
    }
}

#[derive(EntityEvent)]
pub struct ShowScene(Entity);

#[derive(EntityEvent, Clone, Copy)]
struct StateChangedTo<T: States + Default>(#[event_target] Entity, T);

impl<T: States + Default> StateChangedTo<T> {
    pub fn matches(&self, state: &T) -> bool {
        state == &self.1
    }
}

impl<T: States + Default> ForwardableEntityEvent for StateChangedTo<T> {
    fn forward_event(&self, target: Entity) -> Self {
        let mut ev = self.clone();
        ev.0 = target;
        ev
    }
}

#[derive(Component, Clone, Copy, Default)]
#[component(on_add)]
struct MatchStateShowHide<T: States + Default>(T);

impl<T: States + Default> MatchStateShowHide<T> {
    pub fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().queue(move |world: &mut World| {
            let entity = ctx.entity;

            let state = world.get::<MatchStateShowHide<T>>(entity).unwrap().clone();

            world.init_resource::<EventWatcher<StateChangedTo<T>>>();

            let target = world
                .resource_entity::<EventWatcher<StateChangedTo<T>>>()
                .unwrap();

            world
                .entity_mut(target)
                .insert_if_new(StateChangeSystem::<T>(PhantomData));

            world
                .entity_mut(entity)
                .insert(Watch::<StateChangedTo<T>>::new(target))
                .observe(move |ev: On<StateChangedTo<T>>, mut commands: Commands| {
                    if ev.matches(&state.0) {
                        commands.trigger(ShowScene(ev.0));
                    }
                });
        })
    }
}

#[derive(Component)]
#[component(on_add)]
struct StateChangeSystem<S: States + Default>(PhantomData<fn() -> S>);

impl<S: States + Default> StateChangeSystem<S> {
    pub fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world
            .resource_mut::<Schedules>()
            .add_systems(Update, trigger_state_change::<S>);
    }
}

fn trigger_state_change<S: States + Default>(
    state: Res<State<S>>,
    ev_watcher: Res<EventWatcher<StateChangedTo<S>>>,
    mut commands: Commands,
) {
    if state.is_changed() {
        let state = state.get().clone();
        let rec = ev_watcher.event_receiver();

        commands.trigger(StateChangedTo::<S>(rec, state));
    }
}

mod event_dispatcher {
    use super::*;

    pub trait ForwardableEntityEvent: Clone + for<'a> EntityEvent<Trigger<'a>: Default> {
        fn forward_event(&self, target: Entity) -> Self;
    }

    #[derive(Resource)]
    #[component(on_add)]
    pub struct EventWatcher<E: ForwardableEntityEvent> {
        ett: Entity,
        _ev: PhantomData<fn() -> E>,
    }

    impl<E: ForwardableEntityEvent> EventWatcher<E> {
        pub fn on_add(mut world: DeferredWorld, ctx: HookContext) {
            let ett = ctx.entity;

            let mut self_comp = world.get_mut::<EventWatcher<E>>(ett).unwrap();

            self_comp.ett = ett;

            let mut cmd = world.commands();

            cmd.entity(ett).observe(
                |ev: On<E>, watchers: Query<&Watchers<E>>, mut commands: Commands| {
                    let e = ev.event_target();

                    let Ok(watchers) = watchers.get(e) else {
                        return;
                    };

                    watchers.0.iter().for_each(|watcher| {
                        commands.trigger(ev.forward_event(*watcher));
                    });
                },
            );
        }

        pub fn event_receiver(&self) -> Entity {
            self.ett
        }
    }

    impl<E: ForwardableEntityEvent> Default for EventWatcher<E> {
        fn default() -> Self {
            Self {
                ett: Entity::PLACEHOLDER,
                _ev: Default::default(),
            }
        }
    }

    #[derive(Component)]
    #[relationship_target(relationship = Watch<E>)]
    pub struct Watchers<E: ForwardableEntityEvent>(
        #[relationship] Vec<Entity>,
        PhantomData<fn() -> E>,
    );

    #[derive(Component)]
    #[relationship(relationship_target = Watchers<E>)]
    pub struct Watch<E: ForwardableEntityEvent>(#[relationship] Entity, PhantomData<fn() -> E>);

    impl<E: ForwardableEntityEvent> Watch<E> {
        pub fn new(entity: Entity) -> Self {
            Self(entity, PhantomData)
        }
    }
}
