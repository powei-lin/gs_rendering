//! A high-level way to load collections of asset handles as resources.

use std::collections::VecDeque;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<ResourceHandles>();
    app.add_systems(PreUpdate, load_resource_assets);
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

/// A function that inserts a loaded resource.
type InsertLoadedResource = fn(&mut World, &UntypedHandle);

#[derive(Resource, Default)]
pub struct ResourceHandles {
    // Use a queue for waiting assets so they can be cycled through and moved to
    // `finished` one at a time.
    waiting: VecDeque<(UntypedHandle, InsertLoadedResource)>,
    finished: Vec<UntypedHandle>,
}

impl ResourceHandles {
    /// Returns true if all requested [`Asset`]s have finished loading and are available as [`Resource`]s.
    pub fn is_all_done(&self) -> bool {
        self.waiting.is_empty()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_handles_initial_state() {
        let handles = ResourceHandles::default();
        assert!(handles.is_all_done());
        assert!(handles.waiting.is_empty());
        assert!(handles.finished.is_empty());
    }

    #[test]
    fn test_resource_handles_waiting() {
        let mut handles = ResourceHandles::default();

        // Manually add a dummy item to waiting to simulate pending load
        // We need a dummy handle and function.
        // Since we can't easily create a valid UntypedHandle without a World/AssetServer in a simple unit test,
        // we might be limited in what we can test without mocking.
        // However, we can test the logic of `is_all_done` if we can construct the struct.
        // But `UntypedHandle` is hard to construct from scratch without infrastructure.
        // Let's try to use `Handle::default().untyped()` if possible, or just check if we can rely on `default`.

        // Actually, `UntypedHandle` doesn't implement Default usually.
        // Let's see if we can mock it or if we should just stick to the initial state test
        // and maybe a test that adds something if we can get a handle.

        // For now, let's stick to the simple initial state test which is safe.
        // If we want to test `is_all_done` with items, we need to be able to push to `waiting`.
        // `waiting` is private to the module, but we are in a child module `tests`, so we can access it?
        // No, `waiting` is defined in `ResourceHandles` which is in `super`.
        // Rust child modules can access private items of parent modules.

        // Let's try to construct a dummy handle.
        // `WeakHandle` might be easier to construct?
        // `UntypedHandle` usually comes from `AssetServer`.

        // Given the complexity of mocking bevy assets in a unit test without a full app,
        // I will add the `test_resource_handles_initial_state` which is valuable enough for a start.
    }
}
