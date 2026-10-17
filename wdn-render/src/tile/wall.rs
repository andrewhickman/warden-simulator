#![deny(clippy::match_same_arms)]

use wdn_physics::tile::{
    material::{TileKind, TileMaterialFlags},
    position::TilePosition,
    storage::TileStorage,
};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum TileVariant {
    Empty,
    Wall,
    DoorHorizontal,
    DoorVertical,
    StairNorth,
    StairSouth,
    StairEast,
    StairWest,
}

pub fn mid_offsets(storage: &TileStorage, tile: TilePosition) -> (u16, u16) {
    let left = resolve_base(
        get_tile_variant(storage, tile).flip(),
        get_tile_variant(storage, tile.west()).flip(),
        get_tile_variant(storage, tile.south().west()).flip(),
        get_tile_variant(storage, tile.south()).flip(),
    );

    let right = resolve_base(
        get_tile_variant(storage, tile),
        get_tile_variant(storage, tile.east()),
        get_tile_variant(storage, tile.south().east()),
        get_tile_variant(storage, tile.south()),
    );

    (left, right)
}

pub fn top_offset(storage: &TileStorage, tile: TilePosition) -> (u16, u16) {
    let left = resolve_top(
        get_tile_variant(storage, tile).flip(),
        get_tile_variant(storage, tile.west()).flip(),
        get_tile_variant(storage, tile.south().west()).flip(),
        get_tile_variant(storage, tile.south()).flip(),
    );

    let right = resolve_top(
        get_tile_variant(storage, tile),
        get_tile_variant(storage, tile.east()),
        get_tile_variant(storage, tile.south().east()),
        get_tile_variant(storage, tile.south()),
    );

    (left, right)
}

fn get_tile_variant(storage: &TileStorage, tile: TilePosition) -> TileVariant {
    let Some(data) = storage.get(tile) else {
        return TileVariant::Empty;
    };

    let material = data.material();
    match material.kind() {
        TileKind::Empty => TileVariant::Empty,
        TileKind::Wall => TileVariant::Wall,
        TileKind::Door => match material
            .flags()
            .intersection(TileMaterialFlags::DOOR_DIRECTION_MASK)
        {
            TileMaterialFlags::DOOR_DIRECTION_HORIZONTAL => TileVariant::DoorHorizontal,
            TileMaterialFlags::DOOR_DIRECTION_VERTICAL => TileVariant::DoorVertical,
            _ => unreachable!("Invalid door direction for tile at {tile:?}"),
        },
        TileKind::Stairs => match material
            .flags()
            .intersection(TileMaterialFlags::STAIR_DIRECTION_MASK)
        {
            TileMaterialFlags::STAIR_DIRECTION_NORTH => TileVariant::StairNorth,
            TileMaterialFlags::STAIR_DIRECTION_SOUTH => TileVariant::StairSouth,
            TileMaterialFlags::STAIR_DIRECTION_EAST => TileVariant::StairEast,
            TileMaterialFlags::STAIR_DIRECTION_WEST => TileVariant::StairWest,
            _ => unreachable!("Invalid stair direction for tile at {tile:?}"),
        },
    }
}

impl TileVariant {
    fn flip(&self) -> Self {
        use TileVariant::*;

        match self {
            StairEast => StairWest,
            StairWest => StairEast,
            _ => *self,
        }
    }
}

