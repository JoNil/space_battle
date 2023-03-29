use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    math::{Quat, Vec3},
    pbr::PointLightBundle,
    prelude::{
        shape, AssetServer, Assets, Color, CoreSet, IntoSystemConfig, Mesh, PbrBundle,
        StandardMaterial,
    },
    prelude::{
        App, BuildChildren, Camera3dBundle, Commands, ComputedVisibility, Msaa, Res, ResMut,
        Transform, Visibility,
    },
    scene::SceneBundle,
    transform::TransformBundle,
    DefaultPlugins,
};
use bevy_editor_pls::{AddEditorWindow, EditorPlugin};
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::{
    prelude::{
        AdditionalMassProperties, Collider, ExternalForce, GravityScale, NoUserData,
        RapierPhysicsPlugin, ReadMassProperties, RigidBody,
    },
    render::RapierDebugRenderPlugin,
};
use ship::{
    debug_thruster, orientation_regulator, player_thrusters, reset_thrusters, thrusters,
    OrientationRegulator, PlayerShip, Thruster, ThrusterGroup, Thrusters,
};
use std::f32::consts::PI;
use ui::physics_debug_panel::PhysicsProfilingPanel;

mod ship;
mod ui;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: false,
            enabled: true,
            ..Default::default()
        })
        .add_plugin(DebugLinesPlugin::default())
        .add_system(reset_thrusters.in_base_set(CoreSet::PreUpdate))
        .add_system(orientation_regulator)
        .add_system(player_thrusters)
        .add_system(thrusters)
        .add_system(debug_thruster)
        .add_startup_system(add_test_objects)
        .add_startup_system(setup_physics)
        .register_type::<ThrusterGroup>()
        .register_type::<PlayerShip>()
        .register_type::<Thruster>()
        .register_type::<Thrusters>()
        .register_type::<OrientationRegulator>()
        .add_editor_window::<PhysicsProfilingPanel>()
        .run();
}

fn add_test_objects(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_empty()
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.0))
        .insert(AdditionalMassProperties::Mass(100.0))
        .insert(ReadMassProperties::default())
        .insert(ExternalForce::default())
        .insert(Collider::cuboid(1.0, 1.0, 1.0))
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
        .insert(OrientationRegulator::default())
        .insert(Visibility::default())
        .insert(ComputedVisibility::default())
        .with_children(|p| {
            p.spawn(SceneBundle {
                scene: asset_server.load("models/space_ship/scene.gltf#Scene0"),
                ..Default::default()
            });
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
}

pub fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Box::new(2.0, 2.0, 2.0)));

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
