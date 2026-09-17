#![deny(clippy::match_same_arms)]

use wdn_physics::tile::{
    material::{TileKind, TileMaterialFlags},
    position::TilePosition,
    storage::TileStorage,
};

const EMPTY: u16 = 0;
const BASE_WALLCORNER: u16 = 1;
const BASE_WALLVERTICAL: u16 = 2;
const BASE_WALLCORNER_SOUTHDOOR: u16 = 3;
const BASE_WALLVERTICAL_SOUTHSTAIRN: u16 = 4;
const BASE_WALLHORIZONTAL: u16 = 5;
const BASE_WALLINVERSECORNER: u16 = 6;
const BASE_WALLHORIZONTAL_SOUTHDOOR: u16 = 7;
const BASE_WALLINVERSECORNER_SOUTHSTAIRN: u16 = 8;
const BASE_WALLFULL: u16 = 9;
const BASE_WALLFULL_SOUTHSTAIRNFULL: u16 = 10;
const BASE_WALLCORNER_EASTDOOR: u16 = 11;
const BASE_WALLVERTICAL_EASTDOOR: u16 = 12;
const BASE_WALLCORNER_EASTDOOR_SOUTHDOOR: u16 = 13;
const BASE_WALLVERTICAL_EASTDOOR_SOUTHSTAIRN: u16 = 14;
const BASE_WALLCORNER_EASTSTAIRN: u16 = 15;
const BASE_WALLVERTICAL_EASTSTAIRN: u16 = 16;
const BASE_WALLCORNER_EASTSTAIRN_SOUTHDOOR: u16 = 17;
const BASE_WALLVERTICAL_EASTSTAIRN_SOUTHSTAIRN: u16 = 18;
const BASE_WALLCORNER_EASTSTAIRS: u16 = 19;
const BASE_WALLVERTICAL_EASTSTAIRS: u16 = 20;
const BASE_WALLCORNER_EASTSTAIRS_SOUTHDOOR: u16 = 21;
const BASE_WALLVERTICAL_EASTSTAIRS_SOUTHSTAIRN: u16 = 22;
const BASE_WALLVERTICAL_EASTSTAIRS_SOUTHSTAIRNFULL: u16 = 23;
const BASE_STAIRN: u16 = 24;
const BASE_STAIRNFULL: u16 = 25;
const BASE_STAIRS: u16 = 26;
const BASE_STAIRSFULL: u16 = 27;
const BASE_STAIRE: u16 = 28;
const BASE_STAIREFULL: u16 = 29;
const BASE_STAIRW: u16 = 30;
const BASE_STAIRWFULL: u16 = 31;
const TOP_EMPTY_SOUTHWALLCORNER: u16 = 32;
const TOP_EMPTY_SOUTHSTAIRN: u16 = 33;
const TOP_EMPTY_SOUTHSTAIRE: u16 = 34;
const TOP_EMPTY_SOUTHSTAIRW: u16 = 35;
const TOP_EMPTY_SOUTHWALLHORIZONTAL: u16 = 36;
const TOP_EMPTY_SOUTHSTAIRNFULL: u16 = 37;
const TOP_EMPTY_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 38;
const TOP_WALLVERTICAL_SOUTHSTAIRN: u16 = 39;
const TOP_EMPTY_SOUTHSTAIREFULL: u16 = 40;
const TOP_EMPTY_SOUTHSTAIRWFULL: u16 = 41;
const TOP_WALLVERTICAL_SOUTHWALLHORIZONTAL: u16 = 42;
const TOP_WALLVERTICAL_SOUTHSTAIRNFULL: u16 = 43;
const TOP_WALLVERTICAL_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 44;
const TOP_WALLINVERSECORNER_SOUTHSTAIRN: u16 = 45;
const TOP_WALLHORIZONTAL_SOUTHSTAIREFULL: u16 = 46;
const TOP_WALLFULL_SOUTHSTAIRNFULL: u16 = 47;
const TOP_WALLINVERSECORNER_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 48;
const TOP_WALLINVERSECORNER_SOUTHWALLHORIZONTAL: u16 = 49;
const TOP_WALLVERTICAL_SOUTHWALLCORNER_EASTDOOR: u16 = 50;
const TOP_WALLVERTICAL_SOUTHSTAIRN_EASTDOOR: u16 = 51;
const TOP_EMPTY_SOUTHSTAIREFULL_EASTDOOR: u16 = 52;
const TOP_EMPTY_SOUTHSTAIRWFULL_EASTDOOR: u16 = 53;
const TOP_WALLVERTICAL_SOUTHWALLHORIZONTAL_EASTDOOR: u16 = 54;
const TOP_WALLVERTICAL_SOUTHSTAIRNFULL_EASTDOOR: u16 = 55;
const TOP_WALLVERTICAL_SOUTHWALLCORNER_SOUTHEASTSTAIRN_EASTDOOR: u16 = 56;
const TOP_DOOR_SOUTHWALLCORNER: u16 = 57;
const TOP_DOOR_SOUTHWALLHORIZONTAL: u16 = 58;
const TOP_DOOR_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 59;
const TOP_STAIRS_SOUTHWALLCORNER: u16 = 60;
const TOP_STAIRS_SOUTHWALLHORIZONTAL: u16 = 61;
const TOP_STAIRS_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 62;
const TOP_STAIRSFULL_SOUTHWALLCORNER: u16 = 63;
const TOP_STAIRSFULL_SOUTHSTAIRN: u16 = 64;
const TOP_STAIRSFULL_SOUTHWALLHORIZONTAL: u16 = 65;
const TOP_STAIRSFULL_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 66;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum TileVariant {
    Empty,
    Wall,
    Door,
    StairN,
    StairS,
    StairE,
    StairW,
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
        TileKind::Door => TileVariant::Door,
        TileKind::Stairs => match material
            .flags()
            .intersection(TileMaterialFlags::STAIR_DIRECTION_MASK)
        {
            TileMaterialFlags::STAIR_DIRECTION_NORTH => TileVariant::StairN,
            TileMaterialFlags::STAIR_DIRECTION_SOUTH => TileVariant::StairS,
            TileMaterialFlags::STAIR_DIRECTION_EAST => TileVariant::StairE,
            TileMaterialFlags::STAIR_DIRECTION_WEST => TileVariant::StairW,
            _ => unreachable!("Invalid stair direction for tile at {tile:?}"),
        },
    }
}

