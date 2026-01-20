use bevy_app::prelude::*;
use bevy_ecs::entity::EntityHashSet;
use bevy_ecs::{prelude::*, system::RunSystemOnce};
use bevy_math::IVec2;

use bevy_platform::collections::HashSet;
use wdn_physics::layer::Layer;
use wdn_physics::tile::storage::TileChunk;
use wdn_physics::tile::{
    TileChunkOffset, TilePlugin, TilePosition,
    storage::{TileMap, TileMaterial, TileStorage, TileStorageMut},
};

use super::{
    PathPlugin,
    region::{LayerRegion, TileChunkSections},
};

#[test]
fn clear_and_set_tile_updates_region() {
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
    assert_eq!(new_regions.len(), 1);
    assert_ne!(new_regions[0], regions[0]);
    assert_eq!(tile_region(&mut app, center), None);
    assert_eq!(tile_region(&mut app, center.north()), Some(new_regions[0]));
    assert_eq!(tile_region(&mut app, center.south()), Some(new_regions[0]));
    assert_eq!(tile_region(&mut app, center.east()), Some(new_regions[0]));
    assert_eq!(tile_region(&mut app, center.west()), Some(new_regions[0]));
}

#[test]
fn cross_pattern_isolates_center() {
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
fn square_boundary_creates_regions() {
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

    assert_eq!(tile_region(&mut app, center.north()), Some(inside));
    assert_eq!(tile_region(&mut app, center.south()), Some(inside));
    assert_eq!(tile_region(&mut app, center.east()), Some(inside));
    assert_eq!(tile_region(&mut app, center.west()), Some(inside));
}

#[test]
fn subdivide_square_into_quadrants() {
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
fn diamond_split_vertically() {
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
fn horizontal_split_and_merge() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 2);
    for dx in -2..=2 {
        set_tile(&mut app, center.with_offset(IVec2::new(dx, 0)));
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
fn vertical_split_and_merge() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 3, 0);

    set_square(&mut app, center, 5);
    for dy in -5..=5 {
        set_tile(&mut app, center.with_offset(IVec2::new(0, dy)));
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
fn grid_many_regions_then_merge() {
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
fn grid_modifications_merge_and_split() {
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
fn diagonal_wall_splits_region() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 3);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    for i in -2..=2 {
        set_tile(&mut app, center.with_offset(IVec2::new(i, i)));
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
fn sparse_diagonal_walls_stay_connected() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_tile(&mut app, center.with_offset(IVec2::new(0, 0)));
    set_tile(&mut app, center.with_offset(IVec2::new(1, 1)));
    set_tile(&mut app, center.with_offset(IVec2::new(3, 3)));
    set_tile(&mut app, center.with_offset(IVec2::new(4, 4)));

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 1);

    let ne_region = tile_region(&mut app, center.with_offset(IVec2::new(5, 0))).unwrap();
    let sw_region = tile_region(&mut app, center.with_offset(IVec2::new(0, 5))).unwrap();

    assert!(regions.contains(&ne_region));
    assert_eq!(ne_region, sw_region);
}

#[test]
fn square_with_overlapping_corners() {
    let (mut app, layer) = make_app();
    let corner = TilePosition::new(layer, 16, 16);

    for x in -3..=3 {
        set_tile(&mut app, corner.with_offset(IVec2::new(x, -2)));
        set_tile(&mut app, corner.with_offset(IVec2::new(x, 2)));
    }
    for y in -3..=3 {
        set_tile(&mut app, corner.with_offset(IVec2::new(-2, y)));
        set_tile(&mut app, corner.with_offset(IVec2::new(2, y)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, corner).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();

    assert!(regions.contains(&inside));
    assert!(regions.contains(&outside));
    assert_ne!(inside, outside);

    assert_eq!(
        tile_region(&mut app, corner.with_offset(IVec2::new(0, 0))),
        Some(inside)
    );
    assert_eq!(
        tile_region(&mut app, corner.with_offset(IVec2::new(1, 1))),
        Some(inside)
    );
    assert_eq!(
        tile_region(&mut app, corner.with_offset(IVec2::new(-1, -1))),
        Some(inside)
    );
    assert_eq!(
        tile_region(&mut app, corner.with_offset(IVec2::new(1, -1))),
        Some(inside)
    );
    assert_eq!(
        tile_region(&mut app, corner.with_offset(IVec2::new(-1, 1))),
        Some(inside)
    );
}

#[test]
fn toggle_center_tile() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 3);

    set_tile(&mut app, center.with_offset(IVec2::new(0, 0)));

    update_regions(&mut app);

    let initial_regions = get_regions(&mut app);
    assert_eq!(initial_regions.len(), 2);

    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();

    clear_tile(&mut app, center.with_offset(IVec2::new(0, 0)));
    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    assert_eq!(new_regions.len(), 2);

    let inside_after = tile_region(&mut app, center.with_offset(IVec2::new(0, 0))).unwrap();
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(1, 1))),
        Some(inside_after)
    );
    assert_eq!(
        tile_region(&mut app, TilePosition::new(layer, 30, 30)),
        Some(outside)
    );
    assert_ne!(inside_after, outside);
}

