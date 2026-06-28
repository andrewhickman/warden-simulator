use approx::{AbsDiffEq, RelativeEq, assert_relative_eq};
use bevy_app::prelude::*;
use bevy_ecs::entity::EntityHashSet;
use bevy_ecs::{prelude::*, system::RunSystemOnce};
use bevy_math::{Dir2, IVec2};

use bevy_platform::collections::HashSet;
use wdn_physics::layer::Layer;
use wdn_physics::tile::CHUNK_SIZE;
use wdn_physics::tile::adjacency::Adjacency;
use wdn_physics::tile::index::TileIndex;
use wdn_physics::tile::material::TileMaterial;
use wdn_physics::tile::storage::TileChunk;
use wdn_physics::tile::{
    TilePlugin,
    material::TileKind,
    position::{TileChunkOffset, TilePosition},
    storage::{TileMap, TileStorage, TileStorageMut},
};

use crate::door::Door;
use crate::path::door::DoorRegions;
use crate::path::flow::{FlowField, FlowFieldEntry};
use crate::path::region::RegionTiles;
use crate::path::section::TileChunkSections;

use super::{PathPlugin, region::Region};

#[test]
fn region_empty() {
    let (mut app, layer) = make_app();
    let position = TilePosition::new(layer, 5, 5);
    clear_tile(&mut app, position);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    assert_eq!(region_size(&mut app, regions[0]), 1024);
    assert_eq!(tile_region(&mut app, position), Some(regions[0]));

    assert!(region_doors(&mut app, regions[0]).is_empty());
    assert!(get_flow_fields(&mut app).is_empty());
}

#[test]
fn region_update() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    clear_tile(&mut app, center);
    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);
    assert_eq!(region_size(&mut app, regions[0]), 1024);
    assert_eq!(tile_region(&mut app, center), Some(regions[0]));
    assert_eq!(tile_region(&mut app, center.north()), Some(regions[0]));
    assert_eq!(tile_region(&mut app, center.south()), Some(regions[0]));
    assert_eq!(tile_region(&mut app, center.east()), Some(regions[0]));
    assert_eq!(tile_region(&mut app, center.west()), Some(regions[0]));

    set_wall_tile(&mut app, center);
    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 1);
    assert_ne!(new_regions[0], regions[0]);
    assert_eq!(region_size(&mut app, new_regions[0]), 1023);
    assert_eq!(tile_region(&mut app, center), None);
    assert_eq!(tile_region(&mut app, center.north()), Some(new_regions[0]));
    assert_eq!(tile_region(&mut app, center.south()), Some(new_regions[0]));
    assert_eq!(tile_region(&mut app, center.east()), Some(new_regions[0]));
    assert_eq!(tile_region(&mut app, center.west()), Some(new_regions[0]));
}

#[test]
fn region_update2() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 3);
    set_wall_tile(&mut app, center);

    update_regions(&mut app);

    let initial_regions = get_regions(&mut app);
    assert_eq!(initial_regions.len(), 2);

    let inside = tile_region(&mut app, center.north()).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();

    assert!(initial_regions.contains(&inside));
    assert!(initial_regions.contains(&outside));
    assert_ne!(inside, outside);

    assert_eq!(region_size(&mut app, inside), 24);
    assert_eq!(region_size(&mut app, outside), 975);

    clear_tile(&mut app, center.with_offset(IVec2::new(0, 0)));
    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);

    assert!(new_regions.contains(&outside));
    assert!(!new_regions.contains(&inside));

    let new_inside = tile_region(&mut app, center.with_offset(IVec2::new(0, 0))).unwrap();
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(1, 1))),
        Some(new_inside)
    );
    assert_eq!(
        tile_region(&mut app, TilePosition::new(layer, 30, 30)),
        Some(outside)
    );
    assert_ne!(new_inside, outside);
}

#[test]
fn region_door_update1() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 1);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 5, 5)).unwrap();

    let door = set_door_tile(&mut app, center.south());

    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);

    assert!(!new_regions.contains(&inside));
    assert!(!new_regions.contains(&outside));
    assert_ne!(inside, outside);

    let new_inside = tile_region(&mut app, center).unwrap();
    let new_outside = tile_region(&mut app, TilePosition::new(layer, 5, 5)).unwrap();

    assert!(new_regions.contains(&new_inside));
    assert!(new_regions.contains(&new_outside));
    assert_ne!(inside, outside);

    let inside_doors = region_doors(&mut app, new_inside);
    let outside_doors = region_doors(&mut app, new_outside);

    assert_eq!(inside_doors.len(), 1);
    assert!(inside_doors.contains(&door));
    assert_eq!(outside_doors.len(), 1);
    assert!(outside_doors.contains(&door));

    let door_regions = door_regions(&app, door);
    assert!(door_regions.west().is_none());
    assert!(door_regions.east().is_none());
    let door_region_north = door_regions.north().unwrap();
    assert_eq!(door_region_north.region(), new_inside);
    assert!(door_region_north.dead_end());
    let door_region_south = door_regions.south().unwrap();
    assert_eq!(door_region_south.region(), new_outside);
    assert!(door_region_south.dead_end());
}

#[test]
fn region_door_update2() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 5, 0);

    set_square(&mut app, center, 1);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 5, 5)).unwrap();

    assert!(regions.contains(&inside));
    assert!(regions.contains(&outside));
    assert_ne!(inside, outside);

    let door = set_door_tile(&mut app, center.east());

    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);

    assert!(!new_regions.contains(&inside));
    assert!(!new_regions.contains(&outside));

    let new_inside = tile_region(&mut app, center).unwrap();
    let new_outside = tile_region(&mut app, TilePosition::new(layer, 5, 5)).unwrap();

    assert!(new_regions.contains(&new_inside));
    assert!(new_regions.contains(&new_outside));
    assert_ne!(new_inside, new_outside);

    let inside_doors = region_doors(&mut app, new_inside);
    let outside_doors = region_doors(&mut app, new_outside);

    assert_eq!(inside_doors.len(), 1);
    assert!(inside_doors.contains(&door));
    assert_eq!(outside_doors.len(), 1);
    assert!(outside_doors.contains(&door));

    let door_regions = door_regions(&app, door);
    assert!(door_regions.north().is_none());
    assert!(door_regions.south().is_none());
    let door_region_west = door_regions.west().unwrap();
    assert_eq!(door_region_west.region(), new_inside);
    assert!(door_region_west.dead_end());
    let door_region_east = door_regions.east().unwrap();
    assert_eq!(door_region_east.region(), new_outside);
    assert!(door_region_east.dead_end());
}

#[test]
fn region_room1() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 32, 32);

    set_square(&mut app, center, 2);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 50, 50)).unwrap();

    assert!(regions.contains(&inside));
    assert!(regions.contains(&outside));
    assert_ne!(inside, outside);

    assert_eq!(region_size(&mut app, inside), 9);
    assert_eq!(region_size(&mut app, outside), 4071);

    assert_eq!(tile_region(&mut app, center.north()), Some(inside));
    assert_eq!(tile_region(&mut app, center.south()), Some(inside));
    assert_eq!(tile_region(&mut app, center.east()), Some(inside));
    assert_eq!(tile_region(&mut app, center.west()), Some(inside));
}

