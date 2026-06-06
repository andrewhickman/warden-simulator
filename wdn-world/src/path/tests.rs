use approx::assert_relative_eq;
use bevy_app::prelude::*;
use bevy_ecs::entity::EntityHashSet;
use bevy_ecs::{prelude::*, system::RunSystemOnce};
use bevy_math::{Dir2, IVec2};

use bevy_platform::collections::HashSet;
use wdn_physics::layer::Layer;
use wdn_physics::tile::CHUNK_SIZE;
use wdn_physics::tile::adjacency::Adjacency;
use wdn_physics::tile::storage::TileChunk;
use wdn_physics::tile::{
    TilePlugin,
    material::TileMaterial,
    position::{TileChunkOffset, TilePosition},
    storage::{TileMap, TileStorage, TileStorageMut},
};

use crate::door::Door;
use crate::path::flow::{DoorRegions, FlowField, RegionDoors};

use super::{
    PathPlugin,
    region::{Region, TileChunkSections},
};

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

    let door = set_door_tile(&mut app, center.south());

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
    clear_tile(&mut app, TilePosition::new(layer, 5, 5));

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);
    assert!(region_doors(&mut app, regions[0]).is_empty());
    assert!(get_flow_fields(&mut app).is_empty());
}

#[test]
fn flow_door() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 0, 0);

    let door = set_door_tile(&mut app, center);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    let flow_field = region_door_flow_field(&mut app, regions[0], door);
    assert_eq!(flow_field.get(center), None);

    assert_relative_eq!(flow_field[center.north()], Dir2::SOUTH);
    assert_relative_eq!(flow_field[center.south()], Dir2::NORTH);
    assert_relative_eq!(flow_field[center.east()], Dir2::WEST);
    assert_relative_eq!(flow_field[center.west()], Dir2::EAST);

    assert_relative_eq!(flow_field[center.north().east()], Dir2::SOUTH_WEST);
    assert_relative_eq!(flow_field[center.north().west()], Dir2::SOUTH_EAST);
    assert_relative_eq!(flow_field[center.south().east()], Dir2::NORTH_WEST);
    assert_relative_eq!(flow_field[center.south().west()], Dir2::NORTH_EAST);
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
            storage.set_material(position, TileMaterial::Wall);
        })
        .unwrap();
}

fn set_door_tile(app: &mut App, position: TilePosition) -> Entity {
    app.world_mut().spawn((Door::default(), position)).id()
}

fn clear_tile(app: &mut App, position: TilePosition) {
    app.world_mut()
        .run_system_once(move |mut commands: Commands, mut storage: TileStorageMut| {
            if let Some(tile) = storage.index().get_tile(position) {
                commands.entity(tile).despawn();
            }

            storage.set_material(position, TileMaterial::Empty);
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
        .get::<RegionDoors>(region)
        .unwrap()
        .iter()
        .map(|(_, door)| door.door())
        .collect()
}

fn region_size(app: &mut App, region: Entity) -> usize {
    app.world().get::<Region>(region).unwrap().size()
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

fn region_door_flow_field(app: &mut App, region: Entity, door: Entity) -> &FlowField {
    let id = app
        .world()
        .get::<DoorRegions>(door)
        .unwrap()
        .iter()
        .find(|r| r.region() == region)
        .unwrap()
        .flow_field();
    app.world().get::<FlowField>(id).unwrap()
}

fn validate_regions(
    storage: TileStorage,
    regions: Query<(Entity, &Region, &RegionDoors)>,
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
                    .filter(|tile| tile.1.material() == TileMaterial::Empty)
                    .count()
            })
            .sum::<usize>(),
        regions
            .iter()
            .map(|(_, region, _)| region.size())
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
            match chunk.material(offset) {
                TileMaterial::Wall => {
                    assert!(chunk_sections.region(offset).is_none());
                }
                TileMaterial::Door => {
                    assert!(chunk_sections.region(offset).is_none());

                    for (neighbor, adjacency) in [
                        (position.east(), Adjacency::EAST),
                        (position.west(), Adjacency::WEST),
                        (position.north(), Adjacency::NORTH),
                        (position.south(), Adjacency::SOUTH),
                    ] {
                        if let Some(neighbor_chunk_id) = storage.chunk_id(neighbor.chunk_position())
                        {
                            let (_, neighbour_chunk, neighbor_sections) =
                                chunks.get(neighbor_chunk_id).unwrap();
                            if neighbour_chunk.material(neighbor.chunk_offset())
                                == TileMaterial::Empty
                            {
                                assert!(
                                    neighbor_sections
                                        .doors(neighbor.chunk_offset())
                                        .unwrap()
                                        .any(|(door_position, door_adjacency)| door_position
                                            == position
                                            && door_adjacency.contains(adjacency))
                                );
                            }
                        }
                    }
                }
                TileMaterial::Empty => {
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
                            if neighbor_chunk.material(neighbor.chunk_offset())
                                == TileMaterial::Empty
                            {
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
            for &offset in chunk_sections.tiles(section_id).unwrap() {
                assert!(unique_tile_positions.insert((chunk_id, offset)));
                assert_eq!(chunk_sections.region(offset).unwrap(), region_id);
            }

            for (door, _) in chunk_sections.doors(section_id).unwrap() {
                if door.chunk_position() == chunk.position() {
                    assert!(!unique_tile_positions.contains(&(chunk_id, door.chunk_offset())));
                }

                assert_eq!(storage.get_material(door), TileMaterial::Door);
            }

            let (_, region, _) = regions.get(region_id).unwrap();
            assert!(region.sections().any(|(c, s)| c == chunk_id
                && s.chunk_position() == chunk.position()
                && s.chunk_offset() == section_id));
        }
    }

    for (door_id, door_regions, door_position) in &doors {
        for door_region in door_regions.iter() {
            let (_, _, region_doors) = regions.get(door_region.region()).unwrap();
            let region_door = region_doors.get(*door_position).unwrap();
            assert_eq!(region_door.door(), door_id);
            assert_eq!(region_door.flow_field(), door_region.flow_field());
        }
    }
}