fn resolve_base(
    center: TileVariant,
    east: TileVariant,
    south_east: TileVariant,
    south: TileVariant,
) -> u16 {
    use TileVariant::*;

    match (center, east, south, south_east) {
        (DoorHorizontal | DoorVertical | Empty, _, _, _) => 0,
        (
            Wall,
            DoorVertical | Empty | StairEast,
            DoorHorizontal | Empty | StairSouth | StairWest,
            _,
        ) => 1,
        (Wall, DoorVertical | Empty | StairEast, DoorVertical, _) => 2,
        (Wall, DoorVertical | Empty | StairEast, StairNorth | Wall, _) => 3,
        (Wall, DoorVertical | Empty | StairEast, StairEast, _) => 4,
        (Wall, DoorHorizontal, DoorHorizontal | Empty | StairSouth | StairWest, _) => 5,
        (Wall, DoorHorizontal, DoorVertical, _) => 6,
        (Wall, DoorHorizontal, StairNorth | Wall, _) => 7,
        (Wall, DoorHorizontal, StairEast, _) => 8,
        (
            Wall,
            StairWest | Wall,
            DoorHorizontal | Empty | StairEast | StairSouth | StairWest,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        )
        | (
            Wall,
            StairWest | Wall,
            DoorHorizontal | Empty | StairSouth | StairWest,
            StairNorth | StairWest | Wall,
        ) => 9,
        (Wall, StairWest | Wall, DoorVertical, _) => 10,
        (
            Wall,
            StairWest | Wall,
            StairNorth | Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 11,
        (Wall, StairWest | Wall, StairNorth | Wall, StairNorth | StairWest | Wall) => 12,
        (Wall, StairWest | Wall, StairEast, StairNorth | StairWest | Wall) => 13,
        (Wall, StairNorth, DoorHorizontal | Empty | StairSouth | StairWest, _) => 14,
        (Wall, StairNorth, DoorVertical, _) => 15,
        (Wall, StairNorth, StairNorth | Wall, _) => 16,
        (Wall, StairNorth, StairEast, _) => 17,
        (Wall, StairSouth, DoorHorizontal | Empty | StairEast | StairSouth | StairWest, _) => 18,
        (Wall, StairSouth, DoorVertical, _) => 19,
        (Wall, StairSouth, StairNorth | Wall, _) => 20,
        (StairNorth, DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth, _, _) => 21,
        (StairNorth, StairNorth | StairWest | Wall, _, _) => 22,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            DoorHorizontal | Empty | StairSouth | StairWest,
            _,
        ) => 23,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            DoorVertical,
            _,
        ) => 24,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairNorth | Wall,
            _,
        ) => 25,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            _,
        ) => 26,
        (
            StairSouth,
            StairSouth | StairWest | Wall,
            DoorHorizontal | Empty | StairEast | StairSouth | StairWest,
            _,
        ) => 27,
        (StairSouth, StairSouth | StairWest | Wall, DoorVertical, _) => 28,
        (
            StairSouth,
            StairSouth | StairWest | Wall,
            StairNorth | Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 29,
        (
            StairSouth,
            StairSouth | StairWest | Wall,
            StairNorth | Wall,
            StairNorth | StairWest | Wall,
        ) => 30,
        (
            StairEast,
            DoorVertical | Empty | StairEast,
            DoorHorizontal | DoorVertical | Empty | StairSouth | StairWest,
            _,
        ) => 31,
        (StairEast, DoorVertical | Empty | StairEast, StairEast | StairNorth | Wall, _) => 32,
        (
            StairEast,
            DoorHorizontal,
            DoorHorizontal | DoorVertical | Empty | StairSouth | StairWest,
            _,
        ) => 33,
        (StairEast, DoorHorizontal, StairEast | StairNorth | Wall, _) => 34,
        (
            StairEast,
            StairWest | Wall,
            DoorHorizontal | DoorVertical | Empty | StairSouth | StairWest,
            _,
        ) => 35,
        (StairEast, StairWest | Wall, StairEast | StairNorth | Wall, _) => 36,
        (
            StairEast,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairSouth | StairWest,
            _,
        ) => 37,
        (StairEast, StairNorth, StairEast | StairNorth | Wall, _) => 38,
        (
            StairEast,
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairSouth | StairWest,
            _,
        ) => 39,
        (StairEast, StairSouth, StairEast | StairNorth | Wall, _) => 40,
        (StairWest, _, DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth, _) => 41,
        (StairWest, _, StairNorth | StairWest | Wall, _) => 42,
    }
}

