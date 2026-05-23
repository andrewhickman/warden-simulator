use wdn_physics::tile::storage::{DoorAdjacency, TileMaterial, WallAdjacency};

use crate::tile::WALL_OFFSET;

pub fn sprite_offset(material: TileMaterial, walls: WallAdjacency, doors: DoorAdjacency) -> u16 {
    let offset = match material {
        TileMaterial::Empty => empty_sprite_offset(walls),
        TileMaterial::Wall => wall_sprite_offset(walls, doors),
        TileMaterial::Door => door_sprite_offset(walls),
    };

    offset + WALL_OFFSET
}

fn empty_sprite_offset(walls: WallAdjacency) -> u16 {
    const LOOKUP: [u8; 256] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
        4, 4, 4, 4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    ];

    LOOKUP[walls.bits() as usize] as u16
}

fn wall_sprite_offset(walls: WallAdjacency, doors: DoorAdjacency) -> u16 {
    const LOOKUP: [u8; 256] = [
        5, 25, 35, 39, 41, 51, 56, 58, 6, 6, 36, 36, 42, 42, 57, 57, 5, 25, 35, 39, 41, 51, 56, 58,
        6, 6, 36, 36, 42, 42, 57, 57, 7, 26, 7, 26, 43, 52, 43, 52, 8, 8, 8, 8, 44, 44, 44, 44, 9,
        27, 9, 27, 45, 53, 45, 53, 10, 10, 10, 10, 46, 46, 46, 46, 5, 25, 35, 39, 41, 51, 56, 58,
        6, 6, 36, 36, 42, 42, 57, 57, 5, 25, 35, 39, 41, 51, 56, 58, 6, 6, 36, 36, 42, 42, 57, 57,
        11, 28, 11, 28, 47, 54, 47, 54, 12, 12, 12, 12, 48, 48, 48, 48, 13, 29, 13, 29, 49, 55, 49,
        55, 14, 14, 14, 14, 50, 50, 50, 50, 15, 30, 37, 40, 15, 30, 37, 40, 16, 16, 38, 38, 16, 16,
        38, 38, 15, 30, 37, 40, 15, 30, 37, 40, 16, 16, 38, 38, 16, 16, 38, 38, 17, 31, 17, 31, 17,
        31, 17, 31, 18, 18, 18, 18, 18, 18, 18, 18, 19, 32, 19, 32, 19, 32, 19, 32, 20, 20, 20, 20,
        20, 20, 20, 20, 15, 30, 37, 40, 15, 30, 37, 40, 16, 16, 38, 38, 16, 16, 38, 38, 15, 30, 37,
        40, 15, 30, 37, 40, 16, 16, 38, 38, 16, 16, 38, 38, 21, 33, 21, 33, 21, 33, 21, 33, 22, 22,
        22, 22, 22, 22, 22, 22, 23, 34, 23, 34, 23, 34, 23, 34, 24, 24, 24, 24, 24, 24, 24, 24,
    ];

    let wall_bits = walls
        .difference(WallAdjacency::NORTH_WEST | WallAdjacency::NORTH | WallAdjacency::NORTH_EAST)
        .bits();
    let door_bits = doors.bits() >> 1;

    LOOKUP[(wall_bits | door_bits) as usize] as u16
}

fn door_sprite_offset(walls: WallAdjacency) -> u16 {
    const LOOKUP: [u8; 256] = [
        0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0,
        59, 59, 0, 0, 59, 59, 60, 60, 61, 61, 60, 60, 61, 61, 60, 60, 61, 61, 60, 60, 61, 61, 62,
        62, 63, 63, 62, 62, 63, 63, 62, 62, 63, 63, 62, 62, 63, 63, 0, 0, 59, 59, 0, 0, 59, 59, 0,
        0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 64, 64,
        65, 65, 64, 64, 65, 65, 64, 64, 65, 65, 64, 64, 65, 65, 66, 66, 67, 67, 66, 66, 67, 67, 66,
        66, 67, 67, 66, 66, 67, 67, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0,
        59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 60, 60, 61, 61, 60, 60, 61, 61, 60, 60,
        61, 61, 60, 60, 61, 61, 62, 62, 63, 63, 62, 62, 63, 63, 62, 62, 63, 63, 62, 62, 63, 63, 0,
        0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59, 59, 0, 0, 59,
        59, 0, 0, 59, 59, 64, 64, 65, 65, 64, 64, 65, 65, 64, 64, 65, 65, 64, 64, 65, 65, 66, 66,
        67, 67, 66, 66, 67, 67, 66, 66, 67, 67, 66, 66, 67, 67,
    ];

    LOOKUP[walls.bits() as usize] as u16
}

