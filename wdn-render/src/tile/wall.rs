use wdn_physics::tile::storage::TileOccupancy;

use crate::tile::WALL_OFFSET;

pub fn wall_sprite_offset(solid: bool, occupancy: TileOccupancy) -> u16 {
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
        SOLID_LOOKUP[occupancy.bits() as usize] as u16
    } else {
        EMPTY_LOOKUP[occupancy.bits() as usize] as u16
    };

    WALL_OFFSET + offset
}

#[test]
fn test_tile_sprite_index() {
    use std::collections::{HashMap, hash_map};

    let mut patterns: HashMap<(bool, TileOccupancy), u16> = HashMap::new();

    for solid in [false, true] {
        for i in 0..=255u8 {
            let occupancy = TileOccupancy::from_bits_retain(i);
            let mut normal = occupancy.intersection(
                TileOccupancy::SOUTH
                    | TileOccupancy::SOUTH_WEST
                    | TileOccupancy::SOUTH_EAST
                    | TileOccupancy::WEST
                    | TileOccupancy::EAST,
            );

            if solid {
                if !normal.contains(TileOccupancy::SOUTH) {
                    normal.remove(TileOccupancy::SOUTH_WEST);
                    normal.remove(TileOccupancy::SOUTH_EAST);
                }
            } else {
                if !normal.contains(TileOccupancy::SOUTH) {
                    normal = TileOccupancy::NONE;
                }

                normal.remove(TileOccupancy::EAST);
                normal.remove(TileOccupancy::WEST);
            }

            let index = patterns.len() as u16;

            match patterns.entry((solid, normal)) {
                hash_map::Entry::Occupied(entry) => {
                    assert_eq!(
                        wall_sprite_offset(solid, occupancy),
                        *entry.get() as u16,
                        "unexpected sprite index for solid={solid}, occupancy={occupancy:?}, normal={normal:?}"
                    );
                }
                hash_map::Entry::Vacant(entry) => {
                    println!(
                        "new pattern: solid={solid}, occupancy={occupancy:?}, normal={normal:?} => index={index}"
                    );
                    assert_eq!(
                        wall_sprite_offset(solid, occupancy),
                        index,
                        "unexpected sprite index for solid={solid}, occupancy={occupancy:?}, normal={normal:?}"
                    );
                    entry.insert(index);
                }
            }
        }
    }

    assert_eq!(patterns.len(), 25);
}
