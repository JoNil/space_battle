use super::thrusters::{ThrusterGroup, Thrusters};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub struct PlayerShip;

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

    if keyboard.pressed(KeyCode::ShiftLeft) {
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