#[test]
fn region_room2() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_wall_tile(&mut app, center.north());
    set_wall_tile(&mut app, center.south());
    set_wall_tile(&mut app, center.east());
    set_wall_tile(&mut app, center.west());

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let center_region = tile_region(&mut app, center).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 20, 20)).unwrap();

    assert!(regions.contains(&center_region));
    assert!(regions.contains(&outside));
    assert_ne!(center_region, outside);

    assert_eq!(region_size(&mut app, center_region), 1);
    assert_eq!(region_size(&mut app, outside), 1019);

    assert_eq!(tile_region(&mut app, center.north()), None);
    assert_eq!(tile_region(&mut app, center.south()), None);
    assert_eq!(tile_region(&mut app, center.east()), None);
    assert_eq!(tile_region(&mut app, center.west()), None);
}

#[test]
fn region_room3() {
    let (mut app, layer) = make_app();
    let start = TilePosition::new(layer, 0, 0);

    for x in -10i32..=10 {
        set_wall_tile(&mut app, start.with_offset(IVec2::new(x, -1)));
        set_wall_tile(&mut app, start.with_offset(IVec2::new(x, 1)));
    }
    for y in -1i32..=1 {
        set_wall_tile(&mut app, start.with_offset(IVec2::new(-10, y)));
        set_wall_tile(&mut app, start.with_offset(IVec2::new(10, y)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, start).unwrap();
    let outside = tile_region(&mut app, start.with_offset(IVec2::new(0, 5))).unwrap();

    assert!(regions.contains(&inside));
    assert!(regions.contains(&outside));
    assert_ne!(inside, outside);

    for x in -9i32..=9 {
        assert_eq!(
            tile_region(&mut app, start.with_offset(IVec2::new(x, 0))),
            Some(inside)
        );
    }
}

#[test]
fn region_room4() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 5);
    set_square(&mut app, center, 2);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 3);

    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();
    let middle = tile_region(&mut app, center.with_offset(IVec2::new(3, 0))).unwrap();
    let inside = tile_region(&mut app, center).unwrap();

    assert!(regions.contains(&outside));
    assert!(regions.contains(&inside));
    assert!(regions.contains(&middle));

    assert_ne!(outside, inside);
    assert_ne!(outside, middle);
    assert_ne!(inside, middle);

    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(3, 0))),
        Some(middle)
    );
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(-3, 0))),
        Some(middle)
    );
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(0, 3))),
        Some(middle)
    );
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(0, -3))),
        Some(middle)
    );
}

#[test]
fn region_room5() {
    let (mut app, layer) = make_app();
    let origin = TilePosition::new(layer, 0, 0);

    for i in 0..10 {
        set_wall_tile(&mut app, origin.with_offset(IVec2::new(i, 0)));
        set_wall_tile(&mut app, origin.with_offset(IVec2::new(i, 9)));
        set_wall_tile(&mut app, origin.with_offset(IVec2::new(0, i)));
        set_wall_tile(&mut app, origin.with_offset(IVec2::new(9, i)));
    }

    for i in 1i32..8 {
        if i != 4 {
            set_wall_tile(&mut app, origin.with_offset(IVec2::new(5, i)));
        }
    }
    for i in 1i32..6 {
        if i != 2 {
            set_wall_tile(&mut app, origin.with_offset(IVec2::new(i, 5)));
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let maze_inside = tile_region(&mut app, origin.with_offset(IVec2::new(1, 1))).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 20, 20)).unwrap();

    assert!(regions.contains(&maze_inside));
    assert!(regions.contains(&outside));
    assert_ne!(maze_inside, outside);

    let top_left = tile_region(&mut app, origin.with_offset(IVec2::new(1, 1))).unwrap();
    let bottom_right = tile_region(&mut app, origin.with_offset(IVec2::new(7, 7))).unwrap();

    assert_eq!(top_left, bottom_right);
}

#[test]
fn region_room6() {
    let (mut app, layer) = make_app();

    let center = TilePosition::new(layer, 32, 32);

    set_square(&mut app, center, 5);

    for y in -3..5 {
        set_wall_tile(&mut app, center.with_offset(IVec2::new(1, y)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let left = tile_region(&mut app, center.with_offset(IVec2::new(-1, 0))).unwrap();
    let right = tile_region(&mut app, center.with_offset(IVec2::new(3, 0))).unwrap();

    assert_eq!(left, right);
}

#[test]
fn region_room7() {
    let (mut app, layer) = make_app();

    for x in 0..5 {
        set_wall_tile(&mut app, TilePosition::new(layer, x, 5));
    }
    for y in -2..5 {
        set_wall_tile(&mut app, TilePosition::new(layer, 0, y));
        set_wall_tile(&mut app, TilePosition::new(layer, 5, y));
    }

    update_regions(&mut app);

    let initial_regions = get_regions(&mut app);
    assert_eq!(initial_regions.len(), 1);

    let inside = tile_region(&mut app, TilePosition::new(layer, 2, 3)).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 7, 3)).unwrap();

    assert_eq!(inside, outside);
}

#[test]
fn region_room_many() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    set_square(&mut app, center, 5);

    for x in -4..=4 {
        for y in -4..=4 {
            if (x + y) % 2 == 0 {
                set_wall_tile(&mut app, center.with_offset(IVec2::new(x, y)));
            }
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 41);
}

#[test]
fn region_door() {
    let (mut app, layer) = make_app();

    let edge = CHUNK_SIZE as i32 - 1;
    for i in 0..=edge {
        set_wall_tile(&mut app, TilePosition::new(layer, i, 0));
        set_wall_tile(&mut app, TilePosition::new(layer, edge, i));
        set_wall_tile(&mut app, TilePosition::new(layer, edge - i, edge));
        set_wall_tile(&mut app, TilePosition::new(layer, 0, edge - i));
    }

    let south_door = set_door_tile(&mut app, TilePosition::new(layer, 4, 0));
    let east_door = set_door_tile(&mut app, TilePosition::new(layer, edge, 6));
    let north_door = set_door_tile(&mut app, TilePosition::new(layer, 15, edge));
    let west_door = set_door_tile(&mut app, TilePosition::new(layer, 0, 23));

    update_regions(&mut app);

    let inside = tile_region(&mut app, TilePosition::new(layer, 5, 5)).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, -5, -5)).unwrap();

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);
    assert!(regions.contains(&inside));
    assert!(regions.contains(&outside));
    assert_ne!(inside, outside);

    let inside_doors = region_doors(&mut app, inside);
    let outside_doors = region_doors(&mut app, outside);

    assert_eq!(inside_doors.len(), 4);
    assert!(inside_doors.contains(&south_door));
    assert!(inside_doors.contains(&east_door));
    assert!(inside_doors.contains(&north_door));
    assert!(inside_doors.contains(&west_door));

    assert_eq!(outside_doors.len(), 4);
    assert!(outside_doors.contains(&south_door));
    assert!(outside_doors.contains(&east_door));
    assert!(outside_doors.contains(&north_door));
    assert!(outside_doors.contains(&west_door));

    let north_door_regions = door_regions(&app, north_door);
    assert!(north_door_regions.east().is_none());
    assert!(north_door_regions.west().is_none());
    let north_door_south = north_door_regions.south().unwrap();
    assert_eq!(north_door_south.region(), inside);
    assert!(!north_door_south.dead_end());
    let north_door_north = north_door_regions.north().unwrap();
    assert_eq!(north_door_north.region(), outside);
    assert!(!north_door_north.dead_end());

    let east_door_regions = door_regions(&app, east_door);
    assert!(east_door_regions.north().is_none());
    assert!(east_door_regions.south().is_none());
    let east_door_west = east_door_regions.west().unwrap();
    assert_eq!(east_door_west.region(), inside);
    assert!(!east_door_west.dead_end());
    let east_door_east = east_door_regions.east().unwrap();
    assert_eq!(east_door_east.region(), outside);
    assert!(!east_door_east.dead_end());

    let south_door_regions = door_regions(&app, south_door);
    assert!(south_door_regions.east().is_none());
    assert!(south_door_regions.west().is_none());
    let south_door_north = south_door_regions.north().unwrap();
    assert_eq!(south_door_north.region(), inside);
    assert!(!south_door_north.dead_end());
    let south_door_south = south_door_regions.south().unwrap();
    assert_eq!(south_door_south.region(), outside);
    assert!(!south_door_south.dead_end());

    let west_door_regions = door_regions(&app, west_door);
    assert!(west_door_regions.north().is_none());
    assert!(west_door_regions.south().is_none());
    let west_door_east = west_door_regions.east().unwrap();
    assert_eq!(west_door_east.region(), inside);
    assert!(!west_door_east.dead_end());
    let west_door_west = west_door_regions.west().unwrap();
    assert_eq!(west_door_west.region(), outside);
    assert!(!west_door_west.dead_end());
}

