#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    app::TaskPoolThreadAssignmentPolicy,
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
    window::WindowPlugin,
};

use wdn_physics::{
    PhysicsPlugin as WdnPhysicsPlugin,
    kinematics::{GlobalPosition, Position},
    layer::Layer,
    tile::{
        index::TileIndex, material::TileMaterial, position::TilePosition, storage::TileStorageMut,
    },
};
use wdn_render::{RenderPlugin as WdnRenderPlugin, RenderSystems, dev::DevRenderSettings};
use wdn_save::SavePlugin as WdnSavePlugin;
use wdn_tasks::TasksPlugin as WdnTasksPlugin;
use wdn_ui::UiPlugin as WdnUiPlugin;
use wdn_world::pawn::{Pawn, PawnAction};
use wdn_world::{WorldPlugin as WdnWorldPlugin, door::Door};

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(task_pool()).set(window()),
            WdnPhysicsPlugin,
            WdnWorldPlugin,
            WdnTasksPlugin,
            WdnSavePlugin,
            WdnRenderPlugin,
            WdnUiPlugin,
        ))
        .add_systems(Startup, spawn_pawn)
        .add_systems(
            Update,
            (
                handle_pawn_input,
                handle_tile_toggle
                    .before(RenderSystems::RenderDoors)
                    .before(RenderSystems::RenderTiles)
                    .before(RenderSystems::RenderDev),
            ),
        )
        .configure_schedules(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Error,
            report_sets: true,
            ..default()
        })
        .run();
}

fn task_pool() -> TaskPoolPlugin {
    let threads = bevy::tasks::available_parallelism();
    TaskPoolPlugin {
        task_pool_options: TaskPoolOptions {
            io: TaskPoolThreadAssignmentPolicy {
                min_threads: 1,
                max_threads: 1,
                percent: 0.0,
                on_thread_spawn: None,
                on_thread_destroy: None,
            },
            async_compute: TaskPoolThreadAssignmentPolicy {
                min_threads: 1,
                max_threads: usize::MAX,
                percent: 0.25,
                on_thread_spawn: None,
                on_thread_destroy: None,
            },
            compute: TaskPoolThreadAssignmentPolicy {
                min_threads: threads,
                max_threads: usize::MAX,
                percent: 1.0,
                on_thread_spawn: None,
                on_thread_destroy: None,
            },
            ..default()
        },
    }
}

fn window() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Warden Simulator".to_string(),
            canvas: Some("#bevy".to_owned()),
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Target;

fn spawn_pawn(mut commands: Commands, mut storage: TileStorageMut) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.01,
            ..OrthographicProjection::default_2d()
        }),
    ));
    let layer = commands.spawn((Layer::default(),)).id();
    commands.spawn((
        Player,
        Pawn::default(),
        ChildOf(layer),
        Position::new(Vec2::new(0.0, -1.0), Rot2::IDENTITY),
    ));

    commands.spawn((
        Target,
        Pawn::default(),
        ChildOf(layer),
        Position::new(Vec2::new(0.0, 0.0), Rot2::IDENTITY),
    ));

    commands.spawn((Door::default(), TilePosition::new(layer, 2, 2)));
    storage.set_material(TilePosition::new(layer, 1, 2), TileMaterial::Wall);
    storage.set_material(TilePosition::new(layer, 3, 2), TileMaterial::Wall);

    commands.spawn((Door::default(), TilePosition::new(layer, 1, 1)));
    storage.set_material(TilePosition::new(layer, 1, 0), TileMaterial::Wall);
}

fn handle_pawn_input(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    pawn_query: Single<(&mut PawnAction, &GlobalPosition), (With<Pawn>, With<Player>)>,
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
    if mouse.pressed(MouseButton::Left)
        && let Some(cursor_pos) = window.cursor_position()
        && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos)
    {
        let pawn_pos = pawn_transform.position();
        let direction = world_pos - pawn_pos;

        if let Ok(direction) = Dir2::try_from(direction) {
            // Calculate the target angle
            let target_angle = direction.rotation_from_y();

            // Get current rotation (assuming Z-axis rotation)
            let current_angle = pawn_transform.rotation();

            let angle_diff = current_angle.angle_to(target_angle);

            // Threshold for considering the pawn aligned
            const ANGLE_THRESHOLD: f32 = 0.1;
            const LARGE_ANGLE_THRESHOLD: f32 = 1.0;

            if angle_diff.abs() > LARGE_ANGLE_THRESHOLD {
                // Turn in place for large angle differences
                *action = if angle_diff > 0.0 {
                    PawnAction::TurnLeft
                } else {
                    PawnAction::TurnRight
                };
            } else if angle_diff.abs() > ANGLE_THRESHOLD {
                // Steer while moving for small adjustments
                *action = if angle_diff > 0.0 {
                    PawnAction::SteerLeft
                } else {
                    PawnAction::SteerRight
                };
            } else {
                // Walk forward when aligned
                *action = PawnAction::Walk;
            }
            return;
        }
    }

    // Default to standing
    *action = PawnAction::Stand;
}

fn handle_tile_toggle(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    layer: Single<Entity, With<Layer>>,
    index: Res<TileIndex>,
    mut tile_storage: TileStorageMut,
    mut dev_render: ResMut<DevRenderSettings>,
) {
    if (mouse.just_pressed(MouseButton::Right) || mouse.just_pressed(MouseButton::Left))
        && let Some(cursor_pos) = window.cursor_position()
    {
        let (camera, camera_transform) = camera_query.into_inner();
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            // Convert world position to tile position
            let tile_pos = TilePosition::floor(*layer, world_pos);

            // Toggle tile material between Empty and Wall
            let current_material = tile_storage.get_material(tile_pos);
            match current_material {
                TileMaterial::Empty => {
                    if mouse.just_pressed(MouseButton::Left) {
                        tile_storage.set_material(tile_pos, TileMaterial::Wall);
                    }
                }
                TileMaterial::Wall => {
                    if mouse.just_pressed(MouseButton::Left) {
                        commands.spawn((Door::default(), tile_pos));
                    }
                }
                TileMaterial::Door => {
                    let door_id = index.get_tile(tile_pos);

                    if mouse.just_pressed(MouseButton::Left) {
                        if let Some(door_id) = door_id {
                            commands.entity(door_id).despawn();
                        }
                        tile_storage.set_material(tile_pos, TileMaterial::Empty);
                    } else if mouse.just_pressed(MouseButton::Right) {
                        dev_render.draw_door_flow_fields = door_id;
                    }
                }
            };
        }
    }
}
