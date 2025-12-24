#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod pan_camera;

use bevy::{prelude::*, window::{PrimaryWindow, WindowPlugin}};
use rand::Rng;
use wdn_physics::{
    PhysicsPlugin as WdnPhysicsPlugin, PhysicsSystems, integrate::Velocity, pawn::Pawn, tile::{
        TilePosition,
        storage::{TileLayer, TileMaterial, TileStorageMut},
    }
};
use wdn_render::RenderPlugin as WdnRenderPlugin;
use wdn_save::SavePlugin as WdnSavePlugin;
use wdn_tasks::TasksPlugin as WdnTasksPlugin;
use wdn_ui::UiPlugin as WdnUiPlugin;

use crate::pan_camera::{PanCamera, PanCameraPlugin};

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
            WdnTasksPlugin,
            WdnSavePlugin,
            WdnRenderPlugin,
            WdnUiPlugin,
            PanCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, (handle_spawn_pawn, handle_toggle_tile))
        .add_systems(
            FixedUpdate,
            (apply_random_acceleration, change_pawn_acceleration).before(PhysicsSystems::ResolveCollisions),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        PanCamera {
            pan_speed: 20.0,
            key_rotate_ccw: None,
            key_rotate_cw: None,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scale: 0.002,
            ..OrthographicProjection::default_2d()
        }),
    ));

    // Spawn a tile layer for the physics system
    let layer = commands.spawn(TileLayer {}).id();
    commands.insert_resource(GameLayer(layer));
}

/// Resource to store the main tile layer entity
#[derive(Resource)]
struct GameLayer(Entity);

/// Handle left mouse click to spawn a pawn at the cursor position
fn handle_spawn_pawn(
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    layer: Res<GameLayer>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };

        let Ok(window) = window_query.single() else {
            return;
        };

        if let Some(cursor_pos) = window.cursor_position() {
            let world_pos = camera
                .viewport_to_world_2d(camera_transform, cursor_pos)
                .unwrap();
            info!("Spawning pawn at world position: {:?}", world_pos);
            commands.spawn((
                Pawn,
                ChildOf(layer.0),
                Transform::from_translation(world_pos.extend(0.0)),
                RandomAcceleration::default(),
            ));
        }
    }
}

/// Handle right mouse click to toggle tile material between Empty and Wall
fn handle_toggle_tile(
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    layer: Res<GameLayer>,
    mut tile_storage: TileStorageMut,
) {
    if mouse_button.just_pressed(MouseButton::Right) {
        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };

        let Ok(window) = window_query.single() else {
            return;
        };

        if let Some(cursor_pos) = window.cursor_position() {
            let world_pos = camera
                .viewport_to_world_2d(camera_transform, cursor_pos)
                .unwrap();
            let tile_pos = TilePosition::floor(layer.0, world_pos);
            let current_material = tile_storage.get_material(tile_pos);

            let new_material = match current_material {
                TileMaterial::Empty => TileMaterial::Wall,
                TileMaterial::Wall => TileMaterial::Empty,
            };

            info!(
                "Toggling tile at {:?} from {:?} to {:?}",
                tile_pos.position(),
                current_material,
                new_material
            );
            tile_storage.set_material(tile_pos, new_material);
        }
    }
}

/// Component to track random acceleration for a pawn
#[derive(Component)]
struct RandomAcceleration {
    acceleration: Vec2,
    timer: Timer,
}

impl Default for RandomAcceleration {
    fn default() -> Self {
        Self {
            acceleration: random_acceleration(),
            timer: Timer::from_seconds(rand::rng().random_range(1.0..3.0), TimerMode::Repeating),
        }
    }
}

/// Generate a random acceleration vector
fn random_acceleration() -> Vec2 {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let magnitude = rng.random_range(0.5..2.0);
    Vec2::new(angle.cos(), angle.sin()) * magnitude
}

/// System to apply acceleration to pawn velocity
fn apply_random_acceleration(
    mut query: Query<(&mut Velocity, &RandomAcceleration), With<Pawn>>,
    time: Res<Time>,
) {
    let delta_secs = time.delta_secs();

    for (mut velocity, random_accel) in query.iter_mut() {
        let current_vel = velocity.get();
        let new_vel = current_vel + random_accel.acceleration * delta_secs;

        // Limit maximum speed
        let max_speed = 1.0;
        let speed = new_vel.length();
        let limited_vel = if speed > max_speed {
            new_vel.normalize() * max_speed
        } else {
            new_vel
        };

        *velocity = Velocity::new(limited_vel);
    }
}

/// System to periodically change the acceleration direction
fn change_pawn_acceleration(
    mut query: Query<&mut RandomAcceleration, With<Pawn>>,
    time: Res<Time>,
) {
    for mut random_accel in query.iter_mut() {
        random_accel.timer.tick(time.delta());

        if random_accel.timer.just_finished() {
            random_accel.acceleration = random_acceleration();
            random_accel
                .timer
                .set_duration(std::time::Duration::from_secs_f32(
                    rand::rng().random_range(1.0..3.0),
                ));
        }
    }
}