#[test]
fn region_wall1() {
    let (mut app, layer) = make_app();

    for x in 4..8 {
        set_wall_tile(&mut app, TilePosition::new(layer, x, 3));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    let region = tile_region(&mut app, TilePosition::new(layer, 0, 0)).unwrap();
    assert!(regions.contains(&region));
    assert_eq!(region_size(&mut app, region), 1020);
}

#[test]
fn region_wall2() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_wall_tile(&mut app, center.with_offset(IVec2::new(0, 0)));
    set_wall_tile(&mut app, center.with_offset(IVec2::new(1, 1)));
    set_wall_tile(&mut app, center.with_offset(IVec2::new(3, 3)));
    set_wall_tile(&mut app, center.with_offset(IVec2::new(4, 4)));

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    let ne_region = tile_region(&mut app, center.with_offset(IVec2::new(5, 0))).unwrap();
    let sw_region = tile_region(&mut app, center.with_offset(IVec2::new(0, 5))).unwrap();

    assert!(regions.contains(&ne_region));
    assert_eq!(ne_region, sw_region);
}

#[test]
fn region_wall3() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 4);
    clear_tile(&mut app, center.with_offset(IVec2::new(4, 0)));

    for i in -2i32..=3 {
        set_wall_tile(&mut app, center.with_offset(IVec2::new(i, -2)));
    }
    for i in -2i32..=2 {
        set_wall_tile(&mut app, center.with_offset(IVec2::new(-2, i)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();

    assert!(regions.contains(&outside));

    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(3, 0))),
        Some(outside)
    );
    assert_eq!(tile_region(&mut app, center), Some(outside));
}

#[test]
fn region_wall4() {
    let (mut app, layer) = make_app();

    set_wall_tile(&mut app, TilePosition::new(layer, 1, 0));
    set_wall_tile(&mut app, TilePosition::new(layer, 0, 1));

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    let region = regions[0];

    assert_eq!(
        tile_region(&mut app, TilePosition::new(layer, 0, 0)),
        Some(region),
    );
    assert_eq!(
        tile_region(&mut app, TilePosition::new(layer, 1, 1)),
        Some(region),
    );
}

#[test]
fn region_insert() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    for x in -2..=2 {
        for y in -2..=2 {
            set_wall_tile(&mut app, center.with_offset(IVec2::new(x, y)));
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    let outside = tile_region(&mut app, TilePosition::new(layer, 10, 10)).unwrap();

    assert!(regions.contains(&outside));

    assert_eq!(region_size(&mut app, outside), 4071);

    for x in -1..=1 {
        for y in -1..=1 {
            clear_tile(&mut app, center.with_offset(IVec2::new(x, y)));
        }
    }

    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();

    assert!(new_regions.contains(&inside));
    assert!(new_regions.contains(&outside));
    assert_ne!(inside, outside);

    assert_eq!(region_size(&mut app, outside), 4071);
    assert_eq!(region_size(&mut app, inside), 9);
}

#[test]
fn region_remove() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    set_square(&mut app, center, 2);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let outside = tile_region(&mut app, TilePosition::new(layer, 10, 10)).unwrap();
    let inside = tile_region(&mut app, center).unwrap();

    assert!(regions.contains(&outside));
    assert!(regions.contains(&inside));
    assert_ne!(inside, outside);

    assert_eq!(region_size(&mut app, outside), 4071);
    assert_eq!(region_size(&mut app, inside), 9);

    for x in -1..=1 {
        for y in -1..=1 {
            set_wall_tile(&mut app, center.with_offset(IVec2::new(x, y)));
        }
    }

    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 1);

    assert!(new_regions.contains(&outside));

    assert_eq!(region_size(&mut app, outside), 4071);
}

#[test]
fn region_split1() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 2);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 0, 0)).unwrap();

    assert!(regions.contains(&inside));
    assert!(regions.contains(&outside));

    assert_eq!(region_size(&mut app, inside), 9);
    assert_eq!(region_size(&mut app, outside), 999);

    set_wall_tile(&mut app, center.north());
    set_wall_tile(&mut app, center.east());
    set_wall_tile(&mut app, center.west());
    set_wall_tile(&mut app, center.south());

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

    assert_eq!(region_size(&mut app, outside), 999);
    assert_eq!(region_size(&mut app, c), 1);
    assert_eq!(region_size(&mut app, nw), 1);
    assert_eq!(region_size(&mut app, ne), 1);
    assert_eq!(region_size(&mut app, sw), 1);
    assert_eq!(region_size(&mut app, se), 1);
}

