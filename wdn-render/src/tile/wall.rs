use std::collections::HashMap;

use wdn_physics::tile::{
    adjacency::{Adjacency, TileAdjacency},
    material::TileKind,
};

use crate::tile::WALL_OFFSET;

enum WallSpriteOffset {
    Empty,
    EmptyWallTopCorner,
    EmptyWallTopHorizontal,
    EmptyWallTopCornerStairTopVerticalUpper,
    EmptyWallTopHorizontalStairTopVerticalUpper,
    EmptyWallTopCornerStairTopHorizontalUpper,
    EmptyWallTopHorizontalStairTopHorizontalUpper,
    EmptyWallTopCornerStairTopVerticalLower,
    EmptyWallTopHorizontalStairTopVerticalLower,
    WallCorner,
    WallVertical,
    WallCornerDoorSouth,
    WallVerticalStairTopVerticalUpper,
    WallVerticalStairTopHorizontalUpper,
    WallVerticalStairTopVerticalLower,
    WallHorizontal,
    WallInverseCorner,
    WallFull,
    WallHorizontalDoorSouth,
    WallInverseCornerStairTopVerticalUpper,
    WallFullStairTopVerticalUpper,
    WallInverseCornerStairTopHorizontalUpper,
    WallFullStairTopHorizontalUpper,
    WallInverseCornerStairTopVerticalLower,
    WallFullStairTopVerticalLower,
    WallCornerDoorEast,
    WallVerticalDoorEast,
    WallCornerDoorEastSouth,
    WallVerticalStairTopVerticalUpperDoorEast,
    WallVerticalStairTopHorizontalUpperDoorEast,
    WallVerticalStairTopVerticalLowerDoorEast,
    DoorWallTopCorner,
    DoorWallTopHorizontal,
    DoorWallTopCornerStairTopVerticalUpper,
    DoorWallTopHorizontalStairTopVerticalUpper,
    DoorWallTopCornerStairTopHorizontalUpper,
    DoorWallTopHorizontalStairTopHorizontalUpper,
    DoorWallTopCornerStairTopVerticalLower,
    DoorWallTopHorizontalStairTopVerticalLower,
    WallCornerStairVerticalLower,
    WallVerticalStairVerticalLower,
    WallCornerStairVerticalLowerDoorSouth,
    WallVerticalStairVerticalLowerStairTopVerticalUpper,
    WallVerticalStairVerticalLowerStairTopHorizontalUpper,
    WallVerticalStairVerticalLowerStairTopVerticalLower,
    WallHorizontalStairVerticalLower,
    WallInverseCornerStairVerticalLower,
    WallFullStairVerticalLower,
    WallHorizontalStairVerticalLowerDoorSouth,
    WallInverseCornerStairVerticalLowerStairTopVerticalUpper,
    WallFullStairVerticalLowerStairTopVerticalUpper,
    WallInverseCornerStairVerticalLowerStairTopHorizontalUpper,
    WallFullStairVerticalLowerStairTopHorizontalUpper,
    WallInverseCornerStairVerticalLowerStairTopVerticalLower,
    WallFullStairVerticalLowerStairTopVerticalLower,
    WallCornerStairVerticalLowerDoorEast,
    WallVerticalStairVerticalLowerDoorEast,
    WallCornerStairVerticalLowerDoorEastSouth,
    WallVerticalStairVerticalLowerStairTopVerticalUpperDoorEast,
    WallVerticalStairVerticalLowerStairTopHorizontalUpperDoorEast,
    WallVerticalStairVerticalLowerStairTopVerticalLowerDoorEast,
    WallCornerStairHorizontalLower,
    WallVerticalStairHorizontalLower,
    WallCornerStairHorizontalLowerDoorSouth,
    WallVerticalStairHorizontalLowerStairTopVerticalUpper,
    WallVerticalStairHorizontalLowerStairTopHorizontalUpper,
    WallVerticalStairHorizontalLowerStairTopVerticalLower,
    WallHorizontalStairHorizontalLower,
    WallInverseCornerStairHorizontalLower,
    WallFullStairHorizontalLower,
    WallHorizontalStairHorizontalLowerDoorSouth,
    WallInverseCornerStairHorizontalLowerStairTopVerticalUpper,
    WallFullStairHorizontalLowerStairTopVerticalUpper,
    WallInverseCornerStairHorizontalLowerStairTopHorizontalUpper,
    WallFullStairHorizontalLowerStairTopHorizontalUpper,
    WallInverseCornerStairHorizontalLowerStairTopVerticalLower,
    WallFullStairHorizontalLowerStairTopVerticalLower,
    WallCornerStairHorizontalLowerDoorEast,
    WallVerticalStairHorizontalLowerDoorEast,
    WallCornerStairHorizontalLowerDoorEastSouth,
    WallVerticalStairHorizontalLowerStairTopVerticalUpperDoorEast,
    WallVerticalStairHorizontalLowerStairTopHorizontalUpperDoorEast,
    WallVerticalStairHorizontalLowerStairTopVerticalLowerDoorEast,
    WallCornerStairVerticalUpper,
    WallVerticalStairVerticalUpper,
    WallCornerStairVerticalUpperDoorSouth,
    WallVerticalStairVerticalUpperStairTopVerticalUpper,
    WallVerticalStairVerticalUpperStairTopHorizontalUpper,
    WallVerticalStairVerticalUpperStairTopVerticalLower,
    WallHorizontalStairVerticalUpper,
    WallInverseCornerStairVerticalUpper,
    WallFullStairVerticalUpper,
    WallHorizontalStairVerticalUpperDoorSouth,
    WallInverseCornerStairVerticalUpperStairTopVerticalUpper,
    WallFullStairVerticalUpperStairTopVerticalUpper,
    WallInverseCornerStairVerticalUpperStairTopHorizontalUpper,
    WallFullStairVerticalUpperStairTopHorizontalUpper,
    WallInverseCornerStairVerticalUpperStairTopVerticalLower,
    WallFullStairVerticalUpperStairTopVerticalLower,
    WallCornerStairVerticalUpperDoorEast,
    WallVerticalStairVerticalUpperDoorEast,
    WallCornerStairVerticalUpperDoorEastSouth,
    WallVerticalStairVerticalUpperStairTopVerticalUpperDoorEast,
    WallVerticalStairVerticalUpperStairTopHorizontalUpperDoorEast,
    WallVerticalStairVerticalUpperStairTopVerticalLowerDoorEast,
    WallCornerStairHorizontalUpper,
    WallVerticalStairHorizontalUpper,
    WallCornerStairHorizontalUpperDoorSouth,
    WallVerticalStairHorizontalUpperStairTopVerticalUpper,
    WallVerticalStairHorizontalUpperStairTopHorizontalUpper,
    WallVerticalStairHorizontalUpperStairTopVerticalLower,
    WallHorizontalStairHorizontalUpper,
    WallInverseCornerStairHorizontalUpper,
    WallFullStairHorizontalUpper,
    WallHorizontalStairHorizontalUpperDoorSouth,
    WallInverseCornerStairHorizontalUpperStairTopVerticalUpper,
    WallFullStairHorizontalUpperStairTopVerticalUpper,
    WallInverseCornerStairHorizontalUpperStairTopHorizontalUpper,
    WallFullStairHorizontalUpperStairTopHorizontalUpper,
    WallInverseCornerStairHorizontalUpperStairTopVerticalLower,
    WallFullStairHorizontalUpperStairTopVerticalLower,
    WallCornerStairHorizontalUpperDoorEast,
    WallVerticalStairHorizontalUpperDoorEast,
    WallCornerStairHorizontalUpperDoorEastSouth,
    WallVerticalStairHorizontalUpperStairTopVerticalUpperDoorEast,
    WallVerticalStairHorizontalUpperStairTopHorizontalUpperDoorEast,
    WallVerticalStairHorizontalUpperStairTopVerticalLowerDoorEast,
}

