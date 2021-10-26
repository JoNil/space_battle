use bevy::{
    math::DVec3,
    prelude::{AppBuilder, IntoSystem, Plugin, Query},
};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(print_position_system.system());
    }
}

pub struct Position {
    pub pos: DVec3,
}

fn print_position_system(query: Query<&Position>) {
    for pos in query.iter() {}
}
