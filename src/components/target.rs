use super::orientation_regulator::OrientationRegulator;
use bevy::{math::Vec3, prelude::*};

#[derive(Component)]
pub struct Target;

pub fn target_update_system(
    targets: Query<&Transform, With<Target>>,
    mut regulators: Query<(&mut OrientationRegulator, &Transform)>,
) {
    for (mut regulator, source_transform) in regulators.iter_mut() {
        for target_transform in targets.iter() {
            let goal_transform =
                source_transform.looking_at(target_transform.translation, Vec3::ZERO);
            regulator.update_target(goal_transform.rotation);
        }
    }
}
