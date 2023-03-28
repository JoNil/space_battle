use bevy::{
    ecs::event::ManualEventReader,
    input::mouse::MouseMotion,
    prelude::{
        App, Component, Events, Input, KeyCode, Plugin, Quat, Query, Res, ResMut, Resource,
        Transform, Vec3, With,
    },
    time::Time,
    window::{CursorGrabMode, PrimaryWindow, Window},
};

#[derive(Default, Resource)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
        }
    }
}

#[derive(Component)]
pub struct FlyCam;

fn toggle_grab_cursor(window: &mut Window) {
    if window.cursor.grab_mode == CursorGrabMode::None {
        window.cursor.grab_mode = CursorGrabMode::Locked;
    } else {
        window.cursor.grab_mode = CursorGrabMode::None;
    }
    window.cursor.visible = !window.cursor.visible;
}

fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = window_query.get_single() {
        for (_camera, mut transform) in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                if window.cursor.grab_mode == CursorGrabMode::Locked {
                    match key {
                        KeyCode::W => velocity += forward,
                        KeyCode::S => velocity -= forward,
                        KeyCode::A => velocity -= right,
                        KeyCode::D => velocity += right,
                        KeyCode::Space => velocity += Vec3::Y,
                        KeyCode::LShift => velocity -= Vec3::Y,
                        _ => (),
                    }
                }
            }

            velocity = velocity.normalize();

            if !velocity.is_nan() {
                transform.translation += velocity * time.delta_seconds() * settings.speed
            }
        }
    }
}

fn player_look(
    settings: Res<MovementSettings>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<(&FlyCam, &mut Transform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let state = &mut *state;

    if let Ok(window) = window_query.get_single() {
        for (_camera, mut transform) in query.iter_mut() {
            for ev in state.reader_motion.iter(&motion) {
                if window.cursor.grab_mode == CursorGrabMode::Locked {
                    // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                    let window_scale = window.height().min(window.width());

                    state.pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                    state.yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                }

                state.pitch = state.pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation = Quat::from_axis_angle(Vec3::Y, state.yaw)
                    * Quat::from_axis_angle(Vec3::X, state.pitch);
            }
        }
    }
}

fn cursor_grab(
    keys: Res<Input<KeyCode>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = window_query.get_single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(window.as_mut());
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_system(player_move)
            .add_system(player_look)
            .add_system(cursor_grab);
    }
}
