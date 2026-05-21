use wdn_physics::tile::storage::{DoorAdjacency, TileMaterial, WallAdjacency};

use crate::tile::WALL_OFFSET;

pub fn wall_sprite_offset(solid: bool, adjacency: WallAdjacency) -> u16 {
    //     const EMPTY_LOOKUP: [u8; 256] = [
    //         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2,
    //         2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4,
    //         4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2,
    //         2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3,
    //         4, 4, 4, 4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1,
    //         1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3,
    //         3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
    //         1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
    //     ];

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

    todo!()
}

#[test]
fn test_tile_sprite_index() {
    use std::collections::{HashMap, hash_map};

    let mut patterns: HashMap<(TileMaterial, WallAdjacency, DoorAdjacency), u16> = HashMap::new();

    for material in [TileMaterial::Empty, TileMaterial::Wall, TileMaterial::Door] {
        for i in 0..15u8 {
            let doors = DoorAdjacency::from_bits_retain(i);
            for j in 0..=255u8 {
                let walls = WallAdjacency::from_bits_retain(j);

                let mut normal_walls = walls.intersection(
                    WallAdjacency::SOUTH
                        | WallAdjacency::SOUTH_WEST
                        | WallAdjacency::SOUTH_EAST
                        | WallAdjacency::WEST
                        | WallAdjacency::EAST,
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
                        if !normal_walls.contains(WallAdjacency::SOUTH) {
                            normal_walls = WallAdjacency::NONE;
                        }

                        normal_walls.remove(WallAdjacency::EAST | WallAdjacency::WEST);
                        normal_doors = DoorAdjacency::NONE;
                    }
                    TileMaterial::Wall => {
                        if !normal_walls.contains(WallAdjacency::SOUTH) {
                            normal_walls
                                .remove(WallAdjacency::SOUTH_WEST | WallAdjacency::SOUTH_EAST);
                        }
                    }
                    TileMaterial::Door => {
                        normal_walls.remove(WallAdjacency::WEST | WallAdjacency::EAST);
                        normal_doors = DoorAdjacency::NONE;
                    }
                }

                let index = patterns.len() as u16;

                match patterns.entry((material, normal_walls, normal_doors)) {
                    hash_map::Entry::Occupied(entry) => {
                        // assert_eq!(
                        //     wall_sprite_offset(solid, walls),
                        //     WALL_OFFSET + *entry.get() as u16,
                        //     "unexpected sprite index for solid={solid}, adjacency={walls:?}, normal={normal_walls:?}"
                        // );
                    }
                    hash_map::Entry::Vacant(entry) => {
                        println!(
                            "{}: material={material:?} walls={normal_walls:?}, doors={normal_doors:?}",
                            index + 1
                        );
                        // assert_eq!(
                        //     wall_sprite_offset(solid, walls),
                        //     WALL_OFFSET + index,
                        //     "unexpected sprite index for solid={solid}, adjacency={walls:?}, normal={normal_walls:?}"
                        // );
                        entry.insert(index);
                    }
                }
            }
        }
    }

    assert_eq!(patterns.len(), 25);
}
