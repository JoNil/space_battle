use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, math::vec3, prelude::*};
use bevy_editor_pls::{AddEditorWindow, EditorPlugin};
use bevy_rapier3d::{prelude::*, render::RapierDebugRenderPlugin};
use components::{
    defer_collider_loader::{defer_collider_loader, DeferColliderLoader},
    max_torque::{update_max_torque, MaxTorque},
    orientation_regulator::{orientation_regulator, OrientationRegulator},
    player_ship::{player_thrusters, PlayerShip},
    thrusters::{debug_thruster, reset_thrusters, thrusters, Thruster, ThrusterGroup, Thrusters},
};
use std::f32::consts::PI;
use ui::physics_debug_panel::PhysicsProfilingPanel;

mod components;
mod ui;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(EditorPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin {
            enabled: true,
            ..Default::default()
        })
        .add_systems(Startup, add_test_objects)
        .add_systems(Startup, setup_physics)
        .add_systems(
            Update,
            (
                (
                    (reset_thrusters, update_max_torque),
                    (player_thrusters, orientation_regulator),
                    thrusters,
                    debug_thruster,
                )
                    .chain(),
                defer_collider_loader,
            ),
        )
        .register_type::<ThrusterGroup>()
        .register_type::<PlayerShip>()
        .register_type::<Thruster>()
        .register_type::<Thrusters>()
        .register_type::<MaxTorque>()
        .register_type::<OrientationRegulator>()
        .register_type::<ReadMassProperties>()
        .register_type::<DeferColliderLoader>()
        .add_editor_window::<PhysicsProfilingPanel>()
        .run();
}

