use bevy_app::prelude::*;
use bevy_ecs::entity::EntityHashSet;
use bevy_ecs::{prelude::*, system::RunSystemOnce};
use bevy_math::IVec2;

use wdn_physics::layer::Layer;
use wdn_physics::tile::{
    TilePlugin, TilePosition,
    storage::{TileMap, TileMaterial, TileStorageMut},
};

use super::{
    PathPlugin,
    region::{LayerRegion, TileChunkSections},
};

#[test]
fn test_single_solid_tile_in_chunk() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    clear_tile(&mut app, center);
    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);
    assert_eq!(tile_region(&mut app, center), Some(regions[0]));
    assert_eq!(tile_region(&mut app, center.north()), Some(regions[0]));
    assert_eq!(tile_region(&mut app, center.south()), Some(regions[0]));
    assert_eq!(tile_region(&mut app, center.east()), Some(regions[0]));
    assert_eq!(tile_region(&mut app, center.west()), Some(regions[0]));

    set_tile(&mut app, center);
    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);
    assert_ne!(new_regions[0], regions[0]);
    assert_eq!(tile_region(&mut app, center), None);
    assert_eq!(tile_region(&mut app, center.north()), Some(new_regions[0]));
    assert_eq!(tile_region(&mut app, center.south()), Some(new_regions[0]));
    assert_eq!(tile_region(&mut app, center.east()), Some(new_regions[0]));
    assert_eq!(tile_region(&mut app, center.west()), Some(new_regions[0]));
}

#[test]
fn test_enclosed_region_single_chunk() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_tile(&mut app, center.north());
    set_tile(&mut app, center.south());
    set_tile(&mut app, center.east());
    set_tile(&mut app, center.west());

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let center_region = tile_region(&mut app, center).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 20, 20)).unwrap();

    assert!(regions.contains(&center_region));
    assert!(regions.contains(&outside));
    assert_ne!(center_region, outside);

    assert_eq!(tile_region(&mut app, center.north()), None);
    assert_eq!(tile_region(&mut app, center.south()), None);
    assert_eq!(tile_region(&mut app, center.east()), None);
    assert_eq!(tile_region(&mut app, center.west()), None);
}

#[test]
fn test_enclosed_region_multiple_chunks() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 32, 32); // At chunk boundary

    for dx in -2..=2 {
        for dy in -2..=2 {
            let pos = center.with_offset(IVec2::new(dx, dy));
            if dx.abs() < 2 && dy.abs() < 2 {
                clear_tile(&mut app, pos);
            } else {
                set_tile(&mut app, pos);
            }
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 50, 50)).unwrap();

    assert!(regions.contains(&inside));
    assert!(regions.contains(&outside));
    assert_ne!(inside, outside);

    assert_eq!(tile_region(&mut app, center.north()), Some(inside));
    assert_eq!(tile_region(&mut app, center.south()), Some(inside));
    assert_eq!(tile_region(&mut app, center.east()), Some(inside));
    assert_eq!(tile_region(&mut app, center.west()), Some(inside));
}

#[test]
fn test_split_region_single_chunk() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    for dx in -2..=2 {
        for dy in -2..=2 {
            let pos = center.with_offset(IVec2::new(dx, dy));
            if dx.abs() == 2 || dy.abs() == 2 {
                set_tile(&mut app, pos);
            }
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 0, 0)).unwrap();

    assert!(regions.contains(&inside));
    assert!(regions.contains(&outside));

    set_tile(&mut app, center.north());
    set_tile(&mut app, center.east());
    set_tile(&mut app, center.west());
    set_tile(&mut app, center.south());

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 6);
    assert!(regions.contains(&outside));
    assert!(!regions.contains(&inside));
    assert_eq!(
        tile_region(&mut app, TilePosition::new(layer, 0, 0)).unwrap(),
        outside
    );

    assert_eq!(tile_region(&mut app, center.north()), None);
    assert_eq!(tile_region(&mut app, center.south()), None);
    assert_eq!(tile_region(&mut app, center.east()), None);
    assert_eq!(tile_region(&mut app, center.west()), None);

    let c = tile_region(&mut app, center).unwrap();
    let nw = tile_region(&mut app, center.north().west()).unwrap();
    let ne = tile_region(&mut app, center.north().east()).unwrap();
    let sw = tile_region(&mut app, center.south().west()).unwrap();
    let se = tile_region(&mut app, center.south().east()).unwrap();

    let new_regions = EntityHashSet::from_iter([outside, c, nw, ne, sw, se]);
    assert_eq!(new_regions.len(), 6);
    assert!(new_regions.iter().all(|r| regions.contains(r)));
}