#[test]
fn region_split2() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    for dx in -4..=4 {
        for dy in -4..=4 {
            let pos = center.with_offset(IVec2::new(dx, dy));
            if dx.abs() + dy.abs() == 4 {
                set_wall_tile(&mut app, pos);
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

    assert_eq!(region_size(&mut app, outside), 4055);
    assert_eq!(region_size(&mut app, inside), 25);

    for dy in -4..4 {
        set_wall_tile(&mut app, center.with_offset(IVec2::new(0, dy)));
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

    assert_eq!(region_size(&mut app, outside), 4055);
    assert_eq!(region_size(&mut app, left), 9);
    assert_eq!(region_size(&mut app, right), 9);
}

#[test]
fn region_split3() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 3);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    for i in -2..=2 {
        set_wall_tile(&mut app, center.with_offset(IVec2::new(i, i)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 3);

    let ne_region = tile_region(&mut app, center.with_offset(IVec2::new(1, -1))).unwrap();
    let sw_region = tile_region(&mut app, center.with_offset(IVec2::new(-1, 1))).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();

    assert!(regions.contains(&ne_region));
    assert!(regions.contains(&sw_region));
    assert!(regions.contains(&outside));

    assert_ne!(ne_region, sw_region);
    assert_ne!(ne_region, outside);
    assert_ne!(sw_region, outside);

    for i in -2..=2 {
        assert_eq!(
            tile_region(&mut app, center.with_offset(IVec2::new(i, i))),
            None
        );
    }
}

#[test]
fn region_split4() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 4);

    update_regions(&mut app);

    let initial_regions = get_regions(&mut app);
    assert_eq!(initial_regions.len(), 2);

    for i in -3i32..=3 {
        set_wall_tile(&mut app, center.with_offset(IVec2::new(i, 0)));
    }

    update_regions(&mut app);

    let split_regions = get_regions(&mut app);
    assert_eq!(split_regions.len(), 3);

    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();
    let north = tile_region(&mut app, center.with_offset(IVec2::new(0, 2))).unwrap();
    let south = tile_region(&mut app, center.with_offset(IVec2::new(0, -2))).unwrap();

    assert!(split_regions.contains(&outside));
    assert!(split_regions.contains(&north));
    assert!(split_regions.contains(&south));

    assert_ne!(north, south);
    assert_ne!(outside, north);
    assert_ne!(outside, south);
}

#[test]
fn region_door_split() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 1);
    let door = set_door_tile(&mut app, center.north());

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 5, 5)).unwrap();

    assert!(regions.contains(&inside));
    assert!(regions.contains(&outside));
    assert_ne!(inside, outside);

    let inside_doors = region_doors(&mut app, inside);
    let outside_doors = region_doors(&mut app, outside);

    assert_eq!(inside_doors.len(), 1);
    assert!(inside_doors.contains(&door));
    assert_eq!(outside_doors.len(), 1);
    assert!(outside_doors.contains(&door));
}

#[test]
fn region_door_merge() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    set_square(&mut app, center, 2);
    set_wall_tile(&mut app, center.north());
    set_wall_tile(&mut app, center.south());
    set_door_tile(&mut app, center);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 3);
    let outside = tile_region(&mut app, TilePosition::new(layer, 5, 5)).unwrap();
    let west = tile_region(&mut app, center.west()).unwrap();
    let east = tile_region(&mut app, center.east()).unwrap();

    assert!(regions.contains(&outside));
    assert!(regions.contains(&west));
    assert!(regions.contains(&east));
    assert_ne!(outside, west);
    assert_ne!(outside, east);
    assert_ne!(west, east);

    assert_eq!(region_size(&mut app, outside), 4071);
    assert_eq!(region_size(&mut app, west), 3);
    assert_eq!(region_size(&mut app, east), 3);

    clear_tile(&mut app, center);

    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();

    assert!(new_regions.contains(&outside));
    assert!(new_regions.contains(&inside));
    assert_ne!(outside, inside);
    assert_ne!(inside, west);
    assert_ne!(inside, east);

    assert_eq!(region_size(&mut app, outside), 4071);
    assert_eq!(region_size(&mut app, inside), 7);
}

#[test]
fn region_door_insert() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    set_square(&mut app, center, 2);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let outside = tile_region(&mut app, TilePosition::new(layer, 5, 5)).unwrap();
    let inside = tile_region(&mut app, center).unwrap();

    assert!(regions.contains(&outside));
    assert!(regions.contains(&inside));
    assert_ne!(inside, outside);

    assert_eq!(region_size(&mut app, outside), 4071);
    assert_eq!(region_size(&mut app, inside), 9);

    assert!(region_doors(&mut app, inside).is_empty());
    assert!(region_doors(&mut app, outside).is_empty());

    let door = set_door_tile(&mut app, center.with_offset(IVec2::new(0, 2)));

    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);

    let new_inside = tile_region(&mut app, center).unwrap();
    let new_outside = tile_region(&mut app, TilePosition::new(layer, 5, 5)).unwrap();

    assert!(new_regions.contains(&new_inside));
    assert!(new_regions.contains(&new_outside));
    assert!(!new_regions.contains(&inside));
    assert!(!new_regions.contains(&outside));
    assert_ne!(new_inside, new_outside);

    assert_eq!(region_size(&mut app, new_inside), 9);
    assert_eq!(region_size(&mut app, new_outside), 4071);

    assert_eq!(region_doors(&mut app, new_inside), vec![door]);
    assert_eq!(region_doors(&mut app, new_outside), vec![door]);
}

#[test]
fn region_door_update() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    set_square(&mut app, center, 2);
    let door = set_door_tile(&mut app, center.with_offset(IVec2::new(0, 2)));

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let outside = tile_region(&mut app, TilePosition::new(layer, 10, 10)).unwrap();
    let inside = tile_region(&mut app, center).unwrap();

    assert!(regions.contains(&outside));
    assert!(regions.contains(&inside));
    assert_ne!(inside, outside);

    assert_eq!(region_size(&mut app, outside), 4071);
    assert_eq!(region_size(&mut app, inside), 9);

    assert_eq!(region_doors(&mut app, inside), vec![door]);
    assert_eq!(region_doors(&mut app, outside), vec![door]);

    for x in -1..=1 {
        for y in -1..=1 {
            set_wall_tile(&mut app, center.with_offset(IVec2::new(x, y)));
        }
    }

    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 1);

    assert!(new_regions.contains(&outside));

    assert_eq!(region_size(&mut app, outside), 4071);
    assert_eq!(region_doors(&mut app, outside), vec![door]);
}

#[test]
fn region_door_remove() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    set_square(&mut app, center, 2);
    let door = set_door_tile(&mut app, center.with_offset(IVec2::new(0, 2)));

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let outside = tile_region(&mut app, TilePosition::new(layer, 5, 5)).unwrap();
    let inside = tile_region(&mut app, center).unwrap();

    assert!(regions.contains(&outside));
    assert!(regions.contains(&inside));
    assert_ne!(inside, outside);

    assert_eq!(region_size(&mut app, outside), 4071);
    assert_eq!(region_size(&mut app, inside), 9);

    assert_eq!(region_doors(&mut app, inside), vec![door]);
    assert_eq!(region_doors(&mut app, outside), vec![door]);

    clear_tile(&mut app, center.with_offset(IVec2::new(0, 2)));
    set_wall_tile(&mut app, center.with_offset(IVec2::new(0, 2)));

    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);

    let new_inside = tile_region(&mut app, center).unwrap();
    let new_outside = tile_region(&mut app, TilePosition::new(layer, 5, 5)).unwrap();

    assert!(new_regions.contains(&new_inside));
    assert!(new_regions.contains(&new_outside));
    assert!(!new_regions.contains(&inside));
    assert!(!new_regions.contains(&outside));
    assert_ne!(new_inside, new_outside);

    assert_eq!(region_size(&mut app, new_inside), 9);
    assert_eq!(region_size(&mut app, new_outside), 4071);

    assert!(region_doors(&mut app, new_inside).is_empty());
    assert!(region_doors(&mut app, new_outside).is_empty());
}

#[test]
fn region_merge1() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 2);
    for dx in -2..=2 {
        set_wall_tile(&mut app, center.with_offset(IVec2::new(dx, 0)));
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

    assert_eq!(region_size(&mut app, outside), 999);
    assert_eq!(region_size(&mut app, north), 3);
    assert_eq!(region_size(&mut app, south), 3);

    clear_tile(&mut app, center);

    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);

    let combined = tile_region(&mut app, center).unwrap();

    assert!(new_regions.contains(&outside));
    assert!(new_regions.contains(&combined));
    assert!(!new_regions.contains(&north));
    assert!(!new_regions.contains(&south));

    assert_eq!(region_size(&mut app, outside), 999);
    assert_eq!(region_size(&mut app, combined), 7);

    assert_eq!(tile_region(&mut app, center.north()), Some(combined));
    assert_eq!(tile_region(&mut app, center.south()), Some(combined));
    assert_eq!(tile_region(&mut app, center.west()), None);
    assert_eq!(tile_region(&mut app, center.east()), None);
}