pub fn sprite_offset(kind: TileKind, walls: Adjacency, doors: Adjacency) -> u16 {
    let offset = match kind {
        TileKind::Empty | TileKind::Stairs => empty_sprite_offset(walls),
        TileKind::Wall => wall_sprite_offset(walls, doors),
        TileKind::Door => door_sprite_offset(walls),
    };

    offset + WALL_OFFSET
}

fn empty_sprite_offset(walls: Adjacency) -> u16 {
    if walls.contains(Adjacency::SOUTH_EAST | Adjacency::SOUTH) {
        2
    } else if walls.contains(Adjacency::SOUTH) {
        1
    } else {
        0
    }
}

fn wall_sprite_offset(walls: Adjacency, doors: Adjacency) -> u16 {
    const LOOKUP: [u8; 256] = [
        3, 9, 12, 14, 3, 9, 12, 14, 4, 4, 13, 13, 4, 4, 13, 13, 3, 9, 12, 14, 3, 9, 12, 14, 4, 4,
        13, 13, 4, 4, 13, 13, 5, 10, 5, 10, 5, 10, 5, 10, 6, 6, 6, 6, 6, 6, 6, 6, 7, 11, 7, 11, 7,
        11, 7, 11, 8, 8, 8, 8, 8, 8, 8, 8, 3, 9, 12, 14, 3, 9, 12, 14, 4, 4, 13, 13, 4, 4, 13, 13,
        3, 9, 12, 14, 3, 9, 12, 14, 4, 4, 13, 13, 4, 4, 13, 13, 5, 10, 5, 10, 5, 10, 5, 10, 6, 6,
        6, 6, 6, 6, 6, 6, 7, 11, 7, 11, 7, 11, 7, 11, 8, 8, 8, 8, 8, 8, 8, 8, 3, 9, 12, 14, 3, 9,
        12, 14, 4, 4, 13, 13, 4, 4, 13, 13, 3, 9, 12, 14, 3, 9, 12, 14, 4, 4, 13, 13, 4, 4, 13, 13,
        5, 10, 5, 10, 5, 10, 5, 10, 6, 6, 6, 6, 6, 6, 6, 6, 7, 11, 7, 11, 7, 11, 7, 11, 8, 8, 8, 8,
        8, 8, 8, 8, 3, 9, 12, 14, 3, 9, 12, 14, 4, 4, 13, 13, 4, 4, 13, 13, 3, 9, 12, 14, 3, 9, 12,
        14, 4, 4, 13, 13, 4, 4, 13, 13, 5, 10, 5, 10, 5, 10, 5, 10, 6, 6, 6, 6, 6, 6, 6, 6, 7, 11,
        7, 11, 7, 11, 7, 11, 8, 8, 8, 8, 8, 8, 8, 8,
    ];

    let mut mask =
        walls.difference(Adjacency::NORTH_WEST | Adjacency::NORTH | Adjacency::NORTH_EAST);
    if doors.contains(Adjacency::EAST) {
        mask.insert(Adjacency::NORTH_WEST);
    }
    if doors.contains(Adjacency::SOUTH) {
        mask.insert(Adjacency::NORTH);
    }
    if doors.contains(Adjacency::WEST) {
        mask.insert(Adjacency::NORTH_EAST);
    }

    LOOKUP[mask.bits() as usize] as u16
}

