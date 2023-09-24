use bevy::{math::vec3, prelude::*};
use bevy_rapier3d::prelude::{ExternalForce, ReadMassProperties};
use serde::{Deserialize, Serialize};
use std::ops::{BitOr, BitOrAssign};

#[derive(Copy, Clone, Debug, Default, Reflect, Serialize, Deserialize)]
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

    pub fn intersects(self, other: ThrusterGroup) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn index(self) -> usize {
        assert!(self.0 != 0);
        self.0.trailing_zeros() as usize - 1
    }

    pub fn positive_rotation(axis: usize) -> ThrusterGroup {
        match axis {
            0 => ThrusterGroup::XROT,
            1 => ThrusterGroup::YROT,
            2 => ThrusterGroup::ZROT,
            _ => panic!("Unknown Axis"),
        }
    }

    pub fn negative_rotation(axis: usize) -> ThrusterGroup {
        match axis {
            0 => ThrusterGroup::NXROT,
            1 => ThrusterGroup::NYROT,
            2 => ThrusterGroup::NZROT,
            _ => panic!("Unknown Axis"),
        }
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

#[derive(Reflect, Serialize, Deserialize, Default)]
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
    #[serde(skip_serializing)]
    pub group_thrust: [f32; 12],
    #[serde(skip_serializing)]
    pub groups_to_fire: ThrusterGroup,
}

pub fn reset_thrusters(mut query: Query<&mut Thrusters>) {
    for mut thrusters in query.iter_mut() {
        thrusters.groups_to_fire = ThrusterGroup::NONE;
        for i in 0..12 {
            thrusters.group_thrust[i] = 0.0;
        }
    }
}

pub fn thrusters(
    mut query: Query<(
        &Transform,
        &Thrusters,
        &mut ExternalForce,
        &ReadMassProperties,
    )>,
    mut gizmos: Gizmos,
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

            gizmos.line(
                pos,
                pos + 0.4 * -(magnitude * force.normalize()),
                Color::RED,
            );
        }

        {
            let center_of_mass = transform.transform_point(mass_props.0.local_center_of_mass);
            gizmos.line(
                center_of_mass,
                center_of_mass + vec3(0.0, 0.3, 0.0),
                Color::GREEN,
            );
        }
    }
}

pub fn debug_thruster(query: Query<(&Transform, &Thrusters)>, mut gizmos: Gizmos) {
    for (transform, thrusters) in query.iter() {
        for thruster in &thrusters.thrusters {
            let pos = transform.transform_point(thruster.offset);
            let orientation = (transform.rotation * thruster.direction).mul_vec3(-Vec3::Z);
            let end = pos + 0.3 * orientation;
            gizmos.line(pos, end, Color::BLUE);

            let local_x = transform.rotation * Vec3::X;
            let local_y = transform.rotation * Vec3::Y;
            let local_z = transform.rotation * Vec3::Z;

            gizmos.line(pos, pos + 0.1 * local_x, Color::BLUE);
            gizmos.line(pos, pos - 0.1 * local_x, Color::BLUE);
            gizmos.line(pos, pos + 0.1 * local_y, Color::BLUE);
            gizmos.line(pos, pos - 0.1 * local_y, Color::BLUE);
            gizmos.line(pos, pos + 0.1 * local_z, Color::BLUE);
            gizmos.line(pos, pos - 0.1 * local_z, Color::BLUE);
        }
    }
}
