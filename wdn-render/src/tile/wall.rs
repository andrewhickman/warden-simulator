use wdn_physics::tile::storage::{DoorAdjacency, WallAdjacency};

use crate::tile::WALL_OFFSET;

pub fn wall_sprite_offset(solid: bool, adjacency: WallAdjacency) -> u16 {
    const EMPTY_LOOKUP: [u8; 256] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2,
        2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4,
        4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2,
        2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3,
        4, 4, 4, 4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1,
        1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3,
        3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
        1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
    ];

    const SOLID_LOOKUP: [u8; 256] = [
        5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 9, 9, 9, 9, 10, 10,
        10, 10, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 11, 11, 11, 11, 12, 12, 12, 12, 13,
        13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 15, 15, 15, 15, 16, 16, 16, 16,
        17, 17, 17, 17, 18, 18, 18, 18, 19, 19, 19, 19, 20, 20, 20, 20, 15, 15, 15, 15, 16, 16, 16,
        16, 15, 15, 15, 15, 16, 16, 16, 16, 21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23, 24, 24,
        24, 24, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 9, 9, 9, 9,
        10, 10, 10, 10, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 11, 11, 11, 11, 12, 12, 12,
        12, 13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 15, 15, 15, 15, 16, 16,
        16, 16, 17, 17, 17, 17, 18, 18, 18, 18, 19, 19, 19, 19, 20, 20, 20, 20, 15, 15, 15, 15, 16,
        16, 16, 16, 15, 15, 15, 15, 16, 16, 16, 16, 21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23,
        24, 24, 24, 24,
    ];

    let offset = if solid {
        SOLID_LOOKUP[adjacency.bits() as usize] as u16
    } else {
        EMPTY_LOOKUP[adjacency.bits() as usize] as u16
    };

    WALL_OFFSET + offset
}

#[test]
fn test_tile_sprite_index() {
    use std::collections::{HashMap, hash_map};

    let mut patterns: HashMap<(bool, WallAdjacency, DoorAdjacency), u16> = HashMap::new();

    for i in 0..16u8 {
        let doors = DoorAdjacency::from_bits_retain(i);

        for solid in [false, true] {
            for j in 0..=255u8 {
                let adjacency = WallAdjacency::from_bits_retain(j);
                let mut normal = adjacency.intersection(
                    WallAdjacency::SOUTH
                        | WallAdjacency::SOUTH_WEST
                        | WallAdjacency::SOUTH_EAST
                        | WallAdjacency::WEST
                        | WallAdjacency::EAST,
                );

                let mut normal_doors = doors
                    .intersection(DoorAdjacency::SOUTH | DoorAdjacency::WEST | DoorAdjacency::EAST);

                if adjacency.contains(WallAdjacency::SOUTH) {
                    normal_doors.remove(DoorAdjacency::SOUTH);
                }

                if adjacency.contains(WallAdjacency::WEST) {
                    normal_doors.remove(DoorAdjacency::WEST);
                }

                if adjacency.contains(WallAdjacency::EAST) {
                    normal_doors.remove(DoorAdjacency::EAST);
                }

                if solid {
                    if !normal.contains(WallAdjacency::SOUTH) {
                        normal.remove(WallAdjacency::SOUTH_WEST);
                        normal.remove(WallAdjacency::SOUTH_EAST);
                    }
                } else {
                    if !normal.contains(WallAdjacency::SOUTH) {
                        normal = WallAdjacency::NONE;
                    }

                    normal.remove(WallAdjacency::EAST);
                    normal.remove(WallAdjacency::WEST);
                    normal_doors = DoorAdjacency::NONE;
                }

                let index = patterns.len() as u16;

                match patterns.entry((solid, normal, normal_doors)) {
                    hash_map::Entry::Occupied(entry) => {
                        // assert_eq!(
                        //     wall_sprite_offset(solid, adjacency),
                        //     WALL_OFFSET + *entry.get() as u16,
                        //     "unexpected sprite index for solid={solid}, adjacency={adjacency:?}, normal={normal:?}"
                        // );
                    }
                    hash_map::Entry::Vacant(entry) => {
                        println!(
                            "{index}: solid={solid}, adjacency={adjacency:?}, normal={normal:?}, doors={doors:?}"
                        );
                        // assert_eq!(
                        //     wall_sprite_offset(solid, adjacency),
                        //     WALL_OFFSET + index,
                        //     "unexpected sprite index for solid={solid}, adjacency={adjacency:?}, normal={normal:?}"
                        // );
                        entry.insert(index);
                    }
                }
            }
        }
    }

    assert_eq!(patterns.len(), 25);
}