#[test]
fn region_merge2() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 3, 0);

    set_square(&mut app, center, 5);
    for dy in -5..=5 {
        set_wall_tile(&mut app, center.with_offset(IVec2::new(0, dy)));
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

    assert_eq!(region_size(&mut app, outside), 3975);
    assert_eq!(region_size(&mut app, west), 36);
    assert_eq!(region_size(&mut app, east), 36);

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

    assert_eq!(region_size(&mut app, outside), 3975);
    assert_eq!(region_size(&mut app, combined), 73);

    assert_eq!(tile_region(&mut app, ne), Some(combined));
    assert_eq!(tile_region(&mut app, se), Some(combined));
    assert_eq!(tile_region(&mut app, nw), Some(combined));
    assert_eq!(tile_region(&mut app, sw), Some(combined));
}

#[test]
fn region_merge_many() {
    let (mut app, layer) = make_app();

    for x in -15..=15 {
        for y in -15..=15 {
            if x % 3 == 0 || y % 3 == 0 {
                set_wall_tile(&mut app, TilePosition::new(layer, x, y));
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
fn region_split_merge() {
    let (mut app, layer) = make_app();

    for x in -3..=3 {
        for y in -3..=3 {
            if x % 3 == 0 || y % 3 == 0 {
                set_wall_tile(&mut app, TilePosition::new(layer, x, y));
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

    set_wall_tile(&mut app, TilePosition::new(layer, 2, 2));
    clear_tile(&mut app, TilePosition::new(layer, 0, -1));

    update_regions(&mut app);

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

    let combined = tile_region(&mut app, TilePosition::new(layer, -1, -1)).unwrap();
    assert!(new_regions.contains(&combined));
    assert_ne!(combined, outside);
}

#[test]
fn flow_empty() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    let door = set_door_tile(&mut app, center);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    let flow_field = region_door_flow_field(&app, regions[0], door);
    assert_eq!(flow_field.len(), 4095);

    assert!(!contains_flow(&app, regions[0], door, center));

    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.north()),
        flow_entry(0.0, -1.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.south()),
        flow_entry(0.0, 1.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.east()),
        flow_entry(-1.0, 0.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.west()),
        flow_entry(1.0, 0.0, 5),
        epsilon = 0.01
    );

    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.north().east()),
        flow_entry(-0.70710677, -0.70710677, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.north().west()),
        flow_entry(0.70710677, -0.70710677, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.south().east()),
        flow_entry(-0.70710677, 0.70710677, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.south().west()),
        flow_entry(0.70710677, 0.70710677, 10),
        epsilon = 0.01
    );

    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(0, 2))),
        flow_entry(0.0, -1.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(1, 2))),
        flow_entry(-0.70710677, -0.70710677, 12),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(2, 2))),
        flow_entry(-0.70710677, -0.70710677, 17),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(2, 1))),
        flow_entry(-0.70710677, -0.70710677, 12),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(2, 0))),
        flow_entry(-1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(2, -1))
        ),
        flow_entry(-0.70710677, 0.70710677, 12),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(2, -2))
        ),
        flow_entry(-0.70710677, 0.70710677, 17),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(1, -2))
        ),
        flow_entry(-0.70710677, 0.70710677, 12),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(0, -2))
        ),
        flow_entry(0.0, 1.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, -2))
        ),
        flow_entry(0.70710677, 0.70710677, 12),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, -2))
        ),
        flow_entry(0.70710677, 0.70710677, 17),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, -1))
        ),
        flow_entry(0.70710677, 0.70710677, 12),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, 0))
        ),
        flow_entry(1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, 1))
        ),
        flow_entry(0.70710677, -0.70710677, 12),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, 2))
        ),
        flow_entry(0.70710677, -0.70710677, 17),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, 2))
        ),
        flow_entry(0.70710677, -0.70710677, 12),
        epsilon = 0.01
    );
}

#[test]
fn flow_wall1() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    for x in -2..=2 {
        set_wall_tile(&mut app, center.with_offset(IVec2::new(x, 0)));
    }

    let door = set_door_tile(&mut app, center);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    let flow_field = region_door_flow_field(&app, regions[0], door);
    assert_eq!(flow_field.len(), 4091);

    let region_tiles = app.world().get::<RegionTiles>(regions[0]).unwrap();
    assert!(
        region_tiles
            .get_tile_index(center.layer_offset())
            .and_then(|index| flow_field.get(index))
            .is_none()
    );
    assert!(
        region_tiles
            .get_tile_index(center.layer_offset().east())
            .and_then(|index| flow_field.get(index))
            .is_none()
    );
    assert!(
        region_tiles
            .get_tile_index(center.layer_offset().west())
            .and_then(|index| flow_field.get(index))
            .is_none()
    );

    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(0, 1))),
        flow_entry(0.0, -1.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(1, 1))),
        flow_entry(-1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(2, 1))),
        flow_entry(-1.0, 0.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(3, 1))),
        flow_entry(-1.0, 0.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(3, 0))),
        flow_entry(0.0, 1.0, 25),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(3, -1))
        ),
        flow_entry(-1.0, 0.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(2, -1))
        ),
        flow_entry(-1.0, 0.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(1, -1))
        ),
        flow_entry(-1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(0, -1))
        ),
        flow_entry(0.0, 1.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, -1))
        ),
        flow_entry(1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, -1))
        ),
        flow_entry(1.0, 0.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-3, -1))
        ),
        flow_entry(1.0, 0.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-3, 0))
        ),
        flow_entry(0.0, 1.0, 25),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-3, 1))
        ),
        flow_entry(1.0, 0.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, 1))
        ),
        flow_entry(1.0, 0.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, 1))
        ),
        flow_entry(1.0, 0.0, 10),
        epsilon = 0.01
    );
}

#[test]
fn flow_wall2() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    for y in -2..=2 {
        set_wall_tile(&mut app, center.with_offset(IVec2::new(0, y)));
    }

    let door = set_door_tile(&mut app, center);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    let flow_field = region_door_flow_field(&app, regions[0], door);
    assert_eq!(flow_field.len(), 4091);

    let region_tiles = app.world().get::<RegionTiles>(regions[0]).unwrap();
    assert!(
        region_tiles
            .get_tile_index(center.layer_offset())
            .and_then(|index| flow_field.get(index))
            .is_none()
    );
    assert!(
        region_tiles
            .get_tile_index(center.layer_offset().north())
            .and_then(|index| flow_field.get(index))
            .is_none()
    );
    assert!(
        region_tiles
            .get_tile_index(center.layer_offset().south())
            .and_then(|index| flow_field.get(index))
            .is_none()
    );

    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(1, 0))),
        flow_entry(-1.0, 0.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(1, 1))),
        flow_entry(0.0, -1.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(1, 2))),
        flow_entry(0.0, -1.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(1, 3))),
        flow_entry(0.0, -1.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(0, 3))),
        flow_entry(1.0, 0.0, 25),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, 3))
        ),
        flow_entry(0.0, -1.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, 2))
        ),
        flow_entry(0.0, -1.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, 1))
        ),
        flow_entry(0.0, -1.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, 0))
        ),
        flow_entry(1.0, 0.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, -1))
        ),
        flow_entry(0.0, 1.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, -2))
        ),
        flow_entry(0.0, 1.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, -3))
        ),
        flow_entry(0.0, 1.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(0, -3))
        ),
        flow_entry(1.0, 0.0, 25),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(1, -3))
        ),
        flow_entry(0.0, 1.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(1, -2))
        ),
        flow_entry(0.0, 1.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(1, -1))
        ),
        flow_entry(0.0, 1.0, 10),
        epsilon = 0.01
    );
}