fn door_sprite_offset(walls: Adjacency) -> u16 {
    const LOOKUP: [u8; 256] = [
        0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0,
        15, 15, 0, 0, 15, 15, 16, 16, 17, 17, 16, 16, 17, 17, 16, 16, 17, 17, 16, 16, 17, 17, 18,
        18, 19, 19, 18, 18, 19, 19, 18, 18, 19, 19, 18, 18, 19, 19, 0, 0, 15, 15, 0, 0, 15, 15, 0,
        0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 16, 16,
        17, 17, 16, 16, 17, 17, 16, 16, 17, 17, 16, 16, 17, 17, 18, 18, 19, 19, 18, 18, 19, 19, 18,
        18, 19, 19, 18, 18, 19, 19, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0,
        15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 16, 16, 17, 17, 16, 16, 17, 17, 16, 16,
        17, 17, 16, 16, 17, 17, 18, 18, 19, 19, 18, 18, 19, 19, 18, 18, 19, 19, 18, 18, 19, 19, 0,
        0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15, 15, 0, 0, 15,
        15, 0, 0, 15, 15, 16, 16, 17, 17, 16, 16, 17, 17, 16, 16, 17, 17, 16, 16, 17, 17, 18, 18,
        19, 19, 18, 18, 19, 19, 18, 18, 19, 19, 18, 18, 19, 19,
    ];

    LOOKUP[walls.bits() as usize] as u16
}

