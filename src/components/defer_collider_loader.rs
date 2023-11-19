use bevy::{asset::LoadState, prelude::*};
use bevy_rapier3d::prelude::*;

#[derive(Debug, Default, Reflect, Component)]
pub struct DeferColliderLoader;

pub fn defer_collider_loader(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    server: Res<AssetServer>,
    query: Query<(Entity, &DeferColliderLoader, &Handle<Mesh>)>,
) {
    for (e, _, m) in query.iter() {
        if let Some(LoadState::Loaded) = server.get_load_state(m) {
            let collider = Collider::from_bevy_mesh(
                meshes.get(m).unwrap(),
                &ComputedColliderShape::ConvexHull,
            )
            .unwrap();

            commands
                .entity(e)
                .remove::<DeferColliderLoader>()
                .insert(collider);
        }
    }
}