#[test]
fn flow_wall3() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    set_wall_tile(&mut app, center);
    let door = set_door_tile(&mut app, center.north());
    set_wall_tile(&mut app, center.north().north());
    set_wall_tile(&mut app, center.east());
    set_wall_tile(&mut app, center.east().east());
    set_wall_tile(&mut app, center.west());
    set_wall_tile(&mut app, center.west().west());
    set_wall_tile(&mut app, center.south());
    set_wall_tile(&mut app, center.south().south());

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    let flow_field = region_door_flow_field(&mut app, regions[0], door);
    assert_eq!(flow_field.len(), 4087);

    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-3, -3))
        ),
        flow_entry(0.0, 1.0, 35),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, -3))
        ),
        flow_entry(-0.9284767, 0.37139067, 37),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, -3))
        ),
        flow_entry(-0.70710677, 0.70710677, 39),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(0, -3))
        ),
        flow_entry(1.0, 0.0, 44),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(1, -3))
        ),
        flow_entry(0.70710677, 0.70710677, 39),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(2, -3))
        ),
        flow_entry(0.37139067, 0.9284767, 37),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(3, -3))
        ),
        flow_entry(0.0, 1.0, 35),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-3, -2))
        ),
        flow_entry(0.0, 1.0, 30),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, -2))
        ),
        flow_entry(-0.70710677, 0.70710677, 32),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, -2))
        ),
        flow_entry(-0.37139067, 0.9284767, 37),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(1, -2))
        ),
        flow_entry(0.9284767, 0.37139067, 37),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(2, -2))
        ),
        flow_entry(0.70710677, 0.70710677, 32),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(3, -2))
        ),
        flow_entry(0.0, 1.0, 30),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-3, -1))
        ),
        flow_entry(0.0, 1.0, 25),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, -1))
        ),
        flow_entry(-1.0, 0.0, 30),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, -1))
        ),
        flow_entry(-1.0, 0.0, 35),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(1, -1))
        ),
        flow_entry(1.0, 0.0, 35),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(2, -1))
        ),
        flow_entry(1.0, 0.0, 30),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(3, -1))
        ),
        flow_entry(0.0, 1.0, 25),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-3, 0))
        ),
        flow_entry(0.0, 1.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(3, 0))),
        flow_entry(0.0, 1.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-3, 1))
        ),
        flow_entry(1.0, 0.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, 1))
        ),
        flow_entry(1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, 1))
        ),
        flow_entry(1.0, 0.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(1, 1))),
        flow_entry(-1.0, 0.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(2, 1))),
        flow_entry(-1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(3, 1))),
        flow_entry(-1.0, 0.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-3, 2))
        ),
        flow_entry(0.37139067, -0.9284767, 17),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, 2))
        ),
        flow_entry(0.70710677, -0.70710677, 12),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, 2))
        ),
        flow_entry(0.0, -1.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(1, 2))),
        flow_entry(0.0, -1.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(2, 2))),
        flow_entry(-0.70710677, -0.70710677, 12),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(3, 2))),
        flow_entry(-0.9284767, -0.37139067, 17),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-3, 3))
        ),
        flow_entry(0.70710677, -0.70710677, 19),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-2, 3))
        ),
        flow_entry(0.9284767, -0.37139067, 17),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            regions[0],
            door,
            center.with_offset(IVec2::new(-1, 3))
        ),
        flow_entry(0.0, -1.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(0, 3))),
        flow_entry(1.0, 0.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(1, 3))),
        flow_entry(0.0, -1.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(2, 3))),
        flow_entry(-0.37139067, -0.9284767, 17),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, regions[0], door, center.with_offset(IVec2::new(3, 3))),
        flow_entry(-0.70710677, -0.70710677, 19),
        epsilon = 0.01
    );
}

#[test]
fn flow_room1() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    set_square(&mut app, center, 1);
    let north_door = set_door_tile(&mut app, center.north());
    let south_door = set_door_tile(&mut app, center.south());

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();
    let outside = tile_region(&mut app, center.with_offset(IVec2::new(0, 2))).unwrap();

    assert_eq!(get_flow_fields(&mut app).len(), 4);

    let north_inside = region_door_flow_field(&app, inside, north_door);
    let south_inside = region_door_flow_field(&app, inside, south_door);
    let north_outside = region_door_flow_field(&app, outside, north_door);
    let south_outside = region_door_flow_field(&app, outside, south_door);
    assert_eq!(north_inside.len(), 2);
    assert_eq!(south_inside.len(), 2);
    assert_eq!(north_outside.len(), 4088);
    assert_eq!(south_outside.len(), 4088);

    assert_relative_eq!(
        get_flow(&app, inside, north_door, center),
        flow_entry(0.0, 1.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, inside, north_door, center.south()),
        flow_entry(0.0, 1.0, 10),
        epsilon = 0.01
    );

    assert_relative_eq!(
        get_flow(&app, inside, south_door, center),
        flow_entry(0.0, -1.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(&app, inside, south_door, center.north()),
        flow_entry(0.0, -1.0, 10),
        epsilon = 0.01
    );

    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(-2, -2))
        ),
        flow_entry(0.0, 1.0, 35),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(-1, -2))
        ),
        flow_entry(-1.0, 0.0, 40),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(0, -2))
        ),
        flow_entry(1.0, 0.0, 45),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(1, -2))
        ),
        flow_entry(1.0, 0.0, 40),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(2, -2))
        ),
        flow_entry(0.0, 1.0, 35),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(-2, -1))
        ),
        flow_entry(0.0, 1.0, 30),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(0, -1))
        ),
        flow_entry(0.0, -1.0, 50),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(2, -1))
        ),
        flow_entry(0.0, 1.0, 30),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(-2, 0))
        ),
        flow_entry(0.0, 1.0, 25),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(2, 0))
        ),
        flow_entry(0.0, 1.0, 25),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(-2, 1))
        ),
        flow_entry(0.0, 1.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(2, 1))
        ),
        flow_entry(0.0, 1.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(-2, 2))
        ),
        flow_entry(1.0, 0.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(-1, 2))
        ),
        flow_entry(1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(0, 2))
        ),
        flow_entry(0.0, -1.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(1, 2))
        ),
        flow_entry(-1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            north_door,
            center.with_offset(IVec2::new(2, 2))
        ),
        flow_entry(-1.0, 0.0, 15),
        epsilon = 0.01
    );

    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(-2, -2))
        ),
        flow_entry(1.0, 0.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(-1, -2))
        ),
        flow_entry(1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(0, -2))
        ),
        flow_entry(0.0, 1.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(1, -2))
        ),
        flow_entry(-1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(2, -2))
        ),
        flow_entry(-1.0, 0.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(-2, -1))
        ),
        flow_entry(0.0, -1.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(2, -1))
        ),
        flow_entry(0.0, -1.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(-2, 0))
        ),
        flow_entry(0.0, -1.0, 25),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(2, 0))
        ),
        flow_entry(0.0, -1.0, 25),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(-2, 1))
        ),
        flow_entry(0.0, -1.0, 30),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(0, 1))
        ),
        flow_entry(0.0, 1.0, 50),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(2, 1))
        ),
        flow_entry(0.0, -1.0, 30),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(-2, 2))
        ),
        flow_entry(0.0, -1.0, 35),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(-1, 2))
        ),
        flow_entry(-1.0, 0.0, 40),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(0, 2))
        ),
        flow_entry(1.0, 0.0, 45),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(1, 2))
        ),
        flow_entry(1.0, 0.0, 40),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            outside,
            south_door,
            center.with_offset(IVec2::new(2, 2))
        ),
        flow_entry(0.0, -1.0, 35),
        epsilon = 0.01
    );
}