#[test]
fn test_sprite_offset() {
    use TileKind::*;

    let patterns: HashMap<(TileKind, TileKind, TileKind, TileKind), u16> = HashMap::new();

    for center in TileKind::iter() {
        for south in TileKind::iter() {
            for east in TileKind::iter() {
                for south_east in TileKind::iter() {
                    // let mut normal_south = south;
                    // let mut normal_east = east;
                    // let mut normal_south_east = south_east;

                    let offset = match (center, south, south_east, east) {
                        (Empty, Empty | Door, _, _) => 0,
                        (Empty, Wall, Empty | Door | Stairs, _) => 1,
                        (Empty, Wall, Wall, _) => 2,
                        (Empty, Stairs, _, _) => 3,
                        (Wall, Empty, Empty, Empty) => 4,
                        (Wall, Empty, Empty, Wall) => todo!(),
                        (Wall, Empty, Empty, Door) => todo!(),
                        (Wall, Empty, Empty, Stairs) => todo!(),
                        (Wall, Empty, Wall, Empty) => todo!(),
                        (Wall, Empty, Wall, Wall) => todo!(),
                        (Wall, Empty, Wall, Door) => todo!(),
                        (Wall, Empty, Wall, Stairs) => todo!(),
                        (Wall, Empty, Door, Empty) => todo!(),
                        (Wall, Empty, Door, Wall) => todo!(),
                        (Wall, Empty, Door, Door) => todo!(),
                        (Wall, Empty, Door, Stairs) => todo!(),
                        (Wall, Empty, Stairs, Empty) => todo!(),
                        (Wall, Empty, Stairs, Wall) => todo!(),
                        (Wall, Empty, Stairs, Door) => todo!(),
                        (Wall, Empty, Stairs, Stairs) => todo!(),
                        (Wall, Wall, Empty, Empty) => todo!(),
                        (Wall, Wall, Empty, Wall) => todo!(),
                        (Wall, Wall, Empty, Door) => todo!(),
                        (Wall, Wall, Empty, Stairs) => todo!(),
                        (Wall, Wall, Wall, Empty) => todo!(),
                        (Wall, Wall, Wall, Wall) => todo!(),
                        (Wall, Wall, Wall, Door) => todo!(),
                        (Wall, Wall, Wall, Stairs) => todo!(),
                        (Wall, Wall, Door, Empty) => todo!(),
                        (Wall, Wall, Door, Wall) => todo!(),
                        (Wall, Wall, Door, Door) => todo!(),
                        (Wall, Wall, Door, Stairs) => todo!(),
                        (Wall, Wall, Stairs, Empty) => todo!(),
                        (Wall, Wall, Stairs, Wall) => todo!(),
                        (Wall, Wall, Stairs, Door) => todo!(),
                        (Wall, Wall, Stairs, Stairs) => todo!(),
                        (Wall, Door, Empty, Empty) => todo!(),
                        (Wall, Door, Empty, Wall) => todo!(),
                        (Wall, Door, Empty, Door) => todo!(),
                        (Wall, Door, Empty, Stairs) => todo!(),
                        (Wall, Door, Wall, Empty) => todo!(),
                        (Wall, Door, Wall, Wall) => todo!(),
                        (Wall, Door, Wall, Door) => todo!(),
                        (Wall, Door, Wall, Stairs) => todo!(),
                        (Wall, Door, Door, Empty) => todo!(),
                        (Wall, Door, Door, Wall) => todo!(),
                        (Wall, Door, Door, Door) => todo!(),
                        (Wall, Door, Door, Stairs) => todo!(),
                        (Wall, Door, Stairs, Empty) => todo!(),
                        (Wall, Door, Stairs, Wall) => todo!(),
                        (Wall, Door, Stairs, Door) => todo!(),
                        (Wall, Door, Stairs, Stairs) => todo!(),
                        (Wall, Stairs, Empty, Empty) => todo!(),
                        (Wall, Stairs, Empty, Wall) => todo!(),
                        (Wall, Stairs, Empty, Door) => todo!(),
                        (Wall, Stairs, Empty, Stairs) => todo!(),
                        (Wall, Stairs, Wall, Empty) => todo!(),
                        (Wall, Stairs, Wall, Wall) => todo!(),
                        (Wall, Stairs, Wall, Door) => todo!(),
                        (Wall, Stairs, Wall, Stairs) => todo!(),
                        (Wall, Stairs, Door, Empty) => todo!(),
                        (Wall, Stairs, Door, Wall) => todo!(),
                        (Wall, Stairs, Door, Door) => todo!(),
                        (Wall, Stairs, Door, Stairs) => todo!(),
                        (Wall, Stairs, Stairs, Empty) => todo!(),
                        (Wall, Stairs, Stairs, Wall) => todo!(),
                        (Wall, Stairs, Stairs, Door) => todo!(),
                        (Wall, Stairs, Stairs, Stairs) => todo!(),
                        (Door, Empty, Empty, Empty) => todo!(),
                        (Door, Empty, Empty, Wall) => todo!(),
                        (Door, Empty, Empty, Door) => todo!(),
                        (Door, Empty, Empty, Stairs) => todo!(),
                        (Door, Empty, Wall, Empty) => todo!(),
                        (Door, Empty, Wall, Wall) => todo!(),
                        (Door, Empty, Wall, Door) => todo!(),
                        (Door, Empty, Wall, Stairs) => todo!(),
                        (Door, Empty, Door, Empty) => todo!(),
                        (Door, Empty, Door, Wall) => todo!(),
                        (Door, Empty, Door, Door) => todo!(),
                        (Door, Empty, Door, Stairs) => todo!(),
                        (Door, Empty, Stairs, Empty) => todo!(),
                        (Door, Empty, Stairs, Wall) => todo!(),
                        (Door, Empty, Stairs, Door) => todo!(),
                        (Door, Empty, Stairs, Stairs) => todo!(),
                        (Door, Wall, Empty, Empty) => todo!(),
                        (Door, Wall, Empty, Wall) => todo!(),
                        (Door, Wall, Empty, Door) => todo!(),
                        (Door, Wall, Empty, Stairs) => todo!(),
                        (Door, Wall, Wall, Empty) => todo!(),
                        (Door, Wall, Wall, Wall) => todo!(),
                        (Door, Wall, Wall, Door) => todo!(),
                        (Door, Wall, Wall, Stairs) => todo!(),
                        (Door, Wall, Door, Empty) => todo!(),
                        (Door, Wall, Door, Wall) => todo!(),
                        (Door, Wall, Door, Door) => todo!(),
                        (Door, Wall, Door, Stairs) => todo!(),
                        (Door, Wall, Stairs, Empty) => todo!(),
                        (Door, Wall, Stairs, Wall) => todo!(),
                        (Door, Wall, Stairs, Door) => todo!(),
                        (Door, Wall, Stairs, Stairs) => todo!(),
                        (Door, Door, Empty, Empty) => todo!(),
                        (Door, Door, Empty, Wall) => todo!(),
                        (Door, Door, Empty, Door) => todo!(),
                        (Door, Door, Empty, Stairs) => todo!(),
                        (Door, Door, Wall, Empty) => todo!(),
                        (Door, Door, Wall, Wall) => todo!(),
                        (Door, Door, Wall, Door) => todo!(),
                        (Door, Door, Wall, Stairs) => todo!(),
                        (Door, Door, Door, Empty) => todo!(),
                        (Door, Door, Door, Wall) => todo!(),
                        (Door, Door, Door, Door) => todo!(),
                        (Door, Door, Door, Stairs) => todo!(),
                        (Door, Door, Stairs, Empty) => todo!(),
                        (Door, Door, Stairs, Wall) => todo!(),
                        (Door, Door, Stairs, Door) => todo!(),
                        (Door, Door, Stairs, Stairs) => todo!(),
                        (Door, Stairs, Empty, Empty) => todo!(),
                        (Door, Stairs, Empty, Wall) => todo!(),
                        (Door, Stairs, Empty, Door) => todo!(),
                        (Door, Stairs, Empty, Stairs) => todo!(),
                        (Door, Stairs, Wall, Empty) => todo!(),
                        (Door, Stairs, Wall, Wall) => todo!(),
                        (Door, Stairs, Wall, Door) => todo!(),
                        (Door, Stairs, Wall, Stairs) => todo!(),
                        (Door, Stairs, Door, Empty) => todo!(),
                        (Door, Stairs, Door, Wall) => todo!(),
                        (Door, Stairs, Door, Door) => todo!(),
                        (Door, Stairs, Door, Stairs) => todo!(),
                        (Door, Stairs, Stairs, Empty) => todo!(),
                        (Door, Stairs, Stairs, Wall) => todo!(),
                        (Door, Stairs, Stairs, Door) => todo!(),
                        (Door, Stairs, Stairs, Stairs) => todo!(),
                        (Stairs, Empty, Empty, Empty) => todo!(),
                        (Stairs, Empty, Empty, Wall) => todo!(),
                        (Stairs, Empty, Empty, Door) => todo!(),
                        (Stairs, Empty, Empty, Stairs) => todo!(),
                        (Stairs, Empty, Wall, Empty) => todo!(),
                        (Stairs, Empty, Wall, Wall) => todo!(),
                        (Stairs, Empty, Wall, Door) => todo!(),
                        (Stairs, Empty, Wall, Stairs) => todo!(),
                        (Stairs, Empty, Door, Empty) => todo!(),
                        (Stairs, Empty, Door, Wall) => todo!(),
                        (Stairs, Empty, Door, Door) => todo!(),
                        (Stairs, Empty, Door, Stairs) => todo!(),
                        (Stairs, Empty, Stairs, Empty) => todo!(),
                        (Stairs, Empty, Stairs, Wall) => todo!(),
                        (Stairs, Empty, Stairs, Door) => todo!(),
                        (Stairs, Empty, Stairs, Stairs) => todo!(),
                        (Stairs, Wall, Empty, Empty) => todo!(),
                        (Stairs, Wall, Empty, Wall) => todo!(),
                        (Stairs, Wall, Empty, Door) => todo!(),
                        (Stairs, Wall, Empty, Stairs) => todo!(),
                        (Stairs, Wall, Wall, Empty) => todo!(),
                        (Stairs, Wall, Wall, Wall) => todo!(),
                        (Stairs, Wall, Wall, Door) => todo!(),
                        (Stairs, Wall, Wall, Stairs) => todo!(),
                        (Stairs, Wall, Door, Empty) => todo!(),
                        (Stairs, Wall, Door, Wall) => todo!(),
                        (Stairs, Wall, Door, Door) => todo!(),
                        (Stairs, Wall, Door, Stairs) => todo!(),
                        (Stairs, Wall, Stairs, Empty) => todo!(),
                        (Stairs, Wall, Stairs, Wall) => todo!(),
                        (Stairs, Wall, Stairs, Door) => todo!(),
                        (Stairs, Wall, Stairs, Stairs) => todo!(),
                        (Stairs, Door, Empty, Empty) => todo!(),
                        (Stairs, Door, Empty, Wall) => todo!(),
                        (Stairs, Door, Empty, Door) => todo!(),
                        (Stairs, Door, Empty, Stairs) => todo!(),
                        (Stairs, Door, Wall, Empty) => todo!(),
                        (Stairs, Door, Wall, Wall) => todo!(),
                        (Stairs, Door, Wall, Door) => todo!(),
                        (Stairs, Door, Wall, Stairs) => todo!(),
                        (Stairs, Door, Door, Empty) => todo!(),
                        (Stairs, Door, Door, Wall) => todo!(),
                        (Stairs, Door, Door, Door) => todo!(),
                        (Stairs, Door, Door, Stairs) => todo!(),
                        (Stairs, Door, Stairs, Empty) => todo!(),
                        (Stairs, Door, Stairs, Wall) => todo!(),
                        (Stairs, Door, Stairs, Door) => todo!(),
                        (Stairs, Door, Stairs, Stairs) => todo!(),
                        (Stairs, Stairs, Empty, Empty) => todo!(),
                        (Stairs, Stairs, Empty, Wall) => todo!(),
                        (Stairs, Stairs, Empty, Door) => todo!(),
                        (Stairs, Stairs, Empty, Stairs) => todo!(),
                        (Stairs, Stairs, Wall, Empty) => todo!(),
                        (Stairs, Stairs, Wall, Wall) => todo!(),
                        (Stairs, Stairs, Wall, Door) => todo!(),
                        (Stairs, Stairs, Wall, Stairs) => todo!(),
                        (Stairs, Stairs, Door, Empty) => todo!(),
                        (Stairs, Stairs, Door, Wall) => todo!(),
                        (Stairs, Stairs, Door, Door) => todo!(),
                        (Stairs, Stairs, Door, Stairs) => todo!(),
                        (Stairs, Stairs, Stairs, Empty) => todo!(),
                        (Stairs, Stairs, Stairs, Wall) => todo!(),
                        (Stairs, Stairs, Stairs, Door) => todo!(),
                        (Stairs, Stairs, Stairs, Stairs) => todo!(),
                    };

                    let mut adjacency = TileAdjacency::NONE;

                    adjacency.update(Adjacency::SOUTH, TileKind::Empty, south);
                    adjacency.update(Adjacency::EAST, TileKind::Empty, east);
                    adjacency.update(Adjacency::SOUTH_EAST, TileKind::Empty, south_east);

                    println!("material={center:?}, adjacency={adjacency:?}");
                }
            }
        }
    }
}

