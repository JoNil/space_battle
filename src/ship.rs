use bevy::{
    input::Input,
    math::{vec3, Quat, Vec3},
    prelude::{Color, Component, KeyCode, Query, ReflectComponent, Res, ResMut, Transform},
    reflect::{FromReflect, Reflect, ReflectDeserialize, ReflectSerialize},
};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::prelude::{ExternalForce, ReadMassProperties};
use serde::{Deserialize, Serialize};
use std::ops::{BitOr, BitOrAssign};

#[derive(Copy, Clone, Default, Reflect, FromReflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct ThrusterGroup(u32);

impl ThrusterGroup {
    pub const NONE: ThrusterGroup = ThrusterGroup(0);
    pub const FORWARD: ThrusterGroup = ThrusterGroup(1 << 0);
    pub const BACKWARD: ThrusterGroup = ThrusterGroup(1 << 1);
    pub const LEFT: ThrusterGroup = ThrusterGroup(1 << 2);
    pub const RIGHT: ThrusterGroup = ThrusterGroup(1 << 3);
    pub const UP: ThrusterGroup = ThrusterGroup(1 << 4);
    pub const DOWN: ThrusterGroup = ThrusterGroup(1 << 5);
    pub const XROT: ThrusterGroup = ThrusterGroup(1 << 6);
    pub const NXROT: ThrusterGroup = ThrusterGroup(1 << 7);
    pub const YROT: ThrusterGroup = ThrusterGroup(1 << 8);
    pub const NYROT: ThrusterGroup = ThrusterGroup(1 << 9);
    pub const ZROT: ThrusterGroup = ThrusterGroup(1 << 10);
    pub const NZROT: ThrusterGroup = ThrusterGroup(1 << 11);

    fn intersects(self, other: ThrusterGroup) -> bool {
        (self.0 & other.0) != 0
    }

    fn index(self) -> usize {
        assert!(self.0 != 0);
        self.0.trailing_zeros() as usize - 1
    }
}

impl BitOrAssign for ThrusterGroup {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitOr for ThrusterGroup {
    type Output = ThrusterGroup;

    fn bitor(self, rhs: Self) -> Self::Output {
        ThrusterGroup(self.0 | rhs.0)
    }
}

#[derive(Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub struct PlayerShip;

#[derive(Reflect, FromReflect, Serialize, Deserialize, Default)]
#[reflect(Serialize, Deserialize)]
pub struct Thruster {
    pub offset: Vec3,
    pub direction: Quat,
    pub thrust: f32,
    pub group: ThrusterGroup,
}

#[derive(Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Thrusters {
    pub thrusters: Vec<Thruster>,
    pub group_thrust: [f32; 12],
    pub groups_to_fire: ThrusterGroup,
}

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct OrientationRegulator {
    target: Quat,
    gain: f32,
}

impl Default for OrientationRegulator {
    fn default() -> Self {
        Self {
            target: Default::default(),
            gain: 1.0,
        }
    }
}

pub fn reset_thrusters(mut query: Query<&mut Thrusters>) {
    for mut thrusters in query.iter_mut() {
        thrusters.groups_to_fire = ThrusterGroup::NONE;
        for i in 0..12 {
            thrusters.group_thrust[i] = 0.0;
        }
    }
}

pub fn orientation_regulator(
    mut query: Query<(&Transform, &mut Thrusters, &OrientationRegulator)>,
) {
    for (transfrom, mut thrusters, regulator) in query.iter_mut() {
        let mut groups_to_fire = ThrusterGroup::NONE;

        let differense = regulator.target * transfrom.rotation.inverse();
        let differense = differense.to_axis_angle();

        let x_error = differense.0.x * differense.1;
        let y_error = differense.0.y * differense.1;
        let z_error = differense.0.z * differense.1;

        let x_error_abs = x_error.abs();
        let y_error_abs = y_error.abs();
        let z_error_abs = z_error.abs();

        // Calculate the needed thrust magnitudes and determine which thruster groups to fire
        if x_error_abs > 0.0 {
            let x_thrust = regulator.gain * x_error_abs;
            if x_error > 0.0 {
                groups_to_fire |= ThrusterGroup::XROT;
                thrusters.group_thrust[ThrusterGroup::XROT.index()] = x_thrust;
            } else {
                groups_to_fire |= ThrusterGroup::NXROT;
                thrusters.group_thrust[ThrusterGroup::NXROT.index()] = x_thrust;
            }
        }

        if y_error_abs > 0.0 {
            let y_thrust = regulator.gain * y_error_abs;
            if y_error > 0.0 {
                groups_to_fire |= ThrusterGroup::YROT;
                thrusters.group_thrust[ThrusterGroup::YROT.index()] = y_thrust;
            } else {
                groups_to_fire |= ThrusterGroup::NYROT;
                thrusters.group_thrust[ThrusterGroup::NYROT.index()] = y_thrust;
            }
        }

        if z_error_abs > 0.0 {
            let z_thrust = regulator.gain * z_error_abs;
            if z_error > 0.0 {
                groups_to_fire |= ThrusterGroup::ZROT;
                thrusters.group_thrust[ThrusterGroup::ZROT.index()] = z_thrust;
            } else {
                groups_to_fire |= ThrusterGroup::NZROT;
                thrusters.group_thrust[ThrusterGroup::NZROT.index()] = z_thrust;
            }
        }

        thrusters.groups_to_fire |= groups_to_fire;
    }
}

