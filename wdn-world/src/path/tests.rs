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

#[test]
fn test_diagonal_wall_splits_region() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    // First create an enclosed region with a box
    for i in -3i32..=3 {
        set_tile(&mut app, center.with_offset(IVec2::new(i, -3)));
        set_tile(&mut app, center.with_offset(IVec2::new(i, 3)));
        set_tile(&mut app, center.with_offset(IVec2::new(-3, i)));
        set_tile(&mut app, center.with_offset(IVec2::new(3, i)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2); // Inside and outside

    // Now add a diagonal wall from NW to SE inside the box
    for i in -2..=2 {
        set_tile(&mut app, center.with_offset(IVec2::new(i, i)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 3); // Outside, NE region, SW region

    // Check that positions on opposite sides of the diagonal are in different regions
    let ne_region = tile_region(&mut app, center.with_offset(IVec2::new(1, -1))).unwrap();
    let sw_region = tile_region(&mut app, center.with_offset(IVec2::new(-1, 1))).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();

    assert_ne!(ne_region, sw_region);
    assert_ne!(ne_region, outside);
    assert_ne!(sw_region, outside);

    // Tiles on the diagonal itself should be solid (no region)
    for i in -2..=2 {
        assert_eq!(
            tile_region(&mut app, center.with_offset(IVec2::new(i, i))),
            None
        );
    }
}

#[test]
fn test_diagonal_gap_connects_regions() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    // Create a diagonal wall with a gap in the middle
    set_tile(&mut app, center.with_offset(IVec2::new(0, 0)));
    set_tile(&mut app, center.with_offset(IVec2::new(1, 1)));
    // Gap at (2, 2)
    set_tile(&mut app, center.with_offset(IVec2::new(3, 3)));
    set_tile(&mut app, center.with_offset(IVec2::new(4, 4)));

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    // Should only be 1 region since the gap connects both sides
    assert_eq!(regions.len(), 1);

    let ne_region = tile_region(&mut app, center.with_offset(IVec2::new(5, 0))).unwrap();
    let sw_region = tile_region(&mut app, center.with_offset(IVec2::new(0, 5))).unwrap();

    assert_eq!(ne_region, sw_region);
}

#[test]
fn test_l_shaped_corridor() {
    let (mut app, layer) = make_app();
    let corner = TilePosition::new(layer, 16, 16);

    // Create an L-shaped enclosed corridor
    // Horizontal part
    for x in -3..=3 {
        set_tile(&mut app, corner.with_offset(IVec2::new(x, -2)));
        set_tile(&mut app, corner.with_offset(IVec2::new(x, 2)));
    }
    // Vertical part
    for y in -3..=3 {
        set_tile(&mut app, corner.with_offset(IVec2::new(-2, y)));
        set_tile(&mut app, corner.with_offset(IVec2::new(2, y)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let inside = tile_region(&mut app, corner).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();

    assert_ne!(inside, outside);

    // Check connectivity within the L-shape
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
fn test_removing_non_connecting_wall() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    // Create an enclosed region with extra walls inside
    for dx in -3..=3 {
        for dy in -3..=3 {
            let pos = center.with_offset(IVec2::new(dx, dy));
            if dx.abs() == 3 || dy.abs() == 3 {
                set_tile(&mut app, pos);
            }
        }
    }
    // Add an internal wall that doesn't separate regions
    set_tile(&mut app, center.with_offset(IVec2::new(0, 0)));

    update_regions(&mut app);

    let initial_regions = get_regions(&mut app);
    assert_eq!(initial_regions.len(), 2);

    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();

    // Remove the internal wall
    clear_tile(&mut app, center.with_offset(IVec2::new(0, 0)));
    update_regions(&mut app);

    let new_regions = get_regions(&mut app);
    // Should still be 2 regions since the wall didn't separate anything
    assert_eq!(new_regions.len(), 2);

    // The inside region should still exist (though entity may have changed)
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
fn test_cross_shaped_region() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    // Create a fully enclosed cross-shaped region
    // Outer perimeter
    for i in -6i32..=6 {
        set_tile(&mut app, center.with_offset(IVec2::new(i, -6)));
        set_tile(&mut app, center.with_offset(IVec2::new(i, 6)));
        set_tile(&mut app, center.with_offset(IVec2::new(-6, i)));
        set_tile(&mut app, center.with_offset(IVec2::new(6, i)));
    }

    // Fill in areas outside the cross arms
    for dx in -5i32..=5 {
        for dy in -5i32..=5 {
            // Fill corners that are not part of the cross
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

    assert_ne!(inside, outside);

    // Check that all four arms of the cross are connected
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
fn test_narrow_corridor() {
    let (mut app, layer) = make_app();
    let start = TilePosition::new(layer, 0, 0);

    // Create a narrow 1-tile wide corridor with end caps
    for x in -10i32..=10 {
        set_tile(&mut app, start.with_offset(IVec2::new(x, -1)));
        set_tile(&mut app, start.with_offset(IVec2::new(x, 1)));
    }
    // Add end caps to enclose the corridor
    for y in -1i32..=1 {
        set_tile(&mut app, start.with_offset(IVec2::new(-10, y)));
        set_tile(&mut app, start.with_offset(IVec2::new(10, y)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    assert_eq!(regions.len(), 2);

    let corridor = tile_region(&mut app, start).unwrap();
    let outside = tile_region(&mut app, start.with_offset(IVec2::new(0, 5))).unwrap();

    assert_ne!(corridor, outside);

    // Check connectivity along the corridor
    for x in -9i32..=9 {
        assert_eq!(
            tile_region(&mut app, start.with_offset(IVec2::new(x, 0))),
            Some(corridor)
        );
    }
}

#[test]
fn test_donut_shaped_region() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    // Create outer walls
    for dx in -5..=5 {
        for dy in -5..=5 {
            let pos = center.with_offset(IVec2::new(dx, dy));
            if dx.abs() == 5 || dy.abs() == 5 {
                set_tile(&mut app, pos);
            }
        }
    }

    // Create inner walls (the hole in the donut)
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
    assert_eq!(regions.len(), 3);

    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();
    let inside_hole = tile_region(&mut app, center).unwrap();
    let donut = tile_region(&mut app, center.with_offset(IVec2::new(3, 0))).unwrap();

    assert!(regions.contains(&outside));
    assert!(regions.contains(&inside_hole));
    assert!(regions.contains(&donut));

    // All three regions should be different
    assert_ne!(outside, inside_hole);
    assert_ne!(outside, donut);
    assert_ne!(inside_hole, donut);

    // Check that all sides of the donut are connected
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
fn test_checkerboard_pattern() {
    let (mut app, layer) = make_app();
    let origin = TilePosition::new(layer, 0, 0);

    // Create an enclosed area with checkerboard inside
    for i in -1i32..=8 {
        set_tile(&mut app, origin.with_offset(IVec2::new(i, -1)));
        set_tile(&mut app, origin.with_offset(IVec2::new(i, 8)));
        set_tile(&mut app, origin.with_offset(IVec2::new(-1, i)));
        set_tile(&mut app, origin.with_offset(IVec2::new(8, i)));
    }

    // Create a checkerboard pattern inside
    for x in 0..8 {
        for y in 0..8 {
            if (x + y) % 2 == 0 {
                set_tile(&mut app, origin.with_offset(IVec2::new(x, y)));
            }
        }
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    // Should create many small isolated regions due to diagonal blocking
    assert!(regions.len() > 10);

    // Check that diagonal tiles form separate regions
    // (1,2) and (3,4) are empty tiles (odd sums) in different pockets
    let region_1_2 = tile_region(&mut app, origin.with_offset(IVec2::new(1, 2)));
    let region_3_4 = tile_region(&mut app, origin.with_offset(IVec2::new(3, 4)));

    assert!(region_1_2.is_some());
    assert!(region_3_4.is_some());
    // These should be in different regions due to diagonal blocking
    assert_ne!(region_1_2, region_3_4);
}

#[test]
fn test_maze_structure() {
    let (mut app, layer) = make_app();
    let origin = TilePosition::new(layer, 0, 0);

    // Create a simple maze with multiple paths
    // Outer walls
    for i in 0..10 {
        set_tile(&mut app, origin.with_offset(IVec2::new(i, 0)));
        set_tile(&mut app, origin.with_offset(IVec2::new(i, 9)));
        set_tile(&mut app, origin.with_offset(IVec2::new(0, i)));
        set_tile(&mut app, origin.with_offset(IVec2::new(9, i)));
    }

    // Internal walls creating paths
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
    assert_eq!(regions.len(), 2); // Inside maze and outside

    let maze_inside = tile_region(&mut app, origin.with_offset(IVec2::new(1, 1))).unwrap();
    let outside = tile_region(&mut app, TilePosition::new(layer, 20, 20)).unwrap();

    assert_ne!(maze_inside, outside);

    // Check connectivity through the maze passages
    let top_left = tile_region(&mut app, origin.with_offset(IVec2::new(1, 1))).unwrap();
    let bottom_right = tile_region(&mut app, origin.with_offset(IVec2::new(7, 7))).unwrap();

    assert_eq!(top_left, bottom_right); // Should be connected through the passages
}

#[test]
fn test_spiral_pattern() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    // Create a simple spiral
    // Outer square
    for i in -4i32..=4 {
        set_tile(&mut app, center.with_offset(IVec2::new(i, -4)));
        set_tile(&mut app, center.with_offset(IVec2::new(i, 4)));
        set_tile(&mut app, center.with_offset(IVec2::new(-4, i)));
        set_tile(&mut app, center.with_offset(IVec2::new(4, i)));
    }
    // Gap in outer square
    clear_tile(&mut app, center.with_offset(IVec2::new(4, 0)));

    // Inner walls
    for i in -2i32..=3 {
        set_tile(&mut app, center.with_offset(IVec2::new(i, -2)));
    }
    for i in -2i32..=2 {
        set_tile(&mut app, center.with_offset(IVec2::new(-2, i)));
    }

    update_regions(&mut app);

    let regions = get_regions(&mut app);
    // The spiral should connect the outside to the interior through the gap
    // So we should have just outside region
    assert_eq!(regions.len(), 1);

    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();
    // The gap at (4, 0) connects everything
    assert_eq!(
        tile_region(&mut app, center.with_offset(IVec2::new(3, 0))),
        Some(outside)
    );
    assert_eq!(tile_region(&mut app, center), Some(outside));
}

#[test]
fn test_region_update_with_multiple_changes() {
    let (mut app, layer) = make_app();
    let center = TilePosition::new(layer, 16, 16);

    // Create an enclosed box first
    for i in -4i32..=4 {
        set_tile(&mut app, center.with_offset(IVec2::new(i, -4)));
        set_tile(&mut app, center.with_offset(IVec2::new(i, 4)));
        set_tile(&mut app, center.with_offset(IVec2::new(-4, i)));
        set_tile(&mut app, center.with_offset(IVec2::new(4, i)));
    }

    update_regions(&mut app);

    let initial_regions = get_regions(&mut app);
    assert_eq!(initial_regions.len(), 2); // Inside and outside

    // Now create a dividing wall inside the box
    for i in -3i32..=3 {
        set_tile(&mut app, center.with_offset(IVec2::new(i, 0)));
    }

    update_regions(&mut app);

    let split_regions = get_regions(&mut app);
    assert_eq!(split_regions.len(), 3); // Outside, north, and south

    let outside = tile_region(&mut app, TilePosition::new(layer, 30, 30)).unwrap();
    let north = tile_region(&mut app, center.with_offset(IVec2::new(0, 2))).unwrap();
    let south = tile_region(&mut app, center.with_offset(IVec2::new(0, -2))).unwrap();

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

fn update_regions(app: &mut App) {
    app.world_mut().run_schedule(FixedUpdate);
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
