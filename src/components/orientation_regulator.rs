use bevy::{math::vec3, prelude::*};
use bevy_rapier3d::prelude::*;
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
            target: Quat::from_euler(EulerRot::XYZ, 0.0, 0.5, 0.0),
            target_angvel: Default::default(),
            local_angvel: Default::default(),
            p_gain: 10.0,
            i_gain: 0.0,
            d_gain: 0.0,
            prev_error: Vec3::ZERO,
            integral_error: Vec3::ZERO,
            enable: true,
            angle_mode: true,
        }
    }
}

fn positive(v: f32) -> Option<f32> {
    if v > 0.0 {
        Some(v)
    } else {
        None
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

    if dbg!(cube_1) > 0.0 {
        let time_to_target_1 = -q + cube_1.sqrt();
        let time_to_target_2 = -q - cube_1.sqrt();

        if time_to_target_1 > 0.0 {
            time_to_target = dbg!(time_to_target_1).min(time_to_target);
        }

        if time_to_target_2 > 0.0 {
            time_to_target = dbg!(time_to_target_2).min(time_to_target);
        }
    }

    if dbg!(cube_2) > 0.0 {
        let time_to_target_1 = q + cube_2.sqrt();
        let time_to_target_2 = q - cube_2.sqrt();

        if time_to_target_1 > 0.0 {
            time_to_target = dbg!(time_to_target_1).min(time_to_target);
        }

        if time_to_target_2 > 0.0 {
            time_to_target = dbg!(time_to_target_2).min(time_to_target);
        }
    }

    let res = 10.0
        * if time_to_stop.abs() > time_to_target.abs() {
            -angular_velocity.signum()
        } else {
            dir
        };

    info!("ttt {time_to_target:.3} tts {time_to_stop:.3} ra {remaning_angle:.3} ta {target_angle:.3} a {angle:.3} res {res}");
    res
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
                    max_torque
                        .positive_torque
                        .y
                        .min(max_torque.negative_torque.y),
                    mass_props.get().principal_inertia.y,
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

fn angle_difference(a: f32, b: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    f32::rem_euclid(a + PI - b, TAU) - PI
}
