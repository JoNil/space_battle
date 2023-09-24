use bevy::{asset::LoadState, prelude::*};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape};

#[derive(Debug, Default, Reflect, Component)]
pub struct DeferColliderLoader;

pub fn defer_collider_loader(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    server: Res<AssetServer>,
    query: Query<(Entity, &DeferColliderLoader, &Handle<Mesh>)>,
) {
    //let m = &meshes.get(&x_shape);

    for (e, _, m) in query.iter() {
        info!("CHECK!");
        if let LoadState::Loaded = server.get_load_state(m) {
            info!("Loaded!");

            let collider = Collider::from_bevy_mesh(
                meshes.get(m).unwrap(),
                &ComputedColliderShape::ConvexDecomposition(Default::default()),
            )
            .unwrap();

            commands
                .entity(e)
                .remove::<DeferColliderLoader>()
                .insert(collider);
        }
    }
}
