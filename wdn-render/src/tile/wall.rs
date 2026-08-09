use wdn_physics::tile::{adjacency::Adjacency, material::TileKind};

pub fn mid_offsets(kind: TileKind, walls: Adjacency, doors: Adjacency) -> (u16, u16) {
    if kind == TileKind::Wall {
        wall_mid_offset(walls, doors)
    } else {
        (0, 0)
    }
}

pub fn top_offset(kind: TileKind, walls: Adjacency) -> (u16, u16) {
    match kind {
        TileKind::Empty | TileKind::Stairs => {
            (empty_top_offset(walls.flip_x()), empty_top_offset(walls))
        }
        TileKind::Wall => (wall_top_offset(walls.flip_x()), wall_top_offset(walls)),
        TileKind::Door => (door_top_offset(walls.flip_x()), door_top_offset(walls)),
    }
}

fn wall_mid_offset(walls: Adjacency, doors: Adjacency) -> (u16, u16) {
    const LOOKUP: [u8; 256] = [
        1, 5, 7, 9, 1, 5, 7, 9, 2, 2, 8, 8, 2, 2, 8, 8, 1, 5, 7, 9, 1, 5, 7, 9, 2, 2, 8, 8, 2, 2,
        8, 8, 3, 6, 3, 6, 3, 6, 3, 6, 4, 4, 4, 4, 4, 4, 4, 4, 3, 6, 3, 6, 3, 6, 3, 6, 4, 4, 4, 4,
        4, 4, 4, 4, 1, 5, 7, 9, 1, 5, 7, 9, 2, 2, 8, 8, 2, 2, 8, 8, 1, 5, 7, 9, 1, 5, 7, 9, 2, 2,
        8, 8, 2, 2, 8, 8, 3, 6, 3, 6, 3, 6, 3, 6, 4, 4, 4, 4, 4, 4, 4, 4, 3, 6, 3, 6, 3, 6, 3, 6,
        4, 4, 4, 4, 4, 4, 4, 4, 1, 5, 7, 9, 1, 5, 7, 9, 2, 2, 8, 8, 2, 2, 8, 8, 1, 5, 7, 9, 1, 5,
        7, 9, 2, 2, 8, 8, 2, 2, 8, 8, 3, 6, 3, 6, 3, 6, 3, 6, 4, 4, 4, 4, 4, 4, 4, 4, 3, 6, 3, 6,
        3, 6, 3, 6, 4, 4, 4, 4, 4, 4, 4, 4, 1, 5, 7, 9, 1, 5, 7, 9, 2, 2, 8, 8, 2, 2, 8, 8, 1, 5,
        7, 9, 1, 5, 7, 9, 2, 2, 8, 8, 2, 2, 8, 8, 3, 6, 3, 6, 3, 6, 3, 6, 4, 4, 4, 4, 4, 4, 4, 4,
        3, 6, 3, 6, 3, 6, 3, 6, 4, 4, 4, 4, 4, 4, 4, 4,
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

    (
        LOOKUP[mask.flip_x().bits() as usize] as u16,
        LOOKUP[mask.bits() as usize] as u16,
    )
}

fn empty_top_offset(walls: Adjacency) -> u16 {
    if walls.contains(Adjacency::SOUTH) {
        if walls.contains(Adjacency::SOUTH_EAST) {
            11
        } else {
            10
        }
    } else {
        0
    }
}

fn wall_top_offset(walls: Adjacency) -> u16 {
    if walls.contains(Adjacency::SOUTH | Adjacency::SOUTH_EAST) {
        if walls.contains(Adjacency::EAST) {
            12
        } else {
            11
        }
    } else {
        0
    }
}

fn door_top_offset(walls: Adjacency) -> u16 {
    if walls.contains(Adjacency::SOUTH) {
        if walls.contains(Adjacency::SOUTH_EAST) {
            14
        } else {
            13
        }
    } else {
        0
    }
}

// #[test]
// fn test_wall_sprite_offset() {
//     use std::collections::HashMap;

//     let mut patterns: HashMap<(Adjacency, Adjacency), u16> = HashMap::new();

//     for doors in Adjacency::values() {
//         for walls in Adjacency::values() {
//             let normal_walls =
//                 walls.intersection(Adjacency::EAST | Adjacency::SOUTH | Adjacency::WEST);
//             let normal_doors = doors
//                 .intersection(Adjacency::EAST | Adjacency::SOUTH | Adjacency::WEST)
//                 .difference(normal_walls);

//             let offset = 1 + patterns.len() as u16;
//             assert_eq!(
//                 wall_sprite_offset(walls, doors),
//                 *patterns
//                     .entry((normal_walls, normal_doors))
//                     .or_insert(offset),
//                 "unexpected sprite index for walls={walls:?}, doors={doors:?}"
//             );
//         }
//     }
// }
