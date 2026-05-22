use std::{
    fmt,
    ops::{Shl, Shr},
};

use wdn_physics::tile::storage::{DoorAdjacency, TileMaterial, WallAdjacency};

use crate::tile::WALL_OFFSET;

// pub fn wall_sprite_offset(solid: bool, walls: WallAdjacency) -> u16 {
//     todo!()
// }

//     const SOLID_LOOKUP: [u8; 256] = [
//         5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 9, 9, 9, 9, 10, 10,
//         10, 10, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 11, 11, 11, 11, 12, 12, 12, 12, 13,
//         13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 15, 15, 15, 15, 16, 16, 16, 16,
//         17, 17, 17, 17, 18, 18, 18, 18, 19, 19, 19, 19, 20, 20, 20, 20, 15, 15, 15, 15, 16, 16, 16,
//         16, 15, 15, 15, 15, 16, 16, 16, 16, 21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23, 24, 24,
//         24, 24, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 9, 9, 9, 9,
//         10, 10, 10, 10, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 11, 11, 11, 11, 12, 12, 12,
//         12, 13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 15, 15, 15, 15, 16, 16,
//         16, 16, 17, 17, 17, 17, 18, 18, 18, 18, 19, 19, 19, 19, 20, 20, 20, 20, 15, 15, 15, 15, 16,
//         16, 16, 16, 15, 15, 15, 15, 16, 16, 16, 16, 21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23,
//         24, 24, 24, 24,
//     ];

//     let offset = if solid {
//         SOLID_LOOKUP[adjacency.bits() as usize] as u16
//     } else {
//         EMPTY_LOOKUP[adjacency.bits() as usize] as u16
//     };

//     WALL_OFFSET + offset
// }

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

