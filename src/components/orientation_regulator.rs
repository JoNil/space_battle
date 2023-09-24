use bevy::{math::vec3, prelude::*};
use bevy_rapier3d::prelude::{ReadMassProperties, Velocity};
use serde::{Deserialize, Serialize};

use super::{
    max_torque::MaxTorque,
    thrusters::{ThrusterGroup, Thrusters},
};

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct OrientationRegulator {
    target: Quat,
    target_angvel: Vec3,
    #[serde(skip_serializing)]
    local_angvel: Vec3,
    p_gain: f32,
    i_gain: f32,
    d_gain: f32,
    prev_error: Vec3,
    integral_error: Vec3,
    enable: bool,
    angle_mode: bool,
}

impl Default for OrientationRegulator {
    fn default() -> Self {
        Self {
            target: Default::default(),
            target_angvel: Default::default(),
            local_angvel: Default::default(),
            p_gain: 10.0,
            i_gain: 0.0,
            d_gain: 0.0,
            prev_error: Vec3::ZERO,
            integral_error: Vec3::ZERO,
            enable: true,
            angle_mode: false,
        }
    }
}

// https://chat.openai.com/c/1bc5a952-eb65-4f1b-8849-323d84036579
fn calculate_target_angular_velocity(
    target_angle: f32,
    angle: f32,
    angular_velocity: f32,
    max_torque: f32,
    angular_inertia: f32,
) -> f32 {
    // 1. Calculate angular displacement
    let delta_theta = target_angle - angle;

    // 2. Calculate the remaining time to reach the target angle
    let angular_acceleration_due_to_max_torque = max_torque / angular_inertia;
    let remaining_time = (2.0 * delta_theta / angular_acceleration_due_to_max_torque)
        .abs()
        .sqrt();

    // 3. Calculate the required angular acceleration to achieve the angular displacement in the remaining time
    let required_angular_acceleration = 2.0 * delta_theta / (remaining_time * remaining_time);

    // 4. Calculate the torque needed to achieve the required angular acceleration, considering the maximum torque and the moment of inertia
    let desired_torque = angular_inertia * required_angular_acceleration;
    let applied_torque = desired_torque.clamp(-max_torque, max_torque);

    // Update the target angular velocity based on the calculated torque and the current angular velocity
    let delta_angular_velocity = applied_torque / angular_inertia;
    let target_angular_velocity = angular_velocity + delta_angular_velocity;

    target_angular_velocity
}

pub fn orientation_regulator(
    time: Res<Time>,
    mut query: Query<(
        &Transform,
        &Velocity,
        &ReadMassProperties,
        &MaxTorque,
        &mut Thrusters,
        &mut OrientationRegulator,
    )>,
) {
    for (transform, vel, mass_props, max_torque, mut thrusters, mut regulator) in query.iter_mut() {
        regulator.local_angvel = transform.rotation.inverse().mul_vec3(vel.angvel);
        if regulator.enable {
            if regulator.angle_mode {
                let angle = transform.rotation.to_euler(EulerRot::XYZ);

                regulator.target_angvel.y = calculate_target_angular_velocity(
                    regulator.target.y,
                    angle.1,
                    regulator.local_angvel.y,
                    max_torque.negative_torque.y,
                    mass_props.0.principal_inertia.y,
                );
            }

            let mut groups_to_fire = ThrusterGroup::NONE;
            let error = regulator.target_angvel - regulator.local_angvel;

            let error_abs = error.abs();

            let dt = time.delta_seconds();
            let derivative_error = (error - regulator.prev_error) / dt;
            regulator.integral_error = (regulator.integral_error + error * dt)
                .clamp(vec3(-1.0, -1.0, -1.0), vec3(1.0, 1.0, 1.0));

            let thrust = regulator.p_gain * error_abs
                + regulator.i_gain * regulator.integral_error
                + regulator.d_gain * derivative_error;

            for axis in 0..3 {
                if error_abs[axis] > 0.0 {
                    let group = if error[axis] > 0.0 {
                        ThrusterGroup::positive_rotation(axis)
                    } else {
                        ThrusterGroup::negative_rotation(axis)
                    };
                    groups_to_fire |= group;
                    thrusters.group_thrust[group.index()] = thrust[axis];
                }
            }

            regulator.prev_error = error;

            thrusters.groups_to_fire |= groups_to_fire;
        }
    }
}
