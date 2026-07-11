use wdn_physics::tile::{adjacency::Adjacency, material::TileKind};

use crate::tile::WALL_OFFSET;

pub fn sprite_offset(kind: TileKind, walls: Adjacency, doors: Adjacency) -> u16 {
    let offset = match kind {
        TileKind::Empty | TileKind::Stairs => empty_sprite_offset(walls),
        TileKind::Wall => wall_sprite_offset(walls, doors),
        TileKind::Door => door_sprite_offset(walls),
    };

    offset + WALL_OFFSET
}

fn empty_sprite_offset(walls: Adjacency) -> u16 {
    const LOOKUP: [u8; 256] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    ];

    LOOKUP[walls.bits() as usize] as u16
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
                normal_walls.remove(Adjacency::SOUTH_WEST | Adjacency::SOUTH_EAST);
            }

            if walls.contains(Adjacency::WEST) {
                normal_doors.remove(Adjacency::WEST);
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
            normal_walls.remove(Adjacency::SOUTH_WEST | Adjacency::SOUTH_EAST);
        }

        if !walls.contains(Adjacency::SOUTH) {
            normal_walls.remove(Adjacency::SOUTH_WEST | Adjacency::SOUTH_EAST);
        }

        normal_walls.remove(Adjacency::WEST | Adjacency::EAST);

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
