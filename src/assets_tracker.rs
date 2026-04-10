//! A high-level way to load collections of asset handles as resources.

use std::collections::VecDeque;

use bevy::{asset::UntypedAssetId, prelude::*, state};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ResourceHandles>()
        .init_state::<AssetState>();
    app.add_systems(PreUpdate, (load_resource_assets, check_load_states));

    // #[cfg(feature = "dev")]
    // app.add_systems(
    //     Update,
    //     bevy::dev_tools::states::log_transitions::<AssetState>,
    // );
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub(super) enum AssetState {
    #[default]
    Idling,
    Loading,
    Done,
}

pub fn load_resource<T: Resource + Asset + Clone + FromWorld>(world: &mut World) {
    world.load_resource::<T>();
}
pub fn cleanup_resource<T: Asset + Resource>(world: &mut World) {
    world.resource_scope(|world, mut resource_handle: Mut<ResourceHandles>| {
        world.remove_resource::<T>();

        let assets = world.resource::<Assets<T>>();
        assets.ids().map(|id| id.untyped()).for_each(|id| {
            resource_handle.remove(id);
        });
    });
}

pub trait LoadResource {
    /// This will load the [`Resource`] as an [`Asset`]. When all of its asset dependencies
    /// have been loaded, it will be inserted as a resource. This ensures that the resource only
    /// exists when the assets are ready.
    fn load_resource<T: Resource + Asset + Clone + FromWorld>(&mut self) -> &mut Self;
}

impl LoadResource for App {
    fn load_resource<T: Resource + Asset + Clone + FromWorld>(&mut self) -> &mut Self {
        self.init_asset::<T>();
        let world = self.world_mut();

        world
            .resource_mut::<NextState<AssetState>>()
            .set(AssetState::Loading);

        let value = T::from_world(world);
        let assets = world.resource::<AssetServer>();
        let handle = assets.add(value);
        let mut handles = world.resource_mut::<ResourceHandles>();
        handles
            .waiting
            .push_back((handle.untyped(), |world, handle| {
                let assets = world.resource::<Assets<T>>();
                if let Some(value) = assets.get(handle.id().typed::<T>()) {
                    world.insert_resource(value.clone());
                }
            }));
        self
    }
}
impl LoadResource for World {
    fn load_resource<T: Resource + Asset + Clone + FromWorld>(&mut self) -> &mut Self {
        // Do not load assets repeatedly
        let is_loaded = self.contains_resource::<T>();
        let is_loading = self.contains_resource::<ResourceLoadingMarker<T>>();

        if is_loaded || is_loading {
            return self;
        }

        self.resource_mut::<NextState<AssetState>>()
            .set(AssetState::Loading);

        self.init_resource::<ResourceLoadingMarker<T>>();

        let value = T::from_world(self);
        let assets = self.resource::<AssetServer>();
        let handle = assets.add(value);
        let mut handles = self.resource_mut::<ResourceHandles>();
        handles
            .waiting
            .push_back((handle.untyped(), |world, handle| {
                let assets = world.resource::<Assets<T>>();
                if let Some(value) = assets.get(handle.id().typed::<T>()) {
                    world.insert_resource(value.clone());
                    world.remove_resource::<ResourceLoadingMarker<T>>();
                }
            }));
        self
    }
}

/// A function that inserts a loaded resource.
type InsertLoadedResource = fn(&mut World, &UntypedHandle);

#[derive(Resource, Default)]
pub struct ResourceHandles {
    // Use a queue for waiting assets so they can be cycled through and moved to
    // `finished` one at a time.
    waiting: VecDeque<(UntypedHandle, InsertLoadedResource)>,
    finished: Vec<UntypedHandle>,
}

#[derive(Resource, FromWorld)]
struct ResourceLoadingMarker<T: Resource + Asset>(std::marker::PhantomData<T>);

impl ResourceHandles {
    /// Returns true if all requested [`Asset`]s have finished loading and are available as [`Resource`]s.
    pub fn is_all_done(&self) -> bool {
        self.waiting.is_empty()
    }

    pub fn remove(&mut self, ids: UntypedAssetId) {
        self.waiting.retain(|(handle, _)| handle.id() != ids);
        self.finished.retain(|handle| handle.id() != ids);
    }
}

fn load_resource_assets(world: &mut World) {
    world.resource_scope(|world, mut resource_handles: Mut<ResourceHandles>| {
        world.resource_scope(|world, assets: Mut<AssetServer>| {
            for _ in 0..resource_handles.waiting.len() {
                let (handle, insert_fn) = resource_handles.waiting.pop_front().unwrap();
                if assets.is_loaded_with_dependencies(&handle) {
                    insert_fn(world, &handle);
                    resource_handles.finished.push(handle);
                } else {
                    resource_handles.waiting.push_back((handle, insert_fn));
                }
            }
        });
    });
}

fn check_load_states(
    resource_handles: Res<ResourceHandles>,
    mut state: ResMut<NextState<AssetState>>,
    current_state: Res<State<AssetState>>,
) {
    if resource_handles.is_all_done() {
        match *current_state.get() {
            AssetState::Loading => state.set(AssetState::Done),
            AssetState::Done => state.set(AssetState::Idling),
            _ => {}
        }
    }
}

pub fn check_asset_count<T: Asset>(
    assets: Res<Assets<T>>,
    resource_handles: Res<ResourceHandles>,
    mut local_count: Local<usize>,
    mut handle_count: Local<usize>,
) {
    let assets_count = assets.ids().count();
    let handle_finished_count = resource_handles.finished.len();
    if handle_finished_count != *handle_count {
        *handle_count = handle_finished_count;
        info!(
            "{} assets finished loading",
            resource_handles.finished.len()
        );
    }
    if assets_count != *local_count {
        *local_count = assets_count;
        info!("{} assets loaded", assets.ids().count());
    }
}
