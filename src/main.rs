use bevy::{
    math::{DVec3, Vec3},
    pbr::{LightBundle, PbrBundle},
    prelude::{shape, Color, IntoSystem, Query},
    prelude::{
        App, Assets, Commands, Mesh, Msaa, PerspectiveCameraBundle, Res, ResMut, StandardMaterial,
        Transform,
    },
    DefaultPlugins,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use physics::{PhysicsPlugin, Position};

mod physics;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 12.0,          // default: 12.0
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(NoCameraPlayerPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(PhysicsPlugin)
        .add_startup_system(add_test_objects.system())
        .add_startup_system(setup.system())
        .add_system(ui_example.system())
        .run();
}

fn ui_example(egui_context: Res<EguiContext>, query: Query<&Position>) {
    egui::Window::new("Hello").show(egui_context.ctx(), |ui| {
        for pos in query.iter() {
            ui.label(format!("Position: {:?}", pos.pos));
        }
    });
}

struct Ship;

fn add_test_objects(mut commands: Commands) {
    commands.spawn().insert(Ship).insert(Position {
        pos: DVec3::new(0.0, 0.0, 0.0),
    });
    commands.spawn().insert(Ship).insert(Position {
        pos: DVec3::new(1.0, 0.0, 0.0),
    });
    commands.spawn().insert(Ship).insert(Position {
        pos: DVec3::new(-1.0, 0.0, 0.0),
    });
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
    for y in -2..=2 {
        for x in -5..=5 {
            let x01 = (x + 5) as f32 / 10.0;
            let y01 = (y + 2) as f32 / 4.0;
            // sphere
            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.45,
                    subdivisions: 32,
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::hex("ffd891").unwrap(),
                    // vary key PBR parameters on a grid of spheres to show the effect
                    metallic: y01,
                    roughness: x01,
                    ..Default::default()
                }),
                transform: Transform::from_xyz(x as f32, y as f32, 0.0),
                ..Default::default()
            });
        }
    }
    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 5.0)),
        ..Default::default()
    });
    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 8.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCam);
}
