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
            _,
        ) => 9,
        (Wall, StairWest | Wall, DoorVertical, _) => 10,
        (
            Wall,
            StairWest | Wall,
            StairNorth | Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 11,
        (Wall, StairWest | Wall, StairNorth | Wall, StairNorth | StairWest | Wall) => 12,
        (Wall, StairNorth, DoorHorizontal | Empty | StairSouth | StairWest, _) => 13,
        (Wall, StairNorth, DoorVertical, _) => 14,
        (Wall, StairNorth, StairNorth | Wall, _) => 15,
        (Wall, StairNorth, StairEast, _) => 16,
        (Wall, StairSouth, DoorHorizontal | Empty | StairEast | StairSouth | StairWest, _) => 17,
        (Wall, StairSouth, DoorVertical, _) => 18,
        (Wall, StairSouth, StairNorth | Wall, _) => 19,
        (StairNorth, DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth, _, _) => 20,
        (StairNorth, StairNorth | StairWest | Wall, _, _) => 21,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            DoorHorizontal | Empty | StairSouth | StairWest,
            _,
        ) => 22,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            DoorVertical,
            _,
        ) => 23,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairNorth | Wall,
            _,
        ) => 24,
        (
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            _,
        ) => 25,
        (
            StairSouth,
            StairSouth | StairWest | Wall,
            DoorHorizontal | Empty | StairEast | StairSouth | StairWest,
            _,
        ) => 26,
        (StairSouth, StairSouth | StairWest | Wall, DoorVertical, _) => 27,
        (
            StairSouth,
            StairSouth | StairWest | Wall,
            StairNorth | Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 28,
        (
            StairSouth,
            StairSouth | StairWest | Wall,
            StairNorth | Wall,
            StairNorth | StairWest | Wall,
        ) => 29,
        (
            StairEast,
            DoorVertical | Empty | StairEast,
            DoorHorizontal | DoorVertical | Empty | StairSouth | StairWest,
            _,
        ) => 30,
        (StairEast, DoorVertical | Empty | StairEast, StairEast | StairNorth | Wall, _) => 31,
        (
            StairEast,
            DoorHorizontal,
            DoorHorizontal | DoorVertical | Empty | StairSouth | StairWest,
            _,
        ) => 32,
        (StairEast, DoorHorizontal, StairEast | StairNorth | Wall, _) => 33,
        (
            StairEast,
            StairWest | Wall,
            DoorHorizontal | DoorVertical | Empty | StairSouth | StairWest,
            _,
        ) => 34,
        (StairEast, StairWest | Wall, StairEast | StairNorth | Wall, _) => 35,
        (
            StairEast,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairSouth | StairWest,
            _,
        ) => 36,
        (StairEast, StairNorth, StairEast | StairNorth | Wall, _) => 37,
        (
            StairEast,
            StairSouth,
            DoorHorizontal | DoorVertical | Empty | StairSouth | StairWest,
            _,
        ) => 38,
        (StairEast, StairSouth, StairEast | StairNorth | Wall, _) => 39,
        (StairWest, _, DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth, _) => 40,
        (StairWest, _, StairNorth | StairWest | Wall, _) => 41,
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
        | (StairWest, _, StairWest, _) => 0,
        (
            DoorHorizontal | Empty | StairNorth | StairWest,
            _,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 42,
        (
            DoorHorizontal | Empty | StairEast | StairNorth | StairWest,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            Wall,
            StairWest | Wall,
        )
        | (
            DoorHorizontal | Empty | StairNorth | StairWest,
            StairSouth | StairWest | Wall,
            Wall,
            StairWest | Wall,
        ) => 43,
        (
            DoorHorizontal | Empty | StairEast | StairNorth | StairWest,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            Wall,
            StairNorth,
        )
        | (
            DoorHorizontal | Empty | StairNorth | StairWest,
            StairSouth | StairWest | Wall,
            Wall,
            StairNorth,
        ) => 44,
        (
            DoorHorizontal | Empty | StairNorth | StairWest,
            _,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 45,
        (
            DoorHorizontal | Empty | StairEast | StairNorth | StairWest,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairNorth,
            StairNorth | StairWest | Wall,
        )
        | (
            DoorHorizontal | Empty | StairNorth | StairWest,
            StairSouth | StairWest | Wall,
            StairNorth,
            StairNorth | StairWest | Wall,
        ) => 46,
        (
            DoorHorizontal | DoorVertical | Empty | StairWest,
            _,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 47,
        (DoorHorizontal | DoorVertical | Empty | StairWest, _, StairEast, StairWest | Wall) => 48,
        (DoorHorizontal | DoorVertical | Empty | StairWest, _, StairEast, StairNorth) => 49,
        (DoorHorizontal | DoorVertical | Empty | StairNorth | StairEast, _, StairWest, _) => 50,
        (DoorVertical, _, Wall, DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth) => {
            51
        }
        (DoorVertical, _, Wall, StairWest | Wall) => 52,
        (DoorVertical, _, Wall, StairNorth) => 53,
        (
            DoorVertical,
            _,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 54,
        (DoorVertical, _, StairNorth, StairNorth | StairWest | Wall) => 55,
        (
            StairSouth | Wall,
            _,
            Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 56,
        (
            StairSouth | Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            Wall,
            StairWest | Wall,
        ) => 57,
        (
            StairSouth | Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            Wall,
            StairNorth,
        ) => 58,
        (
            StairSouth | Wall,
            _,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 59,
        (
            StairSouth | Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairNorth,
            StairNorth | StairWest | Wall,
        ) => 60,
        (
            StairSouth | Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 61,
        (
            StairSouth | Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            StairWest | Wall,
        ) => 62,
        (
            StairSouth | Wall,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            StairNorth,
        ) => 63,
        (StairSouth | Wall, _, StairWest, _) => 64,
        (StairSouth | Wall, StairSouth | StairWest | Wall, Wall, StairWest | Wall) => 65,
        (StairSouth | Wall, StairSouth | StairWest | Wall, Wall, StairNorth) => 66,
        (
            StairSouth | Wall,
            StairSouth | StairWest | Wall,
            StairNorth,
            StairNorth | StairWest | Wall,
        ) => 67,
        (
            StairSouth | Wall,
            StairSouth | StairWest | Wall,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 68,
        (StairSouth | Wall, StairSouth | StairWest | Wall, StairEast, StairWest | Wall) => 69,
        (StairSouth | Wall, StairSouth | StairWest | Wall, StairEast, StairNorth) => 70,
        (
            StairNorth,
            _,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 71,
        (StairNorth, _, StairEast, StairWest | Wall) => 72,
        (StairNorth, _, StairEast, StairNorth) => 73,
        (StairEast, _, Wall, DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth) => 74,
        (
            StairEast,
            _,
            StairNorth,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 75,
        (
            StairEast,
            _,
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairSouth,
        ) => 76,
        (
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            StairWest | Wall,
        ) => 77,
        (
            StairEast,
            DoorHorizontal | DoorVertical | Empty | StairEast | StairNorth,
            StairEast,
            StairNorth,
        ) => 78,
        (StairEast, StairSouth | StairWest | Wall, Wall, StairWest | Wall) => 79,
        (StairEast, StairSouth | StairWest | Wall, Wall, StairNorth) => 80,
        (StairEast, StairSouth | StairWest | Wall, StairNorth, StairNorth | StairWest | Wall) => 81,
        (StairEast, StairSouth | StairWest | Wall, StairEast, StairWest | Wall) => 82,
        (StairEast, StairSouth | StairWest | Wall, StairEast, StairNorth) => 83,
    }
}