pub fn player_thrusters(
    mut query: Query<(&PlayerShip, &mut Thrusters)>,
    keyboard: Res<Input<KeyCode>>,
) {
    let mut groups_to_fire = ThrusterGroup::NONE;

    if keyboard.pressed(KeyCode::W) {
        groups_to_fire |= ThrusterGroup::FORWARD;
    }

    if keyboard.pressed(KeyCode::S) {
        groups_to_fire |= ThrusterGroup::BACKWARD;
    }

    if keyboard.pressed(KeyCode::D) {
        groups_to_fire |= ThrusterGroup::RIGHT;
    }

    if keyboard.pressed(KeyCode::A) {
        groups_to_fire |= ThrusterGroup::LEFT;
    }

    if keyboard.pressed(KeyCode::Space) {
        groups_to_fire |= ThrusterGroup::UP;
    }

    if keyboard.pressed(KeyCode::LShift) {
        groups_to_fire |= ThrusterGroup::DOWN;
    }

    if keyboard.pressed(KeyCode::Numpad6) {
        groups_to_fire |= ThrusterGroup::NYROT;
    }

    if keyboard.pressed(KeyCode::Numpad4) {
        groups_to_fire |= ThrusterGroup::YROT;
    }

    if keyboard.pressed(KeyCode::Numpad8) {
        groups_to_fire |= ThrusterGroup::NXROT;
    }

    if keyboard.pressed(KeyCode::Numpad5) {
        groups_to_fire |= ThrusterGroup::XROT;
    }

    if keyboard.pressed(KeyCode::Numpad9) {
        groups_to_fire |= ThrusterGroup::NZROT;
    }

    if keyboard.pressed(KeyCode::Numpad7) {
        groups_to_fire |= ThrusterGroup::ZROT;
    }

    for (_, mut thrusters) in query.iter_mut() {
        thrusters.groups_to_fire |= groups_to_fire;
    }
}

pub fn thrusters(
    mut query: Query<(
        &Transform,
        &Thrusters,
        &mut ExternalForce,
        &ReadMassProperties,
    )>,
    mut lines: ResMut<DebugLines>,
) {
    for (transform, thrusters, mut forces, mass_props) in query.iter_mut() {
        *forces = ExternalForce::default();

        for thruster in thrusters
            .thrusters
            .iter()
            .filter(|thruster| thruster.group.intersects(thrusters.groups_to_fire))
        {
            let mut magnitude = 0.0;
            for i in 0..12 {
                if thruster.group.0 & (1 << (i + 1)) > 0 {
                    magnitude += thrusters.group_thrust[i];
                }
            }

            if magnitude == 0.0 {
                magnitude = 1.0;
            } else {
                magnitude = magnitude.clamp(0.0, 1.0);
            }

            let pos = transform.transform_point(thruster.offset);
            let center_of_mass = transform.transform_point(mass_props.0.local_center_of_mass);
            let force = magnitude
                * thruster.thrust
                * -(transform.rotation * thruster.direction).mul_vec3(-Vec3::Z);

            *forces += ExternalForce::at_point(force, pos, center_of_mass);

            lines.line_colored(
                pos,
                pos + 0.4 * -(magnitude * force.normalize()),
                0.0,
                Color::RED,
            );
        }

        {
            let center_of_mass = transform.transform_point(mass_props.0.local_center_of_mass);
            lines.line_colored(
                center_of_mass,
                center_of_mass + vec3(0.0, 0.3, 0.0),
                0.0,
                Color::GREEN,
            );
        }
    }
}

pub fn debug_thruster(query: Query<(&Transform, &Thrusters)>, mut lines: ResMut<DebugLines>) {
    for (transform, thrusters) in query.iter() {
        for thruster in &thrusters.thrusters {
            let pos = transform.transform_point(thruster.offset);
            let orientation = (transform.rotation * thruster.direction).mul_vec3(-Vec3::Z);
            let end = pos + 0.3 * orientation;
            lines.line(pos, end, 0.0);

            let local_x = transform.rotation * Vec3::X;
            let local_y = transform.rotation * Vec3::Y;
            let local_z = transform.rotation * Vec3::Z;

            lines.line(pos, pos + 0.1 * local_x, 0.0);
            lines.line(pos, pos - 0.1 * local_x, 0.0);
            lines.line(pos, pos + 0.1 * local_y, 0.0);
            lines.line(pos, pos - 0.1 * local_y, 0.0);
            lines.line(pos, pos + 0.1 * local_z, 0.0);
            lines.line(pos, pos - 0.1 * local_z, 0.0);
        }
    }
}