#[test]
fn test_split_region_multiple_chunks() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    for dx in -4..=4 {
        for dy in -4..=4 {
            let pos = center.with_offset(IVec2::new(dx, dy));
            if dx.abs() + dy.abs() == 4 {
                set_tile(&mut app, pos);
            }
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let outside = tile_region(&mut app, TilePosition::new(layer, 3, 3)).unwrap();
    let inside = tile_region(&mut app, center).unwrap();
    assert!(regions.contains(&outside));
    assert!(regions.contains(&inside));

    for dy in -4..4 {
        set_tile(&mut app, center.with_offset(IVec2::new(0, dy)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert!(regions.contains(&outside));

    assert_eq!(regions.len(), 3);
    assert!(regions.contains(&outside));
    assert!(!regions.contains(&inside));

    let left = tile_region(&mut app, center.west()).unwrap();
    let right = tile_region(&mut app, center.east()).unwrap();

    let new_regions = EntityHashSet::from_iter([outside, left, right]);
    assert_eq!(new_regions.len(), 3);
    assert!(new_regions.iter().all(|r| regions.contains(r)));
}

#[test]
fn test_combine_two_regions() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    for dx in -2..=2 {
        for dy in -2..=2 {
            let pos = center.with_offset(IVec2::new(dx, dy));
            if dx.abs() == 2 || dy.abs() == 2 || dy == 0 {
                set_tile(&mut app, pos);
            }
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 3);

    let outside = tile_region(&mut app, TilePosition::new(layer, 0, 0)).unwrap();
    let north = tile_region(&mut app, center.north()).unwrap();
    let south = tile_region(&mut app, center.south()).unwrap();

    assert!(regions.contains(&outside));
    assert!(regions.contains(&north));
    assert!(regions.contains(&south));

    clear_tile(&mut app, center);

    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);

    let combined = tile_region(&mut app, center).unwrap();

    assert!(new_regions.contains(&outside));
    assert!(new_regions.contains(&combined));
    assert!(!new_regions.contains(&north));
    assert!(!new_regions.contains(&south));

    assert_eq!(tile_region(&mut app, center.north()), Some(combined));
    assert_eq!(tile_region(&mut app, center.south()), Some(combined));
    assert_eq!(tile_region(&mut app, center.west()), None);
    assert_eq!(tile_region(&mut app, center.east()), None);
}

#[test]
fn test_combine_two_regions_multiple_chunks() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 3, 0);

    for dx in -5..=5 {
        for dy in -5..=5 {
            let pos = center.with_offset(IVec2::new(dx, dy));
            if dx.abs() == 5 || dy.abs() == 5 || dx == 0 {
                set_tile(&mut app, pos);
            }
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 3);

    let outside = tile_region(&mut app, TilePosition::new(layer, 10, 10)).unwrap();
    let west = tile_region(&mut app, center.west()).unwrap();
    let east = tile_region(&mut app, center.east()).unwrap();

    assert!(regions.contains(&outside));
    assert!(regions.contains(&west));
    assert!(regions.contains(&east));

    let ne = TilePosition::new(layer, 7, 4);
    let se = TilePosition::new(layer, 7, -4);
    let nw = TilePosition::new(layer, -1, 4);
    let sw = TilePosition::new(layer, -1, -4);

    assert_eq!(tile_region(&mut app, ne), Some(east));
    assert_eq!(tile_region(&mut app, se), Some(east));
    assert_eq!(tile_region(&mut app, nw), Some(west));
    assert_eq!(tile_region(&mut app, sw), Some(west));

    clear_tile(&mut app, TilePosition::new(layer, 3, 4));
    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);

    let combined = tile_region(&mut app, TilePosition::new(layer, 3, 4)).unwrap();

    assert!(new_regions.contains(&outside));
    assert!(new_regions.contains(&combined));
    assert_ne!(combined, outside);
    assert!(!new_regions.contains(&west));
    assert!(!new_regions.contains(&east));

    assert_eq!(tile_region(&mut app, ne), Some(combined));
    assert_eq!(tile_region(&mut app, se), Some(combined));
    assert_eq!(tile_region(&mut app, nw), Some(combined));
    assert_eq!(tile_region(&mut app, sw), Some(combined));
}

#[test]
fn test_combine_many_regions_single_tick() {
    let (mut app, layer) = make_app();

    for x in -15..=15 {
        for y in -15..=15 {
            if x % 3 == 0 || y % 3 == 0 {
                set_tile(&mut app, TilePosition::new(layer, x, y));
            }
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 101);
    let outside = tile_region(&mut app, TilePosition::new(layer, 16, 16)).unwrap();
    assert!(regions.contains(&outside));

    for x in -14..=14 {
        for y in -14..=14 {
            clear_tile(&mut app, TilePosition::new(layer, x, y));
        }
    }

    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);
    assert!(new_regions.contains(&outside));

    let inside = tile_region(&mut app, TilePosition::new(layer, 0, 0)).unwrap();

    for x in -14..=14 {
        for y in -14..=14 {
            assert_eq!(
                tile_region(&mut app, TilePosition::new(layer, x, y)),
                Some(inside)
            );
        }
    }
}

#[test]
fn test_region_entity_stability() {
    let (mut app, layer) = make_app();

    for x in -3..=3 {
        for y in -3..=3 {
            if x % 3 == 0 || y % 3 == 0 {
                set_tile(&mut app, TilePosition::new(layer, x, y));
            }
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 5);

    let outside = tile_region(&mut app, TilePosition::new(layer, 16, 16)).unwrap();
    let nw = tile_region(&mut app, TilePosition::new(layer, -1, 1)).unwrap();
    let ne = tile_region(&mut app, TilePosition::new(layer, 1, 1)).unwrap();
    let sw = tile_region(&mut app, TilePosition::new(layer, -1, -1)).unwrap();
    let se = tile_region(&mut app, TilePosition::new(layer, 1, -1)).unwrap();

    let expected_regions = EntityHashSet::from_iter([outside, nw, ne, sw, se]);
    assert_eq!(expected_regions.len(), 5);
    assert!(expected_regions.iter().all(|r| regions.contains(r)));

    set_tile(&mut app, TilePosition::new(layer, 2, 2));
    clear_tile(&mut app, TilePosition::new(layer, 0, -1));

    update_regions(&mut app);

    print_region_tiles(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 4);

    assert!(new_regions.contains(&outside));
    assert!(new_regions.contains(&nw));
    assert!(!new_regions.contains(&ne));
    assert!(!new_regions.contains(&sw));
    assert!(!new_regions.contains(&se));

    assert_eq!(
        tile_region(&mut app, TilePosition::new(layer, -1, 1)).unwrap(),
        nw,
    );
    assert_ne!(
        tile_region(&mut app, TilePosition::new(layer, 1, 1)).unwrap(),
        ne,
    );
    assert_ne!(
        tile_region(&mut app, TilePosition::new(layer, -1, -1)).unwrap(),
        sw,
    );
    assert_ne!(
        tile_region(&mut app, TilePosition::new(layer, 1, -1)).unwrap(),
        se,
    );
    assert_eq!(
        tile_region(&mut app, TilePosition::new(layer, -1, -1)).unwrap(),
        tile_region(&mut app, TilePosition::new(layer, 1, -1)).unwrap(),
    );
}

fn make_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((TaskPoolPlugin::default(), TilePlugin, PathPlugin));
    let layer = app.world_mut().spawn(Layer::default()).id();
    (app, layer)
}

fn set_tile(app: &mut App, position: TilePosition) {
    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(position, TileMaterial::Wall);
        })
        .unwrap();
}