impl TileVariant {
    fn flip(&self) -> Self {
        use TileVariant::*;

        match self {
            StairE => StairW,
            StairW => StairE,
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

    match (center, east, south_east, south) {
        (Door | Empty, _, _, _) => EMPTY,
        (Wall, Empty | StairE, _, Empty | StairE | StairS | StairW)
        | (Wall, Empty | StairE, Wall, Door) => BASE_WALLCORNER,
        (Wall, Empty | StairE, _, Wall) => BASE_WALLVERTICAL,
        (Wall, Empty | StairE, Door | Empty | StairE | StairN | StairS | StairW, Door) => {
            BASE_WALLCORNER_SOUTHDOOR
        }
        (Wall, Empty | StairE, _, StairN) => BASE_WALLVERTICAL_SOUTHSTAIRN,
        (Wall, StairW | Wall, _, Empty | StairE | StairS | StairW)
        | (Wall, StairW | Wall, Wall, Door) => BASE_WALLHORIZONTAL,
        (Wall, StairW | Wall, Door | Empty | StairE | StairN | StairS | StairW, Wall) => {
            BASE_WALLINVERSECORNER
        }
        (Wall, StairW | Wall, Door | Empty | StairE | StairN | StairS | StairW, Door) => {
            BASE_WALLHORIZONTAL_SOUTHDOOR
        }
        (Wall, StairW | Wall, Door | Empty | StairE | StairS | StairW, StairN) => {
            BASE_WALLINVERSECORNER_SOUTHSTAIRN
        }
        (Wall, StairW | Wall, Wall, Wall) => BASE_WALLFULL,
        (Wall, StairW | Wall, StairN | Wall, StairN) => BASE_WALLFULL_SOUTHSTAIRNFULL,
        (Wall, Door, _, Empty | StairE | StairS | StairW) | (Wall, Door, Wall, Door) => {
            BASE_WALLCORNER_EASTDOOR
        }
        (Wall, Door, _, Wall) => BASE_WALLVERTICAL_EASTDOOR,
        (Wall, Door, Door | Empty | StairE | StairN | StairS | StairW, Door) => {
            BASE_WALLCORNER_EASTDOOR_SOUTHDOOR
        }
        (Wall, Door, _, StairN) => BASE_WALLVERTICAL_EASTDOOR_SOUTHSTAIRN,
        (Wall, StairN, _, Empty | StairE | StairS | StairW) | (Wall, StairN, Wall, Door) => {
            BASE_WALLCORNER_EASTSTAIRN
        }
        (Wall, StairN, _, Wall) => BASE_WALLVERTICAL_EASTSTAIRN,
        (Wall, StairN, Door | Empty | StairE | StairN | StairS | StairW, Door) => {
            BASE_WALLCORNER_EASTSTAIRN_SOUTHDOOR
        }
        (Wall, StairN, _, StairN) => BASE_WALLVERTICAL_EASTSTAIRN_SOUTHSTAIRN,
        (Wall, StairS, _, Empty | StairE | StairS | StairW) | (Wall, StairS, Wall, Door) => {
            BASE_WALLCORNER_EASTSTAIRS
        }
        (Wall, StairS, _, Wall) => BASE_WALLVERTICAL_EASTSTAIRS,
        (Wall, StairS, Door | Empty | StairE | StairN | StairS | StairW, Door) => {
            BASE_WALLCORNER_EASTSTAIRS_SOUTHDOOR
        }
        (Wall, StairS, Door | Empty | StairE | StairS | StairW, StairN) => {
            BASE_WALLVERTICAL_EASTSTAIRS_SOUTHSTAIRN
        }
        (Wall, StairS, StairN | Wall, StairN) => BASE_WALLVERTICAL_EASTSTAIRS_SOUTHSTAIRNFULL,
        (StairN, Door | Empty | StairE | StairS | StairW, _, _) => BASE_STAIRN,
        (StairN, StairN | Wall, _, _) => BASE_STAIRNFULL,
        (StairS, Door | Empty | StairE | StairN | StairW, _, _) => BASE_STAIRS,
        (StairS, StairS | Wall, _, _) => BASE_STAIRSFULL,
        (StairE, _, _, Door | Empty | StairN | StairS | StairW) => BASE_STAIRE,
        (StairE, _, _, StairE | Wall) => BASE_STAIREFULL,
        (StairW, _, _, Door | Empty | StairE | StairN | StairS) => BASE_STAIRW,
        (StairW, _, _, StairW | Wall) => BASE_STAIRWFULL,
    }
}

fn resolve_top(
    center: TileVariant,
    east: TileVariant,
    south_east: TileVariant,
    south: TileVariant,
) -> u16 {
    use TileVariant::*;

    match (center, east, south_east, south) {
        (_, _, _, Door | Empty | StairS)
        | (
            Wall,
            Empty | StairE | StairN | StairS | StairW | Wall,
            Door | Empty | StairE | StairS,
            Wall,
        )
        | (Wall, StairW | Wall, Wall, Wall)
        | (StairS, _, _, StairE | StairW) => EMPTY,
        (
            Empty | StairE | StairN | StairW,
            Door | Empty | StairE | StairN | StairS | StairW,
            Door | Empty | StairE | StairS,
            Wall,
        )
        | (Door | Empty | StairE | StairN | StairW, Wall, Door | Empty | StairE | StairS, Wall) => {
            TOP_EMPTY_SOUTHWALLCORNER
        }
        (
            Door | Empty | StairE | StairN | StairS | StairW,
            Door | Empty | StairE | StairN | StairW,
            Door | Empty | StairE | StairS | StairW,
            StairN,
        )
        | (
            Door | Empty | StairE | StairN | StairW,
            StairS | Wall,
            Door | Empty | StairE | StairS | StairW,
            StairN,
        ) => TOP_EMPTY_SOUTHSTAIRN,
        (Door | Empty | StairN | StairW, _, _, StairE) => TOP_EMPTY_SOUTHSTAIRE,
        (Door | Empty | StairE | StairN, _, _, StairW) => TOP_EMPTY_SOUTHSTAIRW,
        (
            Empty | StairE | StairN | StairW,
            Door | Empty | StairE | StairN | StairS | StairW,
            StairW | Wall,
            Wall,
        )
        | (Door | Empty | StairE | StairN | StairW, Wall, StairW | Wall, Wall) => {
            TOP_EMPTY_SOUTHWALLHORIZONTAL
        }
        (Door | Empty | StairE | StairN | StairS | StairW, _, StairN | Wall, StairN) => {
            TOP_EMPTY_SOUTHSTAIRNFULL
        }
        (
            Empty | StairE | StairN | StairW,
            Door | Empty | StairE | StairN | StairS | StairW,
            StairN,
            Wall,
        )
        | (Door | Empty | StairE | StairN | StairW, Wall, StairN, Wall) => {
            TOP_EMPTY_SOUTHWALLCORNER_SOUTHEASTSTAIRN
        }
        (
            Wall,
            Empty | StairE | StairN | StairS,
            Door | Empty | StairE | StairS | StairW,
            StairN,
        ) => TOP_WALLVERTICAL_SOUTHSTAIRN,
        (StairE | Wall, Empty | StairE | StairN | StairS, _, StairE)
        | (StairE, Door | StairW | Wall, _, StairE) => TOP_EMPTY_SOUTHSTAIREFULL,
        (StairW | Wall, Empty | StairE | StairN | StairS | StairW | Wall, _, StairW)
        | (StairW, Door, _, StairW) => TOP_EMPTY_SOUTHSTAIRWFULL,
        (Wall, Empty | StairE | StairN | StairS, StairW | Wall, Wall) => {
            TOP_WALLVERTICAL_SOUTHWALLHORIZONTAL
        }
        (Wall, Empty | StairE | StairN | StairS, StairN | Wall, StairN) => {
            TOP_WALLVERTICAL_SOUTHSTAIRNFULL
        }
        (Wall, Empty | StairE | StairN | StairS, StairN, Wall) => {
            TOP_WALLVERTICAL_SOUTHWALLCORNER_SOUTHEASTSTAIRN
        }
        (Wall, StairW | Wall, Door | Empty | StairE | StairS | StairW, StairN) => {
            TOP_WALLINVERSECORNER_SOUTHSTAIRN
        }
        (Wall, StairW | Wall, _, StairE) => TOP_WALLHORIZONTAL_SOUTHSTAIREFULL,
        (Wall, StairW | Wall, StairN | Wall, StairN) => TOP_WALLFULL_SOUTHSTAIRNFULL,
        (Wall, StairW | Wall, StairN, Wall) => {
            TOP_WALLINVERSECORNER_SOUTHWALLCORNER_SOUTHEASTSTAIRN
        }
        (Wall, StairW | Wall, StairW, Wall) => TOP_WALLINVERSECORNER_SOUTHWALLHORIZONTAL,
        (Wall, Door, Door | Empty | StairE | StairS, Wall) => {
            TOP_WALLVERTICAL_SOUTHWALLCORNER_EASTDOOR
        }
        (Wall, Door, Door | Empty | StairE | StairS | StairW, StairN) => {
            TOP_WALLVERTICAL_SOUTHSTAIRN_EASTDOOR
        }
        (Wall, Door, _, StairE) => TOP_EMPTY_SOUTHSTAIREFULL_EASTDOOR,
        (Wall, Door, _, StairW) => TOP_EMPTY_SOUTHSTAIRWFULL_EASTDOOR,
        (Wall, Door, StairW | Wall, Wall) => TOP_WALLVERTICAL_SOUTHWALLHORIZONTAL_EASTDOOR,
        (Wall, Door, StairN | Wall, StairN) => TOP_WALLVERTICAL_SOUTHSTAIRNFULL_EASTDOOR,
        (Wall, Door, StairN, Wall) => TOP_WALLVERTICAL_SOUTHWALLCORNER_SOUTHEASTSTAIRN_EASTDOOR,
        (
            Door,
            Door | Empty | StairE | StairN | StairS | StairW,
            Door | Empty | StairE | StairS,
            Wall,
        ) => TOP_DOOR_SOUTHWALLCORNER,
        (Door, Door | Empty | StairE | StairN | StairS | StairW, StairW | Wall, Wall) => {
            TOP_DOOR_SOUTHWALLHORIZONTAL
        }
        (Door, Door | Empty | StairE | StairN | StairS | StairW, StairN, Wall) => {
            TOP_DOOR_SOUTHWALLCORNER_SOUTHEASTSTAIRN
        }
        (StairS, Door | Empty | StairE | StairN | StairW, Door | Empty | StairE | StairS, Wall) => {
            TOP_STAIRS_SOUTHWALLCORNER
        }
        (StairS, Door | Empty | StairE | StairN | StairW, StairW | Wall, Wall) => {
            TOP_STAIRS_SOUTHWALLHORIZONTAL
        }
        (StairS, Door | Empty | StairE | StairN | StairW, StairN, Wall) => {
            TOP_STAIRS_SOUTHWALLCORNER_SOUTHEASTSTAIRN
        }
        (StairS, StairS | Wall, Door | Empty | StairE | StairS, Wall) => {
            TOP_STAIRSFULL_SOUTHWALLCORNER
        }
        (StairS, StairS | Wall, Door | Empty | StairE | StairS | StairW, StairN) => {
            TOP_STAIRSFULL_SOUTHSTAIRN
        }
        (StairS, StairS | Wall, StairW | Wall, Wall) => TOP_STAIRSFULL_SOUTHWALLHORIZONTAL,
        (StairS, StairS | Wall, StairN, Wall) => TOP_STAIRSFULL_SOUTHWALLCORNER_SOUTHEASTSTAIRN,
    }
}
