// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    camera_controller::pan_camera::{PanCamera, PanCameraPlugin},
    dev_tools::fps_overlay::FpsOverlayPlugin,
    prelude::*,
    window::WindowPlugin,
};

use rand::RngExt;
use wdn_physics::{
    PhysicsPlugin as WdnPhysicsPlugin,
    kinematics::Position,
    layer::Layer,
    tile::{material::TileMaterial, position::TilePosition, storage::TileStorageMut},
};
use wdn_render::RenderPlugin as WdnRenderPlugin;
use wdn_save::SavePlugin as WdnSavePlugin;
use wdn_tasks::TasksPlugin as WdnTasksPlugin;
use wdn_ui::UiPlugin as WdnUiPlugin;
use wdn_world::{
    WorldPlugin as WdnWorldPlugin, WorldSystems,
    door::Door,
    path::region::LayerRegion,
    pawn::{Pawn, PawnAction},
};

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(window()),
            FpsOverlayPlugin::default(),
            PanCameraPlugin,
            WdnPhysicsPlugin,
            WdnWorldPlugin,
            WdnTasksPlugin,
            WdnSavePlugin,
            WdnRenderPlugin,
            WdnUiPlugin,
        ))
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            update_storage.before(WorldSystems::UpdatePaths),
        )
        .add_systems(
            FixedUpdate,
            update_pawns.before(WorldSystems::ApplyPawnActions),
        )
        .run();
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

fn startup(mut commands: Commands, mut time: ResMut<Time<Virtual>>, mut storage: TileStorageMut) {
    commands.spawn((
        Camera2d,
        PanCamera {
            pan_speed: 10.0,
            max_zoom: 50.0,
            key_rotate_cw: None,
            key_rotate_ccw: None,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scale: 0.01,
            ..OrthographicProjection::default_2d()
        }),
    ));

    time.set_relative_speed(8.0);

    let layer = commands.spawn(Layer::default()).id();

    let mut random = rand::rng();
    for x in 0..512 {
        for y in 0..512 {
            let tile = TilePosition::new(layer, x, y);
            if (x % 5 == 0 || y % 5 == 0) && x != 0 && y != 0 && x != 511 && y != 511 {
                if random.random_bool(0.05) {
                    commands.spawn((Door::default(), tile));
                } else if random.random_bool(0.1) {
                    storage.set_material(tile, TileMaterial::Empty);
                } else {
                    storage.set_material(tile, TileMaterial::Wall);
                }
            } else {
                storage.set_material(tile, TileMaterial::Empty);
            }
        }
    }

    for _ in 0..3000 {
        let x = random.random_range(1.0f32..=511.0);
        let y = random.random_range(1.0f32..=511.0);

        commands.spawn((
            Pawn::default(),
            ChildOf(layer),
            Position::new(Vec2::new(x, y), Rot2::IDENTITY),
        ));
    }
}

fn update_storage(
    mut commands: Commands,
    layer: Single<Entity, With<Layer>>,
    regions: Query<&LayerRegion>,
    mut storage: TileStorageMut,
) {
    info!("{} regions", regions.iter().count());

    let mut random = rand::rng();
    let x = random.random_range(1..511);
    let y = random.random_range(1..511);
    let tile = TilePosition::new(*layer, x, y);

    if x % 5 == 0 || y % 5 == 0 {
        if let Some(entity) = storage.index().get_tile(tile) {
            commands.entity(entity).despawn();
        }

        if random.random_bool(0.05) {
            commands.spawn((Door::default(), tile));
        } else if random.random_bool(0.05) {
            storage.set_material(tile, TileMaterial::Empty);
        } else {
            storage.set_material(tile, TileMaterial::Wall);
        }
    }
}

fn update_pawns(mut query: Query<&mut PawnAction>) {
    query
        .par_iter_mut()
        .for_each_init(rand::rng, |rng, mut action| {
            if !rng.random_bool(0.01) {
                return;
            }

            *action = match rng.random_range(0..6) {
                0 => PawnAction::Stand,
                1 => PawnAction::Walk,
                2 => PawnAction::TurnLeft,
                3 => PawnAction::TurnRight,
                4 => PawnAction::SteerLeft,
                5 => PawnAction::SteerRight,
                _ => unreachable!(),
            }
        });
}
