use std::marker::PhantomData;

use bevy::{
    ecs::{lifecycle::HookContext, system::SystemId, world::DeferredWorld},
    prelude::*,
};

fn notify_on_resource_change<R: Resource>(
    resource: Res<R>,
    subscribers: Query<&ResourceSubscribers<R>, With<ResourceSubscribe<R>>>,
    mut commands: Commands,
) {
    let Some(ResourceSubscribers {
        subscribers,
        _marker,
    }) = subscribers.single().ok()
    else {
        warn!(
            "Resource {:?} changed but there are no subscribers available. {:?} count {}",
            std::any::type_name::<R>(),
            std::any::type_name::<ResourceSubscribe<R>>(),
            subscribers.iter().count()
        );
        return;
    };

    if resource.is_changed() {
        subscribers.iter().for_each(|subscriber| {
            commands
                .entity(*subscriber)
                .trigger(ResourceChanged::<R>::from);
        });
    }
}

#[derive(EntityEvent)]
pub struct ResourceChanged<R: Resource> {
    entity: Entity,
    _marker: PhantomData<R>,
}

impl<R: Resource> From<Entity> for ResourceChanged<R> {
    fn from(entity: Entity) -> Self {
        Self {
            entity,
            _marker: PhantomData,
        }
    }
}

#[derive(Resource)]
#[component(on_add)]
struct ResourceSubscribe<R: Resource>(PhantomData<R>);

impl<R: Resource> ResourceSubscribe<R> {
    pub fn on_add(mut world: DeferredWorld, _ctx: HookContext) {
        world.resource_mut::<Schedules>().add_systems(
            Update,
            notify_on_resource_change::<R>.run_if(resource_changed::<R>),
        );
    }
}

impl<R: Resource> Default for ResourceSubscribe<R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Component)]
#[relationship_target(relationship = SubscribeTo<R>)]
struct ResourceSubscribers<R: Resource> {
    #[relationship]
    subscribers: Vec<Entity>,
    _marker: PhantomData<R>,
}

#[derive(Component)]
#[relationship(relationship_target = ResourceSubscribers<R>)]
struct SubscribeTo<R: Resource>(#[relationship] Entity, PhantomData<R>);

impl<R: Resource> SubscribeTo<R> {
    pub fn new(entity: Entity) -> Self {
        Self(entity, PhantomData)
    }
}

/// The callback one-shot system called on subscribing resources change.
///
/// The system should have an [`Entity`] as input, which is the entity context of the subscriber.
#[derive(Component, FromTemplate)]
pub struct ResourceChangeHandler(pub SystemId<In<Entity>, ()>);

/// Returns a [`SubscribeToTemplate`] which allows you to subscribe to changes of a resource.
///
/// When changes occur, the system in the [`ResourceChangeHandler`] component will be triggered.
pub fn subscribe_to<R: Resource>() -> SubscribeToTemplate<R> {
    SubscribeToTemplate::<R>(PhantomData)
}

#[derive(Default)]
pub struct SubscribeToTemplate<R: Resource>(PhantomData<R>);

impl<R: Resource> Template for SubscribeToTemplate<R> {
    type Output = ();

    fn build_template(
        &self,
        context: &mut bevy::ecs::template::TemplateContext,
    ) -> Result<Self::Output> {
        fn run_handler<R: Resource>(
            res_changed: On<ResourceChanged<R>>,
            mut commands: Commands,
            q_h: Query<&ResourceChangeHandler>,
        ) {
            let Ok(handler) = q_h.get(res_changed.entity) else {
                warn!(
                    "Entity {:?} is subscribing to changes of resource {:?} but doesn't have a ResourceChangeHandler component, so it won't be notified of changes",
                    res_changed.entity,
                    std::any::type_name::<R>()
                );
                return;
            };
            commands.run_system_with(handler.0, res_changed.entity);
        }

        context.entity.observe(run_handler::<R>);

        let id = context.entity.id();
        context.entity.world_scope(|world| {
            let rid = world.init_resource::<ResourceSubscribe<R>>();
            let subscribe_entity = world.resource_entities().get(rid).unwrap();
            world
                .entity_mut(id)
                .insert(SubscribeTo::<R>::new(subscribe_entity));
        });

        Ok(())
    }

    fn clone_template(&self) -> Self {
        Self(PhantomData)
    }
}

impl<R: Resource> Scene for SubscribeToTemplate<R> {
    fn resolve(
        self,
        _context: &mut bevy::scene::ResolveContext,
        scene: &mut bevy::scene::ResolvedScene,
    ) -> Result<(), bevy::scene::ResolveSceneError> {
        scene.push_bundle_template(SubscribeToTemplate::<R>(PhantomData));
        Ok(())
    }
}
