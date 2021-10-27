use bevy::{
    math::DVec3,
    prelude::{AppBuilder, IntoSystem, Plugin, Query, Transform},
};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(update_transform_from_position.system());
    }
}

pub struct Position {
    pub pos: DVec3,
}

fn update_transform_from_position(mut query: Query<(&Position, &mut Transform)>) {
    for (pos, mut t) in query.iter_mut() {
        t.translation = pos.pos.as_f32();
    }
}
