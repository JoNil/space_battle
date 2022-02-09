use bevy::{
    input::Input,
    math::{Quat, Vec3},
    prelude::{Component, KeyCode, Query, Res, Transform},
};
use bevy_rapier3d::prelude::{RigidBodyForcesComponent, RigidBodyMassPropsComponent};
use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct ThrusterGroup: u32 {
        const NONE     = 0b0000_0000_0000;
        const FORWARD  = 0b0000_0000_0001;
        const BACKWARD = 0b0000_0000_0010;
        const LEFT     = 0b0000_0000_0100;
        const RIGHT    = 0b0000_0000_1000;
        const UP       = 0b0000_0001_0000;
        const DOWN     = 0b0000_0010_0000;
        const XROT     = 0b0000_0100_0000;
        const NXROT    = 0b0000_1000_0000;
        const YROT     = 0b0001_0000_0000;
        const NYROT    = 0b0010_0000_0000;
        const ZROT     = 0b0100_0000_0000;
        const NZROT    = 0b1000_0000_0000;
    }
}

#[derive(Component)]
pub struct PlayerShip;

#[derive(Default)]
pub struct Thruster {
    pub offset: Vec3,
    pub direction: Quat,
    pub thrust: f32,
    pub group: ThrusterGroup,
}

#[derive(Component, Default)]
pub struct Thrusters {
    pub thrusters: Vec<Thruster>,
    pub group_thrust: [f32; 12],
    pub groups_to_fire: ThrusterGroup,
}

#[derive(Component, Default)]
pub struct OrientationRegulator {
    target: Quat,
}

pub fn orientation_regulator(
    mut query: Query<(&Transform, &mut Thrusters, &OrientationRegulator)>,
) {
    let mut groups_to_fire = ThrusterGroup::NONE;

    for (transfrom, mut thrusters, regulator) in query.iter_mut() {
        let differense = regulator.target * transfrom.rotation.inverse();
        let differense = differense.to_axis_angle();

        let x_error = differense.0.x * differense.1;
        let y_error = differense.0.y * differense.1;
        let z_error = differense.0.z * differense.1;

        println!("{:?}", y_error);
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
        &mut Thrusters,
        &RigidBodyMassPropsComponent,
        &mut RigidBodyForcesComponent,
    )>,
    //mut lines: ResMut<DebugLines>,
) {
    for (transform, mut thrusters, rb_mprops, mut forces) in query.iter_mut() {
        for thruster in thrusters
            .thrusters
            .iter()
            .filter(|thruster| thruster.group.intersects(thrusters.groups_to_fire))
        {
            let pos = transform.mul_vec3(thruster.offset);
            let force =
                thruster.thrust * -(transform.rotation * thruster.direction).mul_vec3(-Vec3::Z);

            forces.apply_force_at_point(rb_mprops, force.into(), pos.into());

            //lines.line_colored(pos, pos + 0.2 * force.normalize(), 0.0, Color::RED);
        }

        thrusters.groups_to_fire = ThrusterGroup::NONE;
    }
}

/*pub fn debug_thruster(query: Query<(&Transform, &Thrusters)>, mut lines: ResMut<DebugLines>) {
    for (transform, thrusters) in query.iter() {
        for thruster in &thrusters.thrusters {
            let pos = transform.mul_vec3(thruster.offset);
            let orientaion = (transform.rotation * thruster.direction).mul_vec3(-Vec3::Z);
            let end = pos + 0.3 * orientaion;
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
}*/