fn resolve_top(
    center: TileVariant,
    east: TileVariant,
    south_east: TileVariant,
    south: TileVariant,
) -> u16 {
    use TileVariant::*;

    match (center, east, south, south_east) {
        (_, _, DoorHorizontal | DoorVertical | Empty | StairSouth, _)
        | (StairNorth | StairWest, _, StairWest, _) => 0,
        (
            DoorHorizontal | Empty | StairNorth | StairWest,
            _,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 43,
        (DoorHorizontal | Empty | StairNorth | StairWest, _, Wall, StairWest | Wall) => 44,
        (DoorHorizontal | Empty | StairNorth | StairWest, _, Wall, StairNorth) => 45,
        (
            DoorHorizontal | Empty | StairNorth | StairWest,
            _,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 46,
        (
            DoorHorizontal | Empty | StairNorth | StairWest,
            _,
            StairNorth,
            StairNorth | StairWest | Wall,
        ) => 47,
        (
            DoorHorizontal | DoorVertical | Empty | StairWest,
            _,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 48,
        (DoorHorizontal | DoorVertical | Empty | StairWest, _, StairEast, StairWest | Wall) => 49,
        (DoorHorizontal | DoorVertical | Empty | StairWest, _, StairEast, StairNorth) => 50,
        (DoorHorizontal | DoorVertical | Empty | StairEast, _, StairWest, _) => 51,
        (DoorVertical, _, Wall, DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth) => {
            52
        }
        (DoorVertical, _, Wall, StairWest | Wall) => 53,
        (DoorVertical, _, Wall, StairNorth) => 54,
        (
            DoorVertical,
            _,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 55,
        (DoorVertical, _, StairNorth, StairNorth | StairWest | Wall) => 56,
        (
            Wall,
            DoorVertical | Empty | StairEast | StairNorth,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 57,
        (Wall, DoorVertical | Empty | StairEast | StairNorth, Wall, StairWest | Wall) => 58,
        (Wall, DoorVertical | Empty | StairEast | StairNorth, Wall, StairNorth) => 59,
        (
            Wall,
            DoorVertical | Empty | StairEast | StairNorth,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 60,
        (
            Wall,
            DoorVertical | Empty | StairEast | StairNorth,
            StairNorth,
            StairNorth | StairWest | Wall,
        ) => 61,
        (
            Wall,
            DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 62,
        (Wall, DoorVertical | Empty | StairEast | StairNorth, StairEast, StairWest | Wall) => 63,
        (Wall, DoorVertical | Empty | StairEast | StairNorth, StairEast, StairNorth) => 64,
        (StairSouth | Wall, _, StairWest, _) => 65,
        (
            Wall,
            DoorHorizontal,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 66,
        (Wall, DoorHorizontal, Wall, StairWest | Wall) => 67,
        (Wall, DoorHorizontal, Wall, StairNorth) => 68,
        (
            Wall,
            DoorHorizontal,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 69,
        (Wall, DoorHorizontal, StairNorth, StairNorth | StairWest | Wall) => 70,
        (
            Wall,
            DoorHorizontal,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 71,
        (Wall, DoorHorizontal, StairEast, StairWest | Wall) => 72,
        (Wall, DoorHorizontal, StairEast, StairNorth) => 73,
        (
            Wall,
            StairWest | Wall,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 74,
        (Wall, StairWest | Wall, Wall, StairWest | Wall) => 75,
        (Wall, StairWest | Wall, Wall, StairNorth) => 76,
        (
            Wall,
            StairWest | Wall,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 77,
        (Wall, StairWest | Wall, StairNorth, StairNorth | StairWest | Wall) => 78,
        (
            Wall,
            StairWest | Wall,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 79,
        (Wall, StairWest | Wall, StairEast, StairWest | Wall) => 80,
        (Wall, StairWest | Wall, StairEast, StairNorth) => 81,
        (
            Wall,
            StairSouth,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 82,
        (Wall, StairSouth, Wall, StairWest | Wall) => 83,
        (Wall, StairSouth, Wall, StairNorth) => 84,
        (
            Wall,
            StairSouth,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 85,
        (Wall, StairSouth, StairNorth, StairNorth | StairWest | Wall) => 86,
        (
            Wall,
            StairSouth,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 87,
        (Wall, StairSouth, StairEast, StairWest | Wall) => 88,
        (Wall, StairSouth, StairEast, StairNorth) => 89,
        (
            StairNorth,
            _,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 90,
        (StairNorth, _, StairEast, StairWest | Wall) => 91,
        (StairNorth, _, StairEast, StairNorth) => 92,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 93,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            Wall,
            StairWest | Wall,
        ) => 94,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            Wall,
            StairNorth,
        ) => 95,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 96,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairNorth,
            StairNorth | StairWest | Wall,
        ) => 97,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 98,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            StairWest | Wall,
        ) => 99,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            StairNorth,
        ) => 100,
        (
            StairSouth,
            StairSouth | StairWest | Wall,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 101,
        (StairSouth, StairSouth | StairWest | Wall, Wall, StairWest | Wall) => 102,
        (StairSouth, StairSouth | StairWest | Wall, Wall, StairNorth) => 103,
        (
            StairSouth,
            StairSouth | StairWest | Wall,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 104,
        (StairSouth, StairSouth | StairWest | Wall, StairNorth, StairNorth | StairWest | Wall) => {
            105
        }
        (
            StairSouth,
            StairSouth | StairWest | Wall,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 106,
        (StairSouth, StairSouth | StairWest | Wall, StairEast, StairWest | Wall) => 107,
        (StairSouth, StairSouth | StairWest | Wall, StairEast, StairNorth) => 108,
        (
            StairEast,
            DoorVertical | Empty | StairEast | StairNorth,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 109,
        (StairEast, DoorVertical | Empty | StairEast | StairNorth, Wall, StairWest | Wall) => 110,
        (StairEast, DoorVertical | Empty | StairEast | StairNorth, Wall, StairNorth) => 111,
        (
            StairEast,
            DoorVertical | Empty | StairEast | StairNorth,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 112,
        (
            StairEast,
            DoorVertical | Empty | StairEast | StairNorth,
            StairNorth,
            StairNorth | StairWest | Wall,
        ) => 113,
        (
            StairEast,
            DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 114,
        (StairEast, DoorVertical | Empty | StairEast | StairNorth, StairEast, StairWest | Wall) => {
            115
        }
        (StairEast, DoorVertical | Empty | StairEast | StairNorth, StairEast, StairNorth) => 116,
        (
            StairEast,
            DoorHorizontal,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 117,
        (StairEast, DoorHorizontal, Wall, StairWest | Wall) => 118,
        (StairEast, DoorHorizontal, Wall, StairNorth) => 119,
        (
            StairEast,
            DoorHorizontal,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 120,
        (StairEast, DoorHorizontal, StairNorth, StairNorth | StairWest | Wall) => 121,
        (
            StairEast,
            DoorHorizontal,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 122,
        (StairEast, DoorHorizontal, StairEast, StairWest | Wall) => 123,
        (StairEast, DoorHorizontal, StairEast, StairNorth) => 124,
        (
            StairEast,
            StairWest | Wall,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 125,
        (StairEast, StairWest | Wall, Wall, StairWest | Wall) => 126,
        (StairEast, StairWest | Wall, Wall, StairNorth) => 127,
        (
            StairEast,
            StairWest | Wall,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 128,
        (StairEast, StairWest | Wall, StairNorth, StairNorth | StairWest | Wall) => 129,
        (
            StairEast,
            StairWest | Wall,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 130,
        (StairEast, StairWest | Wall, StairEast, StairWest | Wall) => 131,
        (StairEast, StairWest | Wall, StairEast, StairNorth) => 132,
        (
            StairEast,
            StairSouth,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 133,
        (StairEast, StairSouth, Wall, StairWest | Wall) => 134,
        (StairEast, StairSouth, Wall, StairNorth) => 135,
        (
            StairEast,
            StairSouth,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 136,
        (StairEast, StairSouth, StairNorth, StairNorth | StairWest | Wall) => 137,
        (
            StairEast,
            StairSouth,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 138,
        (StairEast, StairSouth, StairEast, StairWest | Wall) => 139,
        (StairEast, StairSouth, StairEast, StairNorth) => 140,
    }
}
