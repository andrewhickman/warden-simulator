#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{prelude::*, window::WindowPlugin};

use wdn_physics::PhysicsPlugin as WdnPhysicsPlugin;
use wdn_physics::tile::layer::TileLayer;
use wdn_render::RenderPlugin as WdnRenderPlugin;
use wdn_save::SavePlugin as WdnSavePlugin;
use wdn_tasks::TasksPlugin as WdnTasksPlugin;
use wdn_ui::UiPlugin as WdnUiPlugin;
use wdn_world::WorldPlugin as WdnWorldPlugin;
use wdn_world::pawn::{Pawn, PawnAction};

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Warden Simulator".to_string(),
                    canvas: Some("#bevy".to_owned()),
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            WdnPhysicsPlugin,
            WdnWorldPlugin,
            WdnTasksPlugin,
            WdnSavePlugin,
            WdnRenderPlugin,
            WdnUiPlugin,
        ))
        .add_systems(Startup, spawn_pawn)
        .add_systems(Update, handle_pawn_input)
        .run();
}

fn spawn_pawn(mut commands: Commands) {
    commands.spawn(Camera2d);
    let layer = commands
        .spawn((
            TileLayer::default(),
            Transform::from_scale(Vec3::splat(100.0)),
        ))
        .id();
    commands.spawn((
        Pawn::default(),
        ChildOf(layer),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn handle_pawn_input(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    pawn_query: Single<(&mut PawnAction, &GlobalTransform), With<Pawn>>,
) {
    let (mut action, pawn_transform) = pawn_query.into_inner();

    let (camera, camera_transform) = camera_query.into_inner();

    // Check for attack inputs first (they take priority)
    if keys.just_pressed(KeyCode::KeyQ) {
        *action = PawnAction::AttackLeft;
        return;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        *action = PawnAction::AttackRight;
        return;
    }

    // Handle movement towards cursor on left click
    if mouse.pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                let pawn_pos = pawn_transform.translation().truncate();
                let direction = world_pos - pawn_pos;

                if direction.length_squared() > 0.01 {
                    // Calculate the target angle
                    let target_angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

                    // Get current rotation (assuming Z-axis rotation)
                    let current_rotation = pawn_transform.to_scale_rotation_translation().1;
                    let (_, _, current_angle) = current_rotation.to_euler(EulerRot::XYZ);

                    // Normalize angles to [-PI, PI]
                    let normalize_angle = |angle: f32| {
                        let mut a = angle % std::f32::consts::TAU;
                        if a > std::f32::consts::PI {
                            a -= std::f32::consts::TAU;
                        } else if a < -std::f32::consts::PI {
                            a += std::f32::consts::TAU;
                        }
                        a
                    };

                    let angle_diff = normalize_angle(target_angle - current_angle);

                    // Threshold for considering the pawn aligned
                    const ANGLE_THRESHOLD: f32 = 0.1;

                    if angle_diff.abs() > ANGLE_THRESHOLD {
                        // Turn towards the target
                        *action = if angle_diff > 0.0 {
                            PawnAction::TurnLeft
                        } else {
                            PawnAction::TurnRight
                        };
                    } else {
                        // Walk forward when aligned
                        *action = PawnAction::Walk;
                    }
                    return;
                }
            }
        }
    }

    // Default to standing
    *action = PawnAction::Stand;
}