#[test]
fn test_empty_sprite_offset() {
    use std::collections::HashMap;

    let mut patterns: HashMap<WallAdjacency, u16> = HashMap::new();

    for walls in WallAdjacency::values() {
        let mut normal = walls.intersection(
            WallAdjacency::SOUTH | WallAdjacency::SOUTH_WEST | WallAdjacency::SOUTH_EAST,
        );

        if !normal.contains(WallAdjacency::SOUTH) {
            normal = WallAdjacency::NONE;
        }

        let offset = patterns.len() as u16;
        assert_eq!(
            empty_sprite_offset(walls),
            *patterns.entry(normal).or_insert(offset),
            "unexpected sprite index for walls={walls:?}"
        );
    }
}

#[test]
fn test_wall_sprite_offset() {
    use std::collections::HashMap;

    let mut patterns: HashMap<(WallAdjacency, DoorAdjacency), u16> = HashMap::new();

    for doors in DoorAdjacency::values() {
        for walls in WallAdjacency::values() {
            let mut normal_walls = walls.difference(
                WallAdjacency::NORTH_WEST | WallAdjacency::NORTH | WallAdjacency::NORTH_EAST,
            );
            let mut normal_doors = doors.difference(DoorAdjacency::NORTH);

            if walls.contains(WallAdjacency::SOUTH) {
                normal_doors.remove(DoorAdjacency::SOUTH);
            } else {
                normal_walls.remove(WallAdjacency::SOUTH_WEST | WallAdjacency::SOUTH_EAST);
            }

            if walls.contains(WallAdjacency::WEST) {
                normal_doors.remove(DoorAdjacency::WEST);
            }

            if walls.contains(WallAdjacency::EAST) {
                normal_doors.remove(DoorAdjacency::EAST);
            }

            let offset = 5 + patterns.len() as u16;
            assert_eq!(
                wall_sprite_offset(walls, doors),
                *patterns
                    .entry((normal_walls, normal_doors))
                    .or_insert(offset),
                "unexpected sprite index for walls={walls:?}, doors={doors:?}"
            );
        }
    }
}

#[test]
fn test_door_sprite_offset() {
    use std::collections::HashMap;

    let mut patterns: HashMap<WallAdjacency, u16> = HashMap::new();

    for walls in WallAdjacency::values() {
        let mut normal_walls = walls.difference(
            WallAdjacency::NORTH_WEST
                | WallAdjacency::NORTH_EAST
                | WallAdjacency::WEST
                | WallAdjacency::EAST,
        );

        if !normal_walls.contains(WallAdjacency::SOUTH) {
            normal_walls.remove(WallAdjacency::SOUTH_WEST | WallAdjacency::SOUTH_EAST);
        }

        if !walls.contains(WallAdjacency::SOUTH) {
            normal_walls.remove(WallAdjacency::SOUTH_WEST | WallAdjacency::SOUTH_EAST);
        }

        normal_walls.remove(WallAdjacency::WEST | WallAdjacency::EAST);

        if normal_walls.is_empty() {
            assert_eq!(
                door_sprite_offset(walls),
                0,
                "unexpected sprite index for walls={walls:?}"
            );
        } else {
            let offset = 59 + patterns.len() as u16;
            assert_eq!(
                door_sprite_offset(walls),
                *patterns.entry(normal_walls).or_insert(offset),
                "unexpected sprite index for walls={walls:?}"
            );
        }
    }
}