#[test]
fn nested_squares_single_region() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 6);

    for dx in -5i32..=5 {
        for dy in -5i32..=5 {
            if dx.abs() > 2 && dy.abs() > 2 {
                set_tile(&mut app, center.with_offset(IVec2::new(dx, dy)));
            }
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, center).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();

    assert!(regions.contains(&inside));
    assert!(regions.contains(&outside));
    assert_ne!(inside, outside);

    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(0, -4))),
        Some(inside)
    );
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(0, 4))),
        Some(inside)
    );
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(4, 0))),
        Some(inside)
    );
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(-4, 0))),
        Some(inside)
    );
}

#[test]
fn long_corridor_single_region() {
    let (mut app, layer) = make_app();
    let start = TilePosition::new(layer, 0, 0);

    for x in -10i32..=10 {
        set_tile(&mut app, start.with_offset(IVec2::new(x, -1)));
        set_tile(&mut app, start.with_offset(IVec2::new(x, 1)));
    }
    for y in -1i32..=1 {
        set_tile(&mut app, start.with_offset(IVec2::new(-10, y)));
        set_tile(&mut app, start.with_offset(IVec2::new(10, y)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let corridor = tile_region(&mut app, start).unwrap();
    let outside = tile_region(&mut app, start.with_offset(IVec2::new(0, 5))).unwrap();

    assert!(regions.contains(&corridor));
    assert!(regions.contains(&outside));
    assert_ne!(corridor, outside);

    for x in -9i32..=9 {
        assert_eq!(
            tile_region(&mut app, start.with_offset(IVec2::new(x, 0))),
            Some(corridor)
        );
    }
}

#[test]
fn nested_regions() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 5);
    set_square(&mut app, center, 2);

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 3);

    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();
    let inside_hole = tile_region(&mut app, center).unwrap();
    let donut = tile_region(&mut app, center.with_offset(IVec2::new(3, 0))).unwrap();

    assert!(regions.contains(&outside));
    assert!(regions.contains(&inside_hole));
    assert!(regions.contains(&donut));

    assert_ne!(outside, inside_hole);
    assert_ne!(outside, donut);
    assert_ne!(inside_hole, donut);

    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(3, 0))),
        Some(donut)
    );
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(-3, 0))),
        Some(donut)
    );
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(0, 3))),
        Some(donut)
    );
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(0, -3))),
        Some(donut)
    );
}