#[test]
fn test_empty_sprite_offset() {
    use std::collections::HashMap;

    let mut patterns: HashMap<Adjacency, u16> = HashMap::new();

    for walls in Adjacency::values() {
        let mut normal = walls.intersection(Adjacency::SOUTH_EAST | Adjacency::SOUTH);

        if !normal.contains(Adjacency::SOUTH) {
            normal = Adjacency::NONE;
        }

        let offset = patterns.len() as u16;
        if !patterns.contains_key(&normal) {
            println!("{}: walls={:?}", offset + 1, walls);
        }

        assert_eq!(
            empty_sprite_offset(walls),
            *patterns.entry(normal).or_insert(offset),
            "unexpected sprite index for walls={walls:?}"
        );
    }

    assert_eq!(patterns.len(), 3);
}

#[test]
fn test_wall_sprite_offset() {
    use std::collections::HashMap;

    let mut patterns: HashMap<(Adjacency, Adjacency), u16> = HashMap::new();

    for doors in Adjacency::values() {
        for walls in Adjacency::values() {
            let mut normal_walls =
                walls.intersection(Adjacency::EAST | Adjacency::SOUTH_EAST | Adjacency::SOUTH);

            let mut normal_doors = doors.intersection(Adjacency::EAST | Adjacency::SOUTH);

            if walls.contains(Adjacency::SOUTH) {
                normal_doors.remove(Adjacency::SOUTH);
            } else {
                normal_walls.remove(Adjacency::SOUTH_EAST);
            }

            if walls.contains(Adjacency::EAST) {
                normal_doors.remove(Adjacency::EAST);
            }

            let offset = 3 + patterns.len() as u16;
            if !patterns.contains_key(&(normal_walls, normal_doors)) {
                println!("{}: walls={:?}, doors={:?}", offset + 1, walls, doors);
            }

            assert_eq!(
                wall_sprite_offset(walls, doors),
                *patterns
                    .entry((normal_walls, normal_doors))
                    .or_insert(offset),
                "unexpected sprite index for walls={walls:?}, doors={doors:?}"
            );
        }
    }

    assert_eq!(patterns.len(), 12);
}

#[test]
fn test_door_sprite_offset() {
    use std::collections::HashMap;

    let mut patterns: HashMap<Adjacency, u16> = HashMap::new();

    for walls in Adjacency::values() {
        let mut normal_walls =
            walls.intersection(Adjacency::NORTH | Adjacency::SOUTH_EAST | Adjacency::SOUTH);

        if !normal_walls.contains(Adjacency::SOUTH) {
            normal_walls.remove(Adjacency::SOUTH_EAST);
        }

        if !walls.contains(Adjacency::SOUTH) {
            normal_walls.remove(Adjacency::SOUTH_EAST);
        }

        if normal_walls.is_empty() {
            assert_eq!(
                door_sprite_offset(walls),
                0,
                "unexpected sprite index for walls={walls:?}"
            );
        } else {
            let offset = 15 + patterns.len() as u16;
            if !patterns.contains_key(&normal_walls) {
                println!("{}: walls={:?}", offset + 1, walls);
            }

            assert_eq!(
                door_sprite_offset(walls),
                *patterns.entry(normal_walls).or_insert(offset),
                "unexpected sprite index for walls={walls:?}"
            );
        }
    }

    assert_eq!(patterns.len(), 5);
}