fn wall_sprite_offset(mut walls: WallAdjacency, doors: DoorAdjacency) -> u16 {
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
    let door_bits = doors.bits().shr(1);

    println!("mask: {:b}", wall_bits | door_bits);

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

#[test]
fn test_tile_sprite_index() {
    use std::collections::{HashMap, hash_map};

    let mut patterns: HashMap<(TileMaterial, WallAdjacency, DoorAdjacency), u16> = HashMap::new();

    for material in [TileMaterial::Empty, TileMaterial::Wall, TileMaterial::Door] {
        for i in 0..=15u8 {
            let doors = DoorAdjacency::from_bits_retain(i);
            for j in 0..=255u8 {
                let walls = WallAdjacency::from_bits_retain(j);

                let mut normal_material = material;
                let mut normal_walls = walls.intersection(
                    WallAdjacency::SOUTH
                        | WallAdjacency::SOUTH_WEST
                        | WallAdjacency::SOUTH_EAST
                        | WallAdjacency::WEST
                        | WallAdjacency::EAST
                        | WallAdjacency::NORTH,
                );
                let mut normal_doors = doors
                    .intersection(DoorAdjacency::SOUTH | DoorAdjacency::WEST | DoorAdjacency::EAST);

                if walls.contains(WallAdjacency::SOUTH) {
                    normal_doors.remove(DoorAdjacency::SOUTH);
                }

                if walls.contains(WallAdjacency::WEST) {
                    normal_doors.remove(DoorAdjacency::WEST);
                }

                if walls.contains(WallAdjacency::EAST) {
                    normal_doors.remove(DoorAdjacency::EAST);
                }

                match material {
                    TileMaterial::Empty => {
                        normal_walls.remove(WallAdjacency::NORTH);

                        if !normal_walls.contains(WallAdjacency::SOUTH) {
                            normal_walls = WallAdjacency::NONE;
                        }

                        normal_walls.remove(WallAdjacency::EAST | WallAdjacency::WEST);
                        normal_doors = DoorAdjacency::NONE;
                    }
                    TileMaterial::Wall => {
                        normal_walls.remove(WallAdjacency::NORTH);

                        if !normal_walls.contains(WallAdjacency::SOUTH) {
                            normal_walls
                                .remove(WallAdjacency::SOUTH_WEST | WallAdjacency::SOUTH_EAST);
                        }
                    }
                    TileMaterial::Door => {
                        normal_walls.remove(WallAdjacency::WEST | WallAdjacency::EAST);

                        if !normal_walls.contains(WallAdjacency::SOUTH) {
                            normal_walls
                                .remove(WallAdjacency::SOUTH_WEST | WallAdjacency::SOUTH_EAST);
                        }

                        if normal_walls.is_empty() {
                            normal_material = TileMaterial::Empty;
                        }

                        normal_doors = DoorAdjacency::NONE;
                    }
                }

                let index = patterns.len() as u16;

                let index = match patterns.entry((normal_material, normal_walls, normal_doors)) {
                    hash_map::Entry::Occupied(entry) => *entry.get(),
                    hash_map::Entry::Vacant(entry) => {
                        // println!(
                        //     "{}: material={normal_material:?} walls={normal_walls:?}, doors={normal_doors:?}",
                        //     index + 1
                        // );
                        entry.insert(index);
                        index
                    }
                };

                let expected_index = match material {
                    TileMaterial::Empty => empty_sprite_offset(walls),
                    TileMaterial::Wall => wall_sprite_offset(walls, doors),
                    TileMaterial::Door => door_sprite_offset(walls),
                };

                assert_eq!(
                    index, expected_index,
                    "unexpected sprite index for material={material:?}, walls={walls:?}, doors={doors:?}"
                );
                // println!(
                //     "(TileMaterial::{material:?}, {}, {}) => {index},",
                // );
            }
        }
    }

    println!("EMPTY");
    for i in 0..=255 {
        let index = patterns[&normal(
            TileMaterial::Empty,
            WallAdjacency::from_bits_retain(i),
            DoorAdjacency::NONE,
        )];

        print!("{}, ", index);
    }

    println!("\n\nWALL");
    for i in 0..=255 {
        let mut walls = WallAdjacency::from_bits_retain(i);
        walls.remove(WallAdjacency::NORTH | WallAdjacency::NORTH_WEST | WallAdjacency::NORTH_EAST);
        let mut doors = DoorAdjacency::from_bits_truncate((i & 0b111).shl(1));

        let index = patterns[&normal(TileMaterial::Wall, walls, doors)];

        print!("{}, ", index);
    }

    println!("\n\nDOOR");
    for i in 0..=255 {
        let index = patterns[&normal(
            TileMaterial::Door,
            WallAdjacency::from_bits_retain(i),
            DoorAdjacency::NONE,
        )];

        print!("{}, ", index);
    }

    assert_eq!(patterns.len(), 68);
}

fn normal(
    material: TileMaterial,
    walls: WallAdjacency,
    doors: DoorAdjacency,
) -> (TileMaterial, WallAdjacency, DoorAdjacency) {
    let mut normal_material = material;
    let mut normal_walls = walls.intersection(
        WallAdjacency::SOUTH
            | WallAdjacency::SOUTH_WEST
            | WallAdjacency::SOUTH_EAST
            | WallAdjacency::WEST
            | WallAdjacency::EAST
            | WallAdjacency::NORTH,
    );
    let mut normal_doors =
        doors.intersection(DoorAdjacency::SOUTH | DoorAdjacency::WEST | DoorAdjacency::EAST);

    if walls.contains(WallAdjacency::SOUTH) {
        normal_doors.remove(DoorAdjacency::SOUTH);
    }

    if walls.contains(WallAdjacency::WEST) {
        normal_doors.remove(DoorAdjacency::WEST);
    }

    if walls.contains(WallAdjacency::EAST) {
        normal_doors.remove(DoorAdjacency::EAST);
    }

    match material {
        TileMaterial::Empty => {
            normal_walls.remove(WallAdjacency::NORTH);

            if !normal_walls.contains(WallAdjacency::SOUTH) {
                normal_walls = WallAdjacency::NONE;
            }

            normal_walls.remove(WallAdjacency::EAST | WallAdjacency::WEST);
            normal_doors = DoorAdjacency::NONE;
        }
        TileMaterial::Wall => {
            normal_walls.remove(WallAdjacency::NORTH);

            if !normal_walls.contains(WallAdjacency::SOUTH) {
                normal_walls.remove(WallAdjacency::SOUTH_WEST | WallAdjacency::SOUTH_EAST);
            }
        }
        TileMaterial::Door => {
            normal_walls.remove(WallAdjacency::WEST | WallAdjacency::EAST);

            if !normal_walls.contains(WallAdjacency::SOUTH) {
                normal_walls.remove(WallAdjacency::SOUTH_WEST | WallAdjacency::SOUTH_EAST);
            }

            if normal_walls.is_empty() {
                normal_material = TileMaterial::Empty;
            }

            normal_doors = DoorAdjacency::NONE;
        }
    }

    (normal_material, normal_walls, normal_doors)
}