fn add_test_objects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: asset_server.load("models/ship.glb#Mesh0/Primitive0"),
            material: asset_server.load("models/ship.glb#Material0"),
            ..Default::default()
        })
        .insert(Name::new("Player"))
        .insert(DeferColliderLoader)
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.0))
        .insert(AdditionalMassProperties::Mass(100.0))
        .insert(ReadMassProperties::default())
        .insert(ExternalForce::default())
        .insert(Velocity::default())
        .insert(Sleeping::disabled())
        .insert(Thrusters {
            thrusters: Vec::from([
                Thruster {
                    offset: Vec3::new(0.0, 0.0, 4.0),
                    direction: Quat::from_axis_angle(Vec3::Y, PI),
                    thrust: 200.0,
                    group: ThrusterGroup::FORWARD,
                },
                Thruster {
                    offset: Vec3::new(0.0, 0.0, -4.0),
                    direction: Quat::from_axis_angle(Vec3::Y, 0.0),
                    thrust: 50.0,
                    group: ThrusterGroup::BACKWARD,
                },
                // Upper pointing to sides
                Thruster {
                    offset: Vec3::new(1.0, 1.0, -4.0),
                    direction: Quat::from_axis_angle(Vec3::Y, -PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::LEFT | ThrusterGroup::YROT | ThrusterGroup::ZROT,
                },
                Thruster {
                    offset: Vec3::new(1.0, 1.0, 4.0),
                    direction: Quat::from_axis_angle(Vec3::Y, -PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::LEFT | ThrusterGroup::NYROT | ThrusterGroup::ZROT,
                },
                Thruster {
                    offset: Vec3::new(-1.0, 1.0, -4.0),
                    direction: Quat::from_axis_angle(Vec3::Y, PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::RIGHT | ThrusterGroup::NYROT | ThrusterGroup::NZROT,
                },
                Thruster {
                    offset: Vec3::new(-1.0, 1.0, 4.0),
                    direction: Quat::from_axis_angle(Vec3::Y, PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::RIGHT | ThrusterGroup::YROT | ThrusterGroup::NZROT,
                },
                // Lower pointing to sides
                Thruster {
                    offset: Vec3::new(1.0, -1.0, -4.0),
                    direction: Quat::from_axis_angle(Vec3::Y, -PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::LEFT | ThrusterGroup::YROT | ThrusterGroup::NZROT,
                },
                Thruster {
                    offset: Vec3::new(1.0, -1.0, 4.0),
                    direction: Quat::from_axis_angle(Vec3::Y, -PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::LEFT | ThrusterGroup::NYROT | ThrusterGroup::NZROT,
                },
                Thruster {
                    offset: Vec3::new(-1.0, -1.0, -4.0),
                    direction: Quat::from_axis_angle(Vec3::Y, PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::RIGHT | ThrusterGroup::NYROT | ThrusterGroup::ZROT,
                },
                Thruster {
                    offset: Vec3::new(-1.0, -1.0, 4.0),
                    direction: Quat::from_axis_angle(Vec3::Y, PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::RIGHT | ThrusterGroup::YROT | ThrusterGroup::ZROT,
                },
                // Upper pointing up
                Thruster {
                    offset: Vec3::new(1.0, 1.0, -4.0),
                    direction: Quat::from_axis_angle(Vec3::X, PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::DOWN | ThrusterGroup::NXROT | ThrusterGroup::NZROT,
                },
                Thruster {
                    offset: Vec3::new(1.0, 1.0, 4.0),
                    direction: Quat::from_axis_angle(Vec3::X, PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::DOWN | ThrusterGroup::XROT | ThrusterGroup::NZROT,
                },
                Thruster {
                    offset: Vec3::new(-1.0, 1.0, -4.0),
                    direction: Quat::from_axis_angle(Vec3::X, PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::DOWN | ThrusterGroup::NXROT | ThrusterGroup::ZROT,
                },
                Thruster {
                    offset: Vec3::new(-1.0, 1.0, 4.0),
                    direction: Quat::from_axis_angle(Vec3::X, PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::DOWN | ThrusterGroup::XROT | ThrusterGroup::ZROT,
                },
                // Lower pointing down
                Thruster {
                    offset: Vec3::new(1.0, -1.0, -4.0),
                    direction: Quat::from_axis_angle(Vec3::X, -PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::UP | ThrusterGroup::XROT | ThrusterGroup::ZROT,
                },
                Thruster {
                    offset: Vec3::new(1.0, -1.0, 4.0),
                    direction: Quat::from_axis_angle(Vec3::X, -PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::UP | ThrusterGroup::NXROT | ThrusterGroup::ZROT,
                },
                Thruster {
                    offset: Vec3::new(-1.0, -1.0, -4.0),
                    direction: Quat::from_axis_angle(Vec3::X, -PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::UP | ThrusterGroup::XROT | ThrusterGroup::NZROT,
                },
                Thruster {
                    offset: Vec3::new(-1.0, -1.0, 4.0),
                    direction: Quat::from_axis_angle(Vec3::X, -PI / 2.0),
                    thrust: 1.0,
                    group: ThrusterGroup::UP | ThrusterGroup::NXROT | ThrusterGroup::NZROT,
                },
            ]),
            ..Default::default()
        })
        .insert(PlayerShip)
        .insert(MaxTorque::default())
        .insert(OrientationRegulator::default())
        .with_children(|p| {
            p.spawn(Camera3dBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 1.0, 8.0))
                    .looking_at(Vec3::default(), Vec3::Y),
                ..Default::default()
            });
        });

    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 5.0)),
        ..Default::default()
    });

    commands
        .spawn(PbrBundle {
            mesh: asset_server.load("models/background_1.glb#Mesh0/Primitive0"),
            material: materials.add(StandardMaterial {
                unlit: true,
                ..Default::default()
            }),
            transform: Transform::from_scale(vec3(1000.0, 1000.0, 1000.0)),
            ..Default::default()
        })
        .insert(Name::new("Background"));
}

pub fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(Cuboid::new(2.0, 2.0, 2.0)));

    let num = 8;
    let rad = 1.0;

    let shift = rad * 2.0 + rad;
    let centerx = shift * (num / 2) as f32;
    let centery = shift / 2.0;
    let centerz = shift * (num / 2) as f32;

    let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;
    let mut color = 0;

    for j in 0..8 {
        for i in 0..num {
            for k in 0..num {
                let x = i as f32 * shift - centerx + offset;
                let y = j as f32 * shift + centery - 15.0;
                let z = k as f32 * shift - centerz + offset;
                color += 1;

                commands
                    .spawn_empty()
                    .insert(PbrBundle {
                        mesh: mesh.clone(),
                        material: materials.add(StandardMaterial {
                            base_color: Color::hsl(
                                color as f32 / (num * num * num) as f32 * 360.0,
                                1.0,
                                0.75,
                            ),
                            metallic: 0.5,
                            perceptual_roughness: 0.5,
                            ..Default::default()
                        }),
                        transform: Transform::from_xyz(x, y, z),
                        ..Default::default()
                    })
                    .insert(RigidBody::Dynamic)
                    .insert(GravityScale(0.0))
                    .insert(AdditionalMassProperties::Mass(1.0))
                    .insert(Collider::cuboid(1.0, 1.0, 1.0));
            }
        }

        offset -= 0.05 * rad * (num as f32 - 1.0);
    }
}
