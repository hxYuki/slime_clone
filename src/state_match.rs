use std::marker::PhantomData;

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    ui::experimental::GhostNode,
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
        GhostNode
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
    pub fn on_add(mut world: DeferredWorld, _ctx: HookContext) {
        // Loaded scenes can register Update systems, so spawn them after Update
        // and before their first UI layout and transform propagation.
        world.resource_mut::<Schedules>().add_systems(
            PostUpdate,
            trigger_state_change::<S>
                .before(bevy::ui::UiSystems::Prepare)
                .before(bevy::transform::TransformSystems::Propagate),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppState, title::TitleState};
    use bevy::{asset::AssetPlugin, scene::ScenePlugin, state::app::StatesPlugin};

    #[derive(Component, Clone, Default)]
    struct MainScreen;

    #[derive(Component, Clone, Default)]
    struct OptionScreen;

    fn screens() -> impl Scene {
        bsn! {
            @crate::ui::ui_root()
            DespawnOnExit::<AppState>(AppState::Title)
            Children [
                @matches(TitleState::Main, || bsn! { MainScreen })
                --
                @matches(TitleState::Option, || bsn! { OptionScreen })
            ]
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                watch_for_changes_override: Some(false),
                use_asset_processor_override: Some(false),
                ..default()
            },
            ScenePlugin,
            StatesPlugin,
        ))
        .init_state::<AppState>()
        .add_sub_state::<TitleState>()
        .add_systems(OnEnter(AppState::Title), screens.spawn());
        app
    }

    fn screen_counts(app: &mut App) -> (usize, usize) {
        let world = app.world_mut();
        for (child, parent) in world
            .query_filtered::<(Entity, &ChildOf), With<GlobalTransform>>()
            .iter(world)
        {
            assert!(
                world.get::<GlobalTransform>(parent.parent()).is_some(),
                "B0004: {child:?} has a parent without GlobalTransform"
            );
        }
        (
            world.query::<&MainScreen>().iter(world).count(),
            world.query::<&OptionScreen>().iter(world).count(),
        )
    }

    #[test]
    fn loads_once_switches_and_restores_after_leaving_title() {
        let mut app = test_app();
        app.update();
        assert_eq!(screen_counts(&mut app), (1, 0));
        app.update();
        assert_eq!(screen_counts(&mut app), (1, 0));

        app.world_mut()
            .resource_mut::<NextState<TitleState>>()
            .set(TitleState::Option);
        app.update();
        assert_eq!(screen_counts(&mut app), (0, 1));
        app.update();
        assert_eq!(screen_counts(&mut app), (0, 1));

        app.world_mut()
            .resource_mut::<NextState<TitleState>>()
            .set(TitleState::Main);
        app.update();
        assert_eq!(screen_counts(&mut app), (1, 0));

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        assert_eq!(screen_counts(&mut app), (0, 0));
        assert!(!app.world().contains_resource::<State<TitleState>>());
        app.update();

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Title);
        app.update();
        assert_eq!(screen_counts(&mut app), (1, 0));
        app.update();
        assert_eq!(screen_counts(&mut app), (1, 0));
    }

    #[derive(Resource, Default)]
    struct Volume(f32);

    #[derive(Component)]
    struct DisplayedVolume(f32);

    fn subscribed_options() -> impl Scene {
        use crate::resource_notify::{ResourceChangeHandler, subscribe_to};
        use bevy::ecs::system::system_value;

        bsn! {
            @subscribe_to::<Volume>()
            ResourceChangeHandler(system_value(|
                entity: In<Entity>,
                volume: Res<Volume>,
                mut commands: Commands,
            | {
                commands.entity(entity.0).insert(DisplayedVolume(volume.0));
            }))
        }
    }

    #[test]
    fn newly_loaded_options_keep_receiving_resource_changes() {
        let mut app = test_app();
        app.init_resource::<Volume>();
        app.update();
        app.world_mut()
            .spawn_scene(matches(TitleState::Option, subscribed_options))
            .unwrap();
        app.world_mut()
            .resource_mut::<NextState<TitleState>>()
            .set(TitleState::Option);
        app.update();

        let world = app.world_mut();
        assert_eq!(
            world.query::<&DisplayedVolume>().single(world).unwrap().0,
            0.0
        );
        world.resource_mut::<Volume>().0 = 0.75;
        app.update();
        let world = app.world_mut();
        assert_eq!(
            world.query::<&DisplayedVolume>().single(world).unwrap().0,
            0.75
        );
    }

    fn layout_child() -> impl Scene {
        bsn! {
            Node {
                width: percent(50),
                height: percent(100),
                min_width: px(0),
                flex_grow: 1.0,
            }
        }
    }

    #[test]
    fn matches_preserves_flex_and_grid_layout() {
        use bevy::{
            app::{HierarchyPropagatePlugin, PropagateSet},
            camera::{ComputedCameraValues, RenderTargetInfo},
            text::{FontCx, RemSize},
            ui::{
                ComputedUiRenderTargetInfo, ComputedUiTargetCamera, UiSystems, ui_layout_system,
                ui_surface::UiSurface, update::propagate_ui_target_cameras,
            },
        };

        for display in [Display::Flex, Display::Grid] {
            let mut app = test_app();
            app.add_plugins((
                HierarchyPropagatePlugin::<ComputedUiTargetCamera>::new(PostUpdate),
                HierarchyPropagatePlugin::<ComputedUiRenderTargetInfo>::new(PostUpdate),
            ))
            .init_resource::<UiScale>()
            .init_resource::<UiSurface>()
            .init_resource::<FontCx>()
            .init_resource::<RemSize>()
            .add_systems(
                PostUpdate,
                propagate_ui_target_cameras.in_set(UiSystems::Prepare),
            )
            .add_systems(PostUpdate, ui_layout_system.in_set(UiSystems::Layout))
            .configure_sets(
                PostUpdate,
                (
                    UiSystems::Prepare,
                    PropagateSet::<ComputedUiTargetCamera>::default(),
                    PropagateSet::<ComputedUiRenderTargetInfo>::default(),
                    UiSystems::Layout,
                )
                    .chain(),
            );

            let world = app.world_mut();
            world.spawn((
                Camera2d,
                Camera {
                    computed: ComputedCameraValues {
                        target_info: Some(RenderTargetInfo {
                            physical_size: UVec2::new(1000, 500),
                            scale_factor: 1.0,
                        }),
                        ..default()
                    },
                    ..default()
                },
            ));

            let parent = Node {
                display,
                width: px(300),
                height: px(200),
                column_gap: px(10),
                grid_template_columns: vec![GridTrack::fr(1.0), GridTrack::fr(2.0)],
                ..default()
            };
            let baseline = world.spawn((parent.clone(), Transform::default())).id();
            let wrapped = world.spawn((parent, Transform::default())).id();
            let direct_child = world.spawn_scene(layout_child()).unwrap().id();
            let wrapper = world
                .spawn_scene(matches(TitleState::Main, layout_child))
                .unwrap()
                .id();
            world.entity_mut(baseline).add_child(direct_child);
            world.entity_mut(wrapped).add_child(wrapper);

            let mut siblings = Vec::new();
            for root in [baseline, wrapped] {
                siblings.push(
                    world
                        .spawn((
                            Node {
                                width: px(60),
                                height: px(40),
                                ..default()
                            },
                            ChildOf(root),
                        ))
                        .id(),
                );
            }

            app.update();
            let world = app.world_mut();
            let loaded_child = world.get::<Children>(wrapper).unwrap()[0];
            let mut surface = world.resource_mut::<UiSurface>();
            for (direct, through_matches) in [
                (baseline, wrapped),
                (direct_child, loaded_child),
                (siblings[0], siblings[1]),
            ] {
                let direct_layout = surface.get_layout(direct, true).unwrap().0;
                let wrapped_layout = surface.get_layout(through_matches, true).unwrap().0;
                assert_eq!(direct_layout.size, wrapped_layout.size);
                assert_eq!(direct_layout.location, wrapped_layout.location);
                assert!(direct_layout.size.width > 0.0);
                assert!(direct_layout.size.height > 0.0);
            }
        }
    }
}

fn trigger_state_change<S: States + Default>(
    state: If<Res<State<S>>>,
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
