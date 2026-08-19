#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    app::TaskPoolThreadAssignmentPolicy,
    camera_controller::pan_camera::{PanCamera, PanCameraPlugin},
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
    window::WindowPlugin,
};

use wdn_physics::{
    PhysicsPlugin as WdnPhysicsPlugin,
    kinematics::Position,
    layer::{Layer, LayerStack},
    tile::{
        commands::TileCommandsExt, material::TileMaterial, position::TilePosition,
        storage::TileStorageMut,
    },
};
use wdn_render::{
    RenderPlugin as WdnRenderPlugin, RenderSystems,
    dev::{DevPlugin as WdnDevRenderPlugin, DevRenderSettings},
    layer::LayerView,
};
use wdn_save::SavePlugin as WdnSavePlugin;
use wdn_tasks::TasksPlugin as WdnTasksPlugin;
use wdn_ui::UiPlugin as WdnUiPlugin;
use wdn_world::{
    WorldPlugin as WdnWorldPlugin,
    door::Door,
    pawn::{Pawn, action::PawnAction, path::PawnPath},
    stair::Stair,
};

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(task_pool()).set(window()),
            WdnPhysicsPlugin,
            WdnWorldPlugin,
            WdnTasksPlugin,
            WdnSavePlugin,
            WdnRenderPlugin,
            WdnDevRenderPlugin,
            WdnUiPlugin,
            PanCameraPlugin,
        ))
        .add_systems(Startup, spawn_pawn)
        .init_resource::<TilePlacementMode>()
        .add_systems(
            Update,
            (
                handle_pawn_input.before(RenderSystems::RenderDamage),
                (update_tile_placement_mode, handle_tile_placement)
                    .chain()
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
        PanCamera {
            pan_speed: 10.0,
            max_zoom: 50.0,
            key_rotate_cw: None,
            key_rotate_ccw: None,
            ..default()
        },
    ));

    let layer_stack = commands.spawn(LayerStack::default()).id();
    let layer = commands
        .spawn((Layer::default(), ChildOf(layer_stack)))
        .id();
    commands.spawn((
        Player,
        Pawn::default(),
        ChildOf(layer),
        Position::new(Vec2::new(0.5, 0.5), Rot2::IDENTITY),
    ));

    commands.insert_resource(LayerView::new(layer_stack, 0));

    storage.set_material(TilePosition::new(layer, 3, 0), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 3, 1), TileMaterial::DOOR);
    commands.spawn((
        Door::default(),
        TilePosition::new(layer, 3, 1),
        ChildOf(layer),
    ));
    storage.set_material(TilePosition::new(layer, 3, 2), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 3, 3), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 4, 3), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 5, 3), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 6, 3), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 7, 3), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 7, 2), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 7, 1), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 7, 0), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 7, -1), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 6, -1), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 5, -1), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 4, -1), TileMaterial::WALL);
    storage.set_material(TilePosition::new(layer, 3, -1), TileMaterial::WALL);
}

fn handle_pawn_input(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    layer: Single<Entity, With<Layer>>,
    pawn_query: Single<(Entity, &mut PawnAction, &mut PawnPath), (With<Pawn>, With<Player>)>,
    mut dev_render: ResMut<DevRenderSettings>,
) {
    let (entity, mut action, mut path) = pawn_query.into_inner();

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

    // Handle movement towards cursor on right click
    if mouse.pressed(MouseButton::Right)
        && let Some(cursor_pos) = window.cursor_position()
        && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos)
    {
        let tile_pos = TilePosition::floor(*layer, world_pos);
        path.set_target(tile_pos);

        dev_render.draw_pawn_paths = Some(entity);
    }

    // Default to standing
    *action = PawnAction::Stand;
}

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
enum TilePlacementMode {
    #[default]
    Clear,
    Wall,
    Door,
    StairN,
    StairS,
    StairE,
    StairW,
}

fn update_tile_placement_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<TilePlacementMode>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        info!("Tile placement mode: Clear");
        *mode = TilePlacementMode::Clear;
    } else if keys.just_pressed(KeyCode::Digit2) {
        info!("Tile placement mode: Wall");
        *mode = TilePlacementMode::Wall;
    } else if keys.just_pressed(KeyCode::Digit3) {
        info!("Tile placement mode: Door");
        *mode = TilePlacementMode::Door;
    } else if keys.just_pressed(KeyCode::Digit4) {
        info!("Tile placement mode: StairN");
        *mode = TilePlacementMode::StairN;
    } else if keys.just_pressed(KeyCode::Digit5) {
        info!("Tile placement mode: StairS");
        *mode = TilePlacementMode::StairS;
    } else if keys.just_pressed(KeyCode::Digit6) {
        info!("Tile placement mode: StairE");
        *mode = TilePlacementMode::StairE;
    } else if keys.just_pressed(KeyCode::Digit7) {
        info!("Tile placement mode: StairW");
        *mode = TilePlacementMode::StairW;
    }
}

fn handle_tile_placement(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    layer: Single<Entity, With<Layer>>,
    mode: Res<TilePlacementMode>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = camera_query.into_inner();
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };
    let tile_pos = TilePosition::floor(*layer, world_pos);

    commands.despawn_tile(tile_pos, TileMaterial::EMPTY);

    match *mode {
        TilePlacementMode::Clear => {}
        TilePlacementMode::Wall => {
            commands.set_material(tile_pos, TileMaterial::WALL);
        }
        TilePlacementMode::Door => {
            commands.spawn_tile(tile_pos, TileMaterial::DOOR, Door::default());
        }
        TilePlacementMode::StairN => {
            commands.spawn_tile(tile_pos, TileMaterial::STAIR, Stair::North);
        }
        TilePlacementMode::StairS => {
            commands.spawn_tile(tile_pos, TileMaterial::STAIR, Stair::South);
        }
        TilePlacementMode::StairE => {
            commands.spawn_tile(tile_pos, TileMaterial::STAIR, Stair::East);
        }
        TilePlacementMode::StairW => {
            commands.spawn_tile(tile_pos, TileMaterial::STAIR, Stair::West);
        }
    }
}
