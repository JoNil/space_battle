use bevy::{
    input::Input,
    math::{DVec3, Vec3},
    pbr::{LightBundle, PbrBundle},
    prelude::{
        info, App, Assets, Commands, KeyCode, Mesh, Msaa, PerspectiveCameraBundle, Res, ResMut,
        SpawnSceneCommands, StandardMaterial, Transform,
    },
    prelude::{
        shape, AppBuilder, AssetServer, Color, CoreStage, IntoSystem, Plugin, Query, TextBundle,
        UiCameraBundle,
    },
    text::{Text, TextSection, TextStyle},
    ui::{AlignSelf, Style},
    DefaultPlugins,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use bevy_rapier3d::{
    physics::{
        ColliderBundle, ColliderPositionSync, NoUserData, RapierPhysicsPlugin, RigidBodyBundle,
    },
    prelude::{ColliderShape, PhysicsPipeline, RigidBodyForces, RigidBodyVelocity},
    render::{ColliderDebugRender, RapierRenderPlugin},
};
use physics::{Mass, PhysicsBundle, PhysicsPlugin, Position, Velocity};
use ship::ShipBundle;

mod physics;
mod ship;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(MovementSettings {
            sensitivity: 0.000075, // default: 0.00012
            speed: 12.0,           // default: 12.0
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(NoCameraPlayerPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_plugin(DebugUiPlugin)
        .add_system(keyboard_input_system.system())
        .add_system(ui_example.system())
        .add_startup_system(add_test_objects.system())
        .add_startup_system(setup_physics.system())
        .run();
}

fn ui_example(egui_context: Res<EguiContext>, query: Query<&Position>) {
    egui::Window::new("Hello").show(egui_context.ctx(), |ui| {
        for pos in query.iter() {
            ui.label(format!("Mass: {:#?}", pos.0));
        }
    });
}

fn keyboard_input_system(keyboard: Res<Input<KeyCode>>) {
    if keyboard.pressed(KeyCode::A) {
        info!("'A' currently pressed");
    }
}

fn add_test_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.45,
                subdivisions: 32,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("ff1230").unwrap(),
                metallic: 1.0,
                roughness: 0.12,
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert_bundle(PhysicsBundle {
            pos: Position(DVec3::new(0.0, 0.0, -1.0)),
            vel: Velocity(DVec3::new(0.0, 0.0, 0.0)),
            mass: Mass(1.0),
        })
        .insert_bundle(ShipBundle::default());

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.45,
                subdivisions: 32,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("30ff12").unwrap(),
                metallic: 1.0,
                roughness: 0.12,
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert_bundle(PhysicsBundle {
            pos: Position(DVec3::new(1.0, 0.0, 0.0)),
            vel: Velocity(DVec3::new(1.0, 0.0, 0.0)),
            mass: Mass(1.0),
        })
        .insert_bundle(ShipBundle::default());

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.45,
                subdivisions: 32,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("1230ff").unwrap(),
                metallic: 1.0,
                roughness: 0.12,
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert_bundle(PhysicsBundle {
            pos: Position(DVec3::new(-1.0, 0.0, 0.0)),
            vel: Velocity(DVec3::new(-1.0, 0.0, 0.0)),
            mass: Mass(1.0),
        })
        .insert_bundle(ShipBundle::default());

    commands.spawn_scene(
        asset_server.load(
            format!(
                "{}/assets/models/space_ship/scene.gltf#Scene0",
                env!("CARGO_MANIFEST_DIR")
            )
            .as_str(),
        ),
    );

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 5.0)),
        ..Default::default()
    });

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 8.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCam);
}

pub fn setup_physics(mut commands: Commands) {
    /*
     * Create the cubes
     */
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
                let y = j as f32 * shift + centery + 3.0;
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

    {
        // Build the rigid body.
        let mut rigid_body = RigidBodyBundle {
            position: [-20.0, 10.0, 30.0].into(),
            velocity: RigidBodyVelocity {
                linvel: Vec3::new(0.0, 30.0, -70.0).into(),
                ..RigidBodyVelocity::default()
            },
            forces: RigidBodyForces {
                gravity_scale: 0.0,
                ..RigidBodyForces::default()
            },
            ..RigidBodyBundle::default()
        };

        rigid_body
            .mass_properties
            .local_mprops
            .set_mass(100.0, true);

        let collider = ColliderBundle {
            shape: ColliderShape::cuboid(3.0 * rad, 3.0 * rad, 3.0 * rad),
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
                    ..Default::default()
                },
                ..Default::default()
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
