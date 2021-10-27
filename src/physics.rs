use bevy::{
    core::Time,
    ecs::bundle::Bundle,
    math::DVec3,
    prelude::{AppBuilder, IntoSystem, Plugin, Query, Res, Transform},
};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(update_pos_from_velocity.system())
            .add_system(update_transform_from_position.system());
    }
}

#[derive(Default)]
pub struct Mass(pub f64);

#[derive(Default)]
pub struct Velocity(pub DVec3);

#[derive(Default)]
pub struct Position(pub DVec3);

#[derive(Bundle, Default)]
pub struct PhysicsBundle {
    pub mass: Mass,
    pub vel: Velocity,
    pub pos: Position,
}

fn update_pos_from_velocity(time: Res<Time>, mut query: Query<(&Velocity, &mut Position)>) {
    for (vel, mut pos) in query.iter_mut() {
        pos.0 += vel.0 * time.delta_seconds_f64();
    }
}

fn update_transform_from_position(mut query: Query<(&Position, &mut Transform)>) {
    for (pos, mut t) in query.iter_mut() {
        t.translation = pos.0.as_f32();
    }
}
