use super::thrusters::Thrusters;
use bevy::{math::vec3, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Component, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct MaxTorque {
    #[serde(skip_serializing)]
    pub positive_torque: Vec3,
    #[serde(skip_serializing)]
    pub negative_torque: Vec3,
}

pub fn update_max_torque(mut query: Query<(&Thrusters, &mut MaxTorque)>) {
    for (thrusters, mut max_torque) in query.iter_mut() {
        let max_torque = max_torque.as_mut();

        let positive_torque = &mut max_torque.positive_torque;
        *positive_torque = Vec3::ZERO;

        let negative_torque = &mut max_torque.negative_torque;
        *negative_torque = Vec3::ZERO;

        for thruster in &thrusters.thrusters {
            let local_force = thruster
                .direction
                .mul_vec3(thruster.thrust * vec3(0.0, 0.0, 1.0));

            let torque = thruster.offset.cross(local_force);

            if torque.x > 0.0 {
                positive_torque.x += torque.x;
            } else {
                negative_torque.x += torque.x.abs();
            }

            if torque.y > 0.0 {
                positive_torque.y += torque.y;
            } else {
                negative_torque.y += torque.y.abs();
            }

            if torque.z > 0.0 {
                positive_torque.z += torque.z;
            } else {
                negative_torque.z += torque.z.abs();
            }
        }
    }
}