#[test]
fn flow_room2() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    set_square(&mut app, center, 2);
    set_wall_tile(&mut app, center);
    let outside_door = set_door_tile(&mut app, center.with_offset(IVec2::new(1, 2)));
    let inside_door = set_door_tile(&mut app, center.north());

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    assert_eq!(get_flow_fields(&mut app).len(), 3);

    let inside = tile_region(&mut app, center.east()).unwrap();

    let outside_flow = region_door_flow_field(&app, inside, outside_door);
    let inside_flow = region_door_flow_field(&app, inside, inside_door);

    assert_eq!(outside_flow.len(), 8);
    assert_eq!(inside_flow.len(), 8);

    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            outside_door,
            center.with_offset(IVec2::new(-1, -1))
        ),
        flow_entry(1.0, 0.0, 25),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            outside_door,
            center.with_offset(IVec2::new(0, -1))
        ),
        flow_entry(1.0, 0.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            outside_door,
            center.with_offset(IVec2::new(1, -1))
        ),
        flow_entry(0.0, 1.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            outside_door,
            center.with_offset(IVec2::new(-1, 0))
        ),
        flow_entry(0.0, -1.0, 30),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            outside_door,
            center.with_offset(IVec2::new(1, 1))
        ),
        flow_entry(0.0, 1.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            outside_door,
            center.with_offset(IVec2::new(-1, 1))
        ),
        flow_entry(1.0, 0.0, 35),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            outside_door,
            center.with_offset(IVec2::new(0, 1))
        ),
        flow_entry(1.0, 0.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            outside_door,
            center.with_offset(IVec2::new(1, 1))
        ),
        flow_entry(0.0, 1.0, 5),
        epsilon = 0.01
    );

    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            inside_door,
            center.with_offset(IVec2::new(-1, -1))
        ),
        flow_entry(0.0, 1.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            inside_door,
            center.with_offset(IVec2::new(0, -1))
        ),
        flow_entry(1.0, 0.0, 20),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            inside_door,
            center.with_offset(IVec2::new(1, -1))
        ),
        flow_entry(0.0, 1.0, 15),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            inside_door,
            center.with_offset(IVec2::new(-1, 0))
        ),
        flow_entry(0.0, 1.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            inside_door,
            center.with_offset(IVec2::new(1, 0))
        ),
        flow_entry(0.0, 1.0, 10),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            inside_door,
            center.with_offset(IVec2::new(-1, 1))
        ),
        flow_entry(1.0, 0.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            inside_door,
            center.with_offset(IVec2::new(1, 1))
        ),
        flow_entry(-1.0, 0.0, 5),
        epsilon = 0.01
    );
    assert_relative_eq!(
        get_flow(
            &app,
            inside,
            inside_door,
            center.with_offset(IVec2::new(1, 2))
        ),
        flow_entry(0.0, -1.0, 10),
        epsilon = 0.01
    );
}

#[test]
fn flow_update() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    set_square(&mut app, center, 2);
    let door = set_door_tile(&mut app, center.with_offset(IVec2::new(0, 2)));

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let flow_fields = get_flow_fields(&mut app);
    assert_eq!(flow_fields.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();

    let flow_field_id = region_door_flow_field_id(&app, inside, door);
    let flow_field = app.world().get::<FlowField>(flow_field_id).unwrap();
    assert_eq!(flow_field.len(), 9);

    assert_relative_eq!(
        get_flow(&app, inside, door, center.south()),
        flow_entry(0.0, 1.0, 15),
        epsilon = 0.01
    );

    set_wall_tile(&mut app, center);
    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);
    assert!(!new_regions.contains(&inside));

    let new_flow_fields = get_flow_fields(&mut app);
    assert_eq!(new_flow_fields.len(), 2);
    assert!(!new_flow_fields.contains(&flow_field_id));

    let new_inside = tile_region(&mut app, center.south()).unwrap();
    assert_ne!(new_inside, inside);

    let new_flow_field_id = region_door_flow_field_id(&app, new_inside, door);
    assert_ne!(new_flow_field_id, flow_field_id);

    let new_flow_field = app.world().get::<FlowField>(new_flow_field_id).unwrap();
    assert_eq!(new_flow_field.len(), 8);

    assert_relative_eq!(
        get_flow(&app, new_inside, door, center.south()),
        flow_entry(1.0, 0.0, 25),
        epsilon = 0.01
    );
}

fn make_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((TaskPoolPlugin::default(), TilePlugin, PathPlugin));
    let layer = app.world_mut().spawn(Layer::default()).id();
    (app, layer)
}

fn set_wall_tile(app: &mut App, position: TilePosition) {
    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(position, TileMaterial::WALL);
        })
        .unwrap();
}

fn set_door_tile(app: &mut App, position: TilePosition) -> Entity {
    app.world_mut().spawn((Door::default(), position)).id()
}

fn clear_tile(app: &mut App, position: TilePosition) {
    app.world_mut()
        .run_system_once(move |mut commands: Commands, mut storage: TileStorageMut| {
            if let Some(tile) = storage.index.get_tile(position) {
                commands.entity(tile).despawn();
            }

            storage.set_material(position, TileMaterial::EMPTY);
        })
        .unwrap();
}

fn set_square(app: &mut App, center: TilePosition, radius: i32) {
    for i in -radius..=radius {
        set_wall_tile(app, center.with_offset(IVec2::new(i, -radius)));
        set_wall_tile(app, center.with_offset(IVec2::new(i, radius)));
        set_wall_tile(app, center.with_offset(IVec2::new(-radius, i)));
        set_wall_tile(app, center.with_offset(IVec2::new(radius, i)));
    }
}

fn update_regions(app: &mut App) {
    app.world_mut().run_schedule(FixedUpdate);

    app.world_mut().run_system_once(validate_regions).unwrap();
}

fn get_regions(app: &mut App) -> Vec<Entity> {
    let mut query = app.world_mut().query_filtered::<Entity, With<Region>>();
    query.iter(app.world()).collect()
}

fn get_flow_fields(app: &mut App) -> Vec<Entity> {
    let mut query = app.world_mut().query_filtered::<Entity, With<FlowField>>();
    query.iter(app.world()).collect()
}

