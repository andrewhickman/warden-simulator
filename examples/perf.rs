use bevy::{
    camera_controller::pan_camera::{PanCamera, PanCameraPlugin},
    dev_tools::fps_overlay::FpsOverlayPlugin,
    ecs::batching::BatchingStrategy,
    log::{error, info},
    prelude::*,
    window::WindowPlugin,
};

use rand::RngExt;
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};
use wdn_physics::{
    PhysicsPlugin as WdnPhysicsPlugin,
    kinematics::Position,
    layer::Layer,
    tile::{material::TileKind, storage::TileStorageMut},
};
use wdn_render::RenderPlugin as WdnRenderPlugin;
use wdn_save::SavePlugin as WdnSavePlugin;
use wdn_tasks::TasksPlugin as WdnTasksPlugin;
use wdn_ui::UiPlugin as WdnUiPlugin;
use wdn_world::{
    WorldPlugin as WdnWorldPlugin, WorldSystems,
    path::region::RegionTiles,
    pawn::{Pawn, action::PawnAction},
};

pub mod generate;

#[derive(Resource, Default)]
struct RegionImageExported(bool);

struct RegionLegendEntry {
    region_index: u32,
    region_id: u64,
    size: usize,
    doors: usize,
    color: [u8; 3],
}

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
        // .add_systems(
        //     FixedUpdate,
        //     update_storage.before(WorldSystems::UpdateRegions),
        // )
        .add_systems(
            FixedUpdate,
            update_generated_map.before(WorldSystems::UpdateRegions),
        )
        .add_systems(
            FixedUpdate,
            update_pawns.before(WorldSystems::ApplyPawnActions),
        )
        .init_resource::<RegionImageExported>()
        .add_systems(
            FixedUpdate,
            export_regions_image_once.after(WorldSystems::UpdateRegions),
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
    let grid = generate::GeneratedTileGrid::new();
    let changed_tiles =
        generate::apply_grid_to_map(&mut commands, &mut storage, layer, grid.kinds());
    info!("map updated: {} changed tiles", changed_tiles);
    commands.insert_resource(grid);

    let mut random = rand::rng();
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

fn update_generated_map(
    mut commands: Commands,
    mut storage: TileStorageMut,
    layer: Single<Entity, With<Layer>>,
    mut grid: ResMut<generate::GeneratedTileGrid>,
    time: Res<Time<Virtual>>,
) {
    if !grid.tick_and_maybe_regenerate(time.delta()) {
        return;
    }

    let changed_tiles =
        generate::apply_grid_to_map(&mut commands, &mut storage, *layer, grid.kinds());
    info!("map updated: {} changed tiles", changed_tiles);
}

fn update_pawns(mut query: Query<&mut PawnAction>) {
    query
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::new().min_batch_size(16))
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

fn export_regions_image_once(
    mut exported: ResMut<RegionImageExported>,
    regions: Query<(Entity, &RegionTiles)>,
) {
    if exported.0 || regions.is_empty() {
        return;
    }

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    let mut tile_count = 0usize;

    for (_, tiles) in regions.iter() {
        for (_, tile) in tiles.tiles() {
            if tile.is_door() {
                continue;
            }

            let pos = tile.position();
            min_x = min_x.min(pos.x());
            min_y = min_y.min(pos.y());
            max_x = max_x.max(pos.x());
            max_y = max_y.max(pos.y());
            tile_count += 1;
        }
    }

    if tile_count == 0 {
        return;
    }

    let width = (max_x - min_x + 1) as u32;
    let height = (max_y - min_y + 1) as u32;
    let mut image = vec![0u8; width as usize * height as usize * 3];
    let mut legend = Vec::with_capacity(regions.iter().len());

    for (region_index, (region_id, tiles)) in regions.iter().enumerate() {
        let region_color = region_color(region_index as u32);
        let mut region_size = 0usize;
        let mut door_count = 0usize;

        for (_, tile) in tiles.tiles() {
            let position = tile.position();
            let x = (position.x() - min_x) as u32;
            let y = (max_y - position.y()) as u32;
            let pixel = ((y * width + x) * 3) as usize;

            let color = if tile.is_door() {
                door_count += 1;
                [255, 255, 255]
            } else if tile.kind() == TileKind::Wall {
                region_size += 1;
                [0, 0, 0]
            } else {
                region_size += 1;
                region_color
            };
            image[pixel..pixel + 3].copy_from_slice(&color);
        }

        legend.push(RegionLegendEntry {
            region_index: region_index as u32,
            region_id: region_id.to_bits(),
            size: region_size,
            doors: door_count,
            color: region_color,
        });
    }

    let output_path = Path::new("target").join("regions.png");
    if let Err(err) = write_png(&output_path, width, height, &image) {
        error!("failed to write region image at {:?}: {err}", output_path);
        return;
    }

    let legend_path = Path::new("target").join("regions_legend.csv");
    if let Err(err) = write_legend_csv(&legend_path, &legend) {
        error!("failed to write region legend at {:?}: {err}", legend_path);
        return;
    }

    exported.0 = true;
    info!(
        "wrote region map PNG to {:?} and legend to {:?} ({}x{}, {} regions)",
        output_path,
        legend_path,
        width,
        height,
        regions.iter().len()
    );
}

fn write_png(path: &Path, width: u32, height: u32, image: &[u8]) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err("output path has no parent directory".to_string());
    };
    fs::create_dir_all(parent).map_err(|err| err.to_string())?;

    let file = File::create(path).map_err(|err| err.to_string())?;
    let writer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut png_writer = encoder.write_header().map_err(|err| err.to_string())?;
    png_writer
        .write_image_data(image)
        .map_err(|err| err.to_string())?;

    Ok(())
}

fn write_legend_csv(path: &Path, legend: &[RegionLegendEntry]) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err("output path has no parent directory".to_string());
    };
    fs::create_dir_all(parent).map_err(|err| err.to_string())?;

    let file = File::create(path).map_err(|err| err.to_string())?;
    let mut writer = BufWriter::new(file);

    writer
        .write_all(b"region_index,region_id,size,doors,r,g,b,hex\n")
        .map_err(|err| err.to_string())?;

    for entry in legend {
        let [r, g, b] = entry.color;
        writeln!(
            writer,
            "{},{},{},{},{},{},{},#{:02X}{:02X}{:02X}",
            entry.region_index, entry.region_id, entry.size, entry.doors, r, g, b, r, g, b
        )
        .map_err(|err| err.to_string())?;
    }

    writer.flush().map_err(|err| err.to_string())?;
    Ok(())
}

fn region_color(index: u32) -> [u8; 3] {
    let hue = (index as f32 * 137.50777).rem_euclid(360.0);
    hsv_to_rgb(hue, 0.85, 0.95)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let x = c * (1.0 - (((h / 60.0).rem_euclid(2.0)) - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    [
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    ]
}
