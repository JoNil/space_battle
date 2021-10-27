use bevy::{
    ecs::bundle::Bundle,
    math::{Quat, Vec3},
};

#[derive(Bundle, Default)]
pub struct ShipBundle {
    pub thrusters: Thrusters,
}

#[derive(Default)]
pub struct Thruster {
    pub offset: Vec3,
    pub direction: Quat,
    pub throttle: f32,
    pub thrust: f32,
}

#[derive(Default)]
pub struct Thrusters(pub Vec<Thruster>);