fn region_doors(app: &mut App, region: Entity) -> Vec<Entity> {
    app.world()
        .get::<RegionTiles>(region)
        .unwrap()
        .doors()
        .iter()
        .map(|door| door.door())
        .collect()
}

fn region_size(app: &mut App, region: Entity) -> usize {
    let region_tiles = app.world().get::<RegionTiles>(region).unwrap();
    region_tiles.size() - region_tiles.door_count()
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

fn region_door_flow_field_id(app: &App, region: Entity, door: Entity) -> Entity {
    app.world()
        .get::<DoorRegions>(door)
        .unwrap()
        .iter()
        .find(|r| r.region() == region)
        .unwrap()
        .flow_field()
}

fn region_door_flow_field(app: &App, region: Entity, door: Entity) -> &FlowField {
    let id = region_door_flow_field_id(app, region, door);
    app.world().get::<FlowField>(id).unwrap()
}

fn door_regions(app: &App, door: Entity) -> &DoorRegions {
    app.world().get::<DoorRegions>(door).unwrap()
}

fn flow_entry(dir_x: f32, dir_y: f32, cost: u32) -> FlowFieldEntry {
    FlowFieldEntry::new(Dir2::from_xy(dir_x, dir_y).unwrap(), cost)
}

fn contains_flow(app: &App, region: Entity, door: Entity, position: TilePosition) -> bool {
    let flow_field = region_door_flow_field(app, region, door);
    let region_tiles = app.world().get::<RegionTiles>(region).unwrap();
    flow_field
        .get(
            region_tiles
                .get_tile_index(position.layer_offset())
                .unwrap(),
        )
        .is_some()
}

fn get_flow(app: &App, region: Entity, door: Entity, position: TilePosition) -> FlowFieldEntry {
    let flow_field = region_door_flow_field(app, region, door);
    let region_tiles = app.world().get::<RegionTiles>(region).unwrap();
    flow_field
        .get(
            region_tiles
                .get_tile_index(position.layer_offset())
                .unwrap(),
        )
        .unwrap()
}

fn validate_regions(
    storage: TileStorage,
    index: Res<TileIndex>,
    regions: Query<(Entity, &Region, &RegionTiles)>,
    chunks: Query<(Entity, &TileChunk, &TileChunkSections)>,
    doors: Query<(Entity, &DoorRegions, &TilePosition)>,
) {
    assert_eq!(
        storage
            .chunks()
            .iter()
            .map(|chunk| {
                chunk
                    .tiles()
                    .filter(|tile| tile.1.kind() == TileKind::Empty)
                    .count()
            })
            .sum::<usize>(),
        regions
            .iter()
            .map(|(_, _, tiles)| tiles.size() - tiles.door_count())
            .sum::<usize>()
    );

    let mut unique_chunk_sections = HashSet::new();
    for (region_id, region, _) in regions {
        for (chunk_id, section) in region.sections() {
            let chunk_sections = chunks.get(chunk_id).unwrap().2;
            assert!(unique_chunk_sections.insert(section));
            assert_eq!(
                chunk_sections.region(section.chunk_offset()).unwrap(),
                region_id
            );
        }
    }

    let mut unique_tile_positions = HashSet::new();
    for (chunk_id, chunk, chunk_sections) in &chunks {
        for offset in TileChunkOffset::iter() {
            let position = TilePosition::from((chunk.position(), offset));
            match chunk.kind(offset) {
                TileKind::Wall => {
                    assert!(chunk_sections.region(offset).is_none());
                }
                TileKind::Door => {
                    assert!(chunk_sections.region(offset).is_none());

                    let door_id = index.get_tile(position).unwrap();
                    let door_regions = doors.get(door_id).unwrap().1;

                    for (neighbor, adjacency) in [
                        (position.east(), Adjacency::EAST),
                        (position.west(), Adjacency::WEST),
                        (position.north(), Adjacency::NORTH),
                        (position.south(), Adjacency::SOUTH),
                    ] {
                        if let Some(neighbor_chunk_id) = storage.chunk_id(neighbor.chunk_position())
                        {
                            let (_, neighbor_chunk, neighbor_sections) =
                                chunks.get(neighbor_chunk_id).unwrap();
                            if neighbor_chunk.kind(neighbor.chunk_offset()) == TileKind::Empty {
                                let region =
                                    neighbor_sections.region(neighbor.chunk_offset()).unwrap();
                                let region_tiles = regions.get(region).unwrap().2;
                                assert!(
                                    region_tiles
                                        .doors()
                                        .iter()
                                        .any(|region_door| region_door.position()
                                            == position.layer_offset()
                                            && region_door.door() == door_id)
                                );

                                assert!(door_regions.iter().any(|door_region| {
                                    door_region.region() == region
                                        && door_region.adjacency().contains(adjacency)
                                }));
                            }
                        }
                    }
                }
                TileKind::Empty => {
                    let region = chunk_sections.region(offset).unwrap();

                    for neighbor in [
                        position.east(),
                        position.west(),
                        position.north(),
                        position.south(),
                    ] {
                        if let Some(neighbor_chunk_id) = storage.chunk_id(neighbor.chunk_position())
                        {
                            let (_, neighbor_chunk, neighbor_sections) =
                                chunks.get(neighbor_chunk_id).unwrap();
                            if neighbor_chunk.kind(neighbor.chunk_offset()) == TileKind::Empty {
                                let neighbor_region =
                                    neighbor_sections.region(neighbor.chunk_offset()).unwrap();
                                assert_eq!(neighbor_region, region);
                            }
                        }
                    }
                }
            }
        }

        for section_id in chunk_sections.sections() {
            let region_id = chunk_sections.region(section_id).unwrap();
            let region_tiles = regions.get(region_id).unwrap().2;

            for &offset in chunk_sections.tiles(section_id).unwrap() {
                assert!(unique_tile_positions.insert((chunk_id, offset)));
                assert_eq!(chunk_sections.region(offset).unwrap(), region_id);
            }

            for door in region_tiles.doors() {
                let door_position = TilePosition::from((chunk.layer(), door.position()));
                if door_position.chunk_position() == chunk.position() {
                    assert!(
                        !unique_tile_positions
                            .contains(&(chunk_id, door.position().chunk_offset()))
                    );
                }

                assert_eq!(storage.get_kind(door_position), TileKind::Door);
            }

            let (_, region, _) = regions.get(region_id).unwrap();
            assert!(region.sections().any(|(c, s)| c == chunk_id
                && s.chunk_position() == chunk.position()
                && s.chunk_offset() == section_id));
        }
    }

    for (door_id, door_regions, door_position) in &doors {
        for door_region in door_regions.iter() {
            let (_, _, region_tiles) = regions.get(door_region.region()).unwrap();
            let region_door = region_tiles
                .doors()
                .iter()
                .find(|region_door| region_door.position() == door_position.layer_offset())
                .unwrap();
            assert_eq!(region_door.door(), door_id);
            assert_eq!(region_door.flow_field(), door_region.flow_field());
        }
    }
}

impl AbsDiffEq for FlowFieldEntry {
    type Epsilon = <f32 as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.dir().abs_diff_eq(&other.dir(), epsilon) && self.cost() == other.cost()
    }
}

impl RelativeEq for FlowFieldEntry {
    fn default_max_relative() -> Self::Epsilon {
        f32::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.dir().relative_eq(&other.dir(), epsilon, max_relative) && self.cost() == other.cost()
    }
}
