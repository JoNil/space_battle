use std::f32::consts::PI;

use bevy::{
    math::{Quat, Vec3},
    pbr::LightBundle,
    prelude::{
        App, Assets, BuildChildren, Commands, Mesh, Msaa, PerspectiveCameraBundle, Res, ResMut,
        SpawnSceneAsChildCommands, StandardMaterial, Transform,
    },
    prelude::{
        AppBuilder, AssetServer, Color, CoreStage, IntoSystem, Plugin, Query, TextBundle,
        UiCameraBundle,
    },
    text::{Text, TextSection, TextStyle},
    ui::{AlignSelf, Style},
    DefaultPlugins,
};
use bevy_egui::{
    egui::{self, DragValue},
    EguiContext, EguiPlugin,
};
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::{
    physics::{
        ColliderBundle, ColliderPositionSync, NoUserData, RapierPhysicsPlugin, RigidBodyBundle,
    },
    prelude::{ColliderShape, PhysicsPipeline, RigidBodyForces},
    render::{ColliderDebugRender, RapierRenderPlugin},
};
use camera::{CameraPlugin, FlyCam, MovementSettings};
use ship::{
    debug_thruster, orientation_regulator, player_thrusters, thrusters, OrientationRegulator,
    PlayerShip, Thruster, ThrusterGroup, Thrusters,
};

mod camera;
mod ship;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(MovementSettings {
            sensitivity: 0.000075, // default: 0.00012
            speed: 12.0,           // default: 12.0
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_plugin(DebugUiPlugin)
        .add_system(orientation_regulator.system())
        .add_system(player_thrusters.system())
        .add_system(thrusters.system())
        .add_system(ui_example.system())
        .add_system(debug_thruster.system())
        .add_startup_system(add_test_objects.system())
        .add_startup_system(setup_physics.system())
        .run();
}

fn ui_example(egui_context: Res<EguiContext>, mut query: Query<&mut Thrusters>) {
    egui::Window::new("Hello").show(egui_context.ctx(), |ui| {
        for mut thrusters in query.iter_mut() {
            for thruster in &mut thrusters.thrusters {
                ui.horizontal(|ui| {
                    ui.label("Thruster");
                    ui.add(DragValue::new(&mut thruster.offset.x).speed(0.01));
                    ui.add(DragValue::new(&mut thruster.offset.y).speed(0.01));
                    ui.add(DragValue::new(&mut thruster.offset.z).speed(0.01));
                });
            }
        }
    });
}

fn add_test_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    {
        // Spawn player ship

        let mut rigid_body = RigidBodyBundle {
            position: [0.0, 0.0, 0.0].into(),
            forces: RigidBodyForces {
                gravity_scale: 0.0,
                ..RigidBodyForces::default()
            },
            ..RigidBodyBundle::default()
        };

        let collider = ColliderBundle {
            shape: ColliderShape::cuboid(1.0, 1.0, 1.0),
            ..ColliderBundle::default()
        };

        rigid_body
            .mass_properties
            .local_mprops
            .set_mass(100.0, true);

        commands
            .spawn()
            .insert_bundle(rigid_body)
            .insert_bundle(collider)
            .insert(ColliderDebugRender::with_id(0))
            .insert(ColliderPositionSync::Discrete)
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
            .with_children(|p| {
                p.spawn_scene(asset_server.load("models/space_ship/scene.gltf#Scene0"));
                p.spawn_bundle(PerspectiveCameraBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, 1.0, 8.0))
                        .looking_at(Vec3::default(), Vec3::Y),
                    ..Default::default()
                })
                .insert(FlyCam);
            });
    }
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 5.0)),
        ..Default::default()
    });
}

pub fn setup_physics(mut commands: Commands) {
    // Create the cubes
    let num = 8;
    let rad = 1.0;

    let shift = rad * 2.0 + rad;
    let centerx = shift * (num / 2) as f32;
    let centery = shift / 2.0;
    let centerz = shift * (num / 2) as f32;

    let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;
    let mut color = 0;

    for j in 0usize..20 {
        for i in 0..num {
            for k in 0usize..num {
                let x = i as f32 * shift - centerx + offset;
                let y = j as f32 * shift + centery - 15.0;
                let z = k as f32 * shift - centerz + offset;
                color += 1;

                // Build the rigid body.
                let mut rigid_body = RigidBodyBundle {
                    position: [x, y, z].into(),
                    forces: RigidBodyForces {
                        gravity_scale: 0.0,
                        ..RigidBodyForces::default()
                    },
                    ..RigidBodyBundle::default()
                };

                rigid_body.mass_properties.local_mprops.set_mass(1.0, true);

                let collider = ColliderBundle {
                    shape: ColliderShape::cuboid(rad, rad, rad),
                    ..ColliderBundle::default()
                };

                commands
                    .spawn()
                    .insert_bundle(rigid_body)
                    .insert_bundle(collider)
                    .insert(ColliderDebugRender::with_id(color))
                    .insert(ColliderPositionSync::Discrete);
            }
        }

        offset -= 0.05 * rad * (num as f32 - 1.0);
    }
}

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_ui.system())
            .add_system_to_stage(CoreStage::Update, text_update_system.system());
    }
}

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server
        .load(format!("{}/assets/FiraSans-Bold.ttf", env!("CARGO_MANIFEST_DIR")).as_str());
    commands
        // 2d camera
        .spawn()
        .insert_bundle(UiCameraBundle::default());
    // texture
    commands.spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            ..Default::default()
        },
        text: Text {
            sections: vec![TextSection {
                value: "Physics time0.1234567890".to_string(),
                style: TextStyle {
                    font: font_handle,
                    font_size: 15.0,
                    color: Color::BLACK,
                },
            }],
            ..Default::default()
        },
        ..Default::default()
    });
}

pub fn text_update_system(pipeline: Res<PhysicsPipeline>, mut query: Query<&mut Text>) {
    let profile_string = format!(
        r#"Total: {:.2}ms
Collision detection: {:.2}ms
|_ Broad-phase: {:.2}ms
   Narrow-phase: {:.2}ms
Island computation: {:.2}ms
Solver: {:.2}ms
|_ Velocity assembly: {:.2}ms
   Velocity resolution: {:.2}ms
   Velocity integration: {:.2}ms
   Position assembly: {:.2}ms
   Position resolution: {:.2}ms
CCD: {:.2}ms
|_ # of substeps: {}
   TOI computation: {:.2}ms
   Broad-phase: {:.2}ms
   Narrow-phase: {:.2}ms
   Solver: {:.2}ms"#,
        pipeline.counters.step_time(),
        pipeline.counters.collision_detection_time(),
        pipeline.counters.broad_phase_time(),
        pipeline.counters.narrow_phase_time(),
        pipeline.counters.island_construction_time(),
        pipeline.counters.solver_time(),
        pipeline.counters.solver.velocity_assembly_time.time(),
        pipeline.counters.velocity_resolution_time(),
        pipeline.counters.solver.velocity_update_time.time(),
        pipeline.counters.solver.position_assembly_time.time(),
        pipeline.counters.position_resolution_time(),
        pipeline.counters.ccd_time(),
        pipeline.counters.ccd.num_substeps,
        pipeline.counters.ccd.toi_computation_time.time(),
        pipeline.counters.ccd.broad_phase_time.time(),
        pipeline.counters.ccd.narrow_phase_time.time(),
        pipeline.counters.ccd.solver_time.time(),
    );

    for mut text in query.iter_mut() {
        text.sections[0].value = profile_string.clone();
    }
}