fn clear_tile(app: &mut App, position: TilePosition) {
    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(position, TileMaterial::Empty);
        })
        .unwrap();
}

fn update_regions(app: &mut App) {
    app.world_mut().run_schedule(FixedUpdate);
}

fn get_regions(app: &mut App) -> Vec<Entity> {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<LayerRegion>>();
    query.iter(app.world()).collect()
}

fn print_region_tiles(app: &mut App) {
    let mut region_query = app.world_mut().query::<(Entity, &LayerRegion)>();
    let mut chunk_query = app.world_mut().query::<&TileChunkSections>();

    for (entity, region_data) in region_query.iter(app.world()) {
        println!("region {:?}:", entity);
        for (chunk, sections) in region_data.sections() {
            println!("  chunk {:?}:", chunk);
            let chunk = chunk_query.get(app.world(), chunk).unwrap();

            for section in sections {
                let tiles = chunk.tiles(*section).unwrap();
                println!("    {} tiles", tiles.len());
            }
        }
    }
}

fn tile_region(app: &mut App, position: TilePosition) -> Option<Entity> {
    let chunk_entity = app
        .world()
        .resource::<TileMap>()
        .get(position.chunk_position())?;
    let region = app
        .world()
        .get::<TileChunkSections>(chunk_entity)?
        .region(position.chunk_offset())?;
    Some(region)
}
