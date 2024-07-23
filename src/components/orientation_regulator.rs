use super::{
    max_torque::MaxTorque,
    thrusters::{ThrusterGroup, Thrusters},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct OrientationRegulator {
    target: Quat,
    target_angvel: Vec3,
    #[serde(skip_serializing)]
    local_angvel: Vec3,
    p_gain: f32,
    enable: bool,
}

impl Default for OrientationRegulator {
    fn default() -> Self {
        Self {
            target: Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
            target_angvel: Default::default(),
            local_angvel: Default::default(),
            p_gain: 10.0,
            enable: true,
        }
    }
}

fn calculate_target_angular_velocity(
    target_angle: f32,
    angle: f32,
    angular_velocity: f32,
    max_torque: f32,
    angular_inertia: f32,
) -> f32 {
    if angular_inertia == 0.0 {
        return angular_velocity;
    }

    let angular_acceleration = max_torque / angular_inertia;

    let remaning_angle = angle_difference(target_angle, angle);

    let dir = remaning_angle.signum();

    let time_to_stop = angular_velocity.abs() / angular_acceleration;

    let mut time_to_target = 1000.0;

    let q = angular_velocity / (2.0 * angular_acceleration);

    let cube_1 = remaning_angle / angular_acceleration + q * q;
    let cube_2 = -remaning_angle / angular_acceleration + q * q;

    if cube_1 > 0.0 {
        let time_to_target_1 = -q + cube_1.sqrt();
        let time_to_target_2 = -q - cube_1.sqrt();

        if time_to_target_1 > 0.0 {
            time_to_target = time_to_target_1.min(time_to_target);
        }

        if time_to_target_2 > 0.0 {
            time_to_target = time_to_target_2.min(time_to_target);
        }
    }

    if cube_2 > 0.0 {
        let time_to_target_1 = q + cube_2.sqrt();
        let time_to_target_2 = q - cube_2.sqrt();

        if time_to_target_1 > 0.0 {
            time_to_target = time_to_target_1.min(time_to_target);
        }

        if time_to_target_2 > 0.0 {
            time_to_target = time_to_target_2.min(time_to_target);
        }
    }

    10.0 * if time_to_stop.abs() > time_to_target.abs() {
        -angular_velocity.signum()
    } else {
        dir
    }
}

pub fn orientation_regulator(
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
            let angle = Vec3::from(transform.rotation.to_euler(EulerRot::XYZ));
            let target_angle = Vec3::from(regulator.target.to_euler(EulerRot::XYZ));

            for axis in 0..3 {
                regulator.target_angvel[axis] = calculate_target_angular_velocity(
                    target_angle[axis],
                    angle[axis],
                    regulator.local_angvel[axis],
                    max_torque.positive_torque[axis].min(max_torque.negative_torque[axis]),
                    mass_props.get().principal_inertia[axis],
                );
            }

            let mut groups_to_fire = ThrusterGroup::NONE;
            let error = regulator.target_angvel - regulator.local_angvel;

            let error_abs = error.abs();

            let thrust = regulator.p_gain * error_abs;

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

            thrusters.groups_to_fire |= groups_to_fire;
        }
    }
}

fn angle_difference(a: f32, b: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    f32::rem_euclid(a + PI - b, TAU) - PI
}