#[test]
fn checkerboard_many_regions() {
    let (mut app, layer) = make_app();
    let origin = TilePosition::new(layer, 0, 0);

    let center = origin.with_offset(IVec2::new(3, 3));
    for i in -4i32..=5 {
        set_tile(&mut app, center.with_offset(IVec2::new(i, -4)));
        set_tile(&mut app, center.with_offset(IVec2::new(i, 5)));
    }
    for i in -4i32..=5 {
        set_tile(&mut app, center.with_offset(IVec2::new(-4, i)));
        set_tile(&mut app, center.with_offset(IVec2::new(5, i)));
    }

    for x in 0..8 {
        for y in 0..8 {
            if (x + y) % 2 == 0 {
                set_tile(&mut app, origin.with_offset(IVec2::new(x, y)));
            }
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert!(regions.len() > 10);

    let region_1_2 = tile_region(&mut app, origin.with_offset(IVec2::new(1, 2))).unwrap();
    let region_3_4 = tile_region(&mut app, origin.with_offset(IVec2::new(3, 4))).unwrap();

    assert!(regions.contains(&region_1_2));
    assert!(regions.contains(&region_3_4));
    assert_ne!(region_1_2, region_3_4);
}

#[test]
fn maze_stays_connected() {
    let (mut app, layer) = make_app();
    let origin = TilePosition::new(layer, 0, 0);

    for i in 0..10 {
        set_tile(&mut app, origin.with_offset(IVec2::new(i, 0)));
        set_tile(&mut app, origin.with_offset(IVec2::new(i, 9)));
        set_tile(&mut app, origin.with_offset(IVec2::new(0, i)));
        set_tile(&mut app, origin.with_offset(IVec2::new(9, i)));
    }

    for i in 1i32..8 {
        if i != 4 {
            set_tile(&mut app, origin.with_offset(IVec2::new(5, i)));
        }
    }
    for i in 1i32..6 {
        if i != 2 {
            set_tile(&mut app, origin.with_offset(IVec2::new(i, 5)));
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
fn partial_boundary_stays_connected() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 4);
    clear_tile(&mut app, center.with_offset(IVec2::new(4, 0)));

    for i in -2i32..=3 {
        set_tile(&mut app, center.with_offset(IVec2::new(i, -2)));
    }
    for i in -2i32..=2 {
        set_tile(&mut app, center.with_offset(IVec2::new(-2, i)));
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
fn horizontal_wall_splits_region() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    set_square(&mut app, center, 4);

    update_regions(&mut app);

    let initial_regions = get_regions(&mut app);
    assert_eq!(initial_regions.len(), 2);

    for i in -3i32..=3 {
        set_tile(&mut app, center.with_offset(IVec2::new(i, 0)));
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

fn set_square(app: &mut App, center: TilePosition, radius: i32) {
    for i in -radius..=radius {
        set_tile(app, center.with_offset(IVec2::new(i, -radius)));
        set_tile(app, center.with_offset(IVec2::new(i, radius)));
        set_tile(app, center.with_offset(IVec2::new(-radius, i)));
        set_tile(app, center.with_offset(IVec2::new(radius, i)));
    }
}

fn update_regions(app: &mut App) {
    app.world_mut().run_schedule(FixedUpdate);

    app.world_mut().run_system_once(validate_regions).unwrap();
}

fn get_regions(app: &mut App) -> Vec<Entity> {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<LayerRegion>>();
    query.iter(app.world()).collect()
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

fn validate_regions(
    storage: TileStorage,
    regions: Query<(Entity, &LayerRegion)>,
    chunks: Query<(Entity, &TileChunk, &TileChunkSections)>,
) {
    let mut unique_chunk_sections = HashSet::new();
    for (region_id, region) in regions {
        for (chunk_id, sections) in region.sections() {
            let chunk_sections = chunks.get(chunk_id).unwrap().2;
            for &section in sections {
                assert!(unique_chunk_sections.insert((chunk_id, section)));
                assert_eq!(chunk_sections.region(section).unwrap(), region_id);
            }
        }
    }

    let mut unique_tile_positions = HashSet::new();
    for (chunk_id, chunk, chunk_sections) in &chunks {
        for offset in TileChunkOffset::iter() {
            let position = TilePosition::from_chunk_position_and_offset(chunk.position(), offset);
            let tile = chunk.get(offset);
            if tile.is_solid() {
                assert!(chunk_sections.region(offset).is_none());
            } else {
                let region = chunk_sections.region(offset).unwrap();

                for neighbor in [
                    position.east(),
                    position.west(),
                    position.north(),
                    position.south(),
                ] {
                    if let Some(neighbor_chunk_id) = storage.chunk_id(neighbor.chunk_position()) {
                        let (_, neighbor_chunk, neighbor_sections) =
                            chunks.get(neighbor_chunk_id).unwrap();
                        if !neighbor_chunk.get(neighbor.chunk_offset()).is_solid() {
                            let neighbor_region =
                                neighbor_sections.region(neighbor.chunk_offset()).unwrap();
                            assert_eq!(neighbor_region, region);
                        }
                    }
                }
            }
        }

        for section in chunk_sections.sections() {
            let region_id = chunk_sections.region(section).unwrap();
            for &offset in chunk_sections.tiles(section).unwrap() {
                assert!(
                    unique_tile_positions.insert((chunk_id, offset)),
                    "duplicate tile position in chunk {:?} at offset {:?}",
                    chunk_id,
                    offset
                );
                assert_eq!(chunk_sections.region(offset).unwrap(), region_id);
            }

            let (_, region) = regions.get(region_id).unwrap();
            let region_chunk = region.sections().find(|&(c, _)| c == chunk_id).unwrap().1;
            assert!(region_chunk.contains(&section));
        }
    }
}
