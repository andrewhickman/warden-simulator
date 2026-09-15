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
const BASE_WALLHORIZONTAL: u16 = 4;
const BASE_WALLINVERSECORNER: u16 = 5;
const BASE_WALLHORIZONTAL_SOUTHDOOR: u16 = 6;
const BASE_WALLFULL: u16 = 7;
const BASE_WALLCORNER_EASTDOOR: u16 = 8;
const BASE_WALLVERTICAL_EASTDOOR: u16 = 9;
const BASE_WALLCORNER_EASTDOOR_SOUTHDOOR: u16 = 10;
const BASE_WALLCORNER_EASTSTAIRN: u16 = 11;
const BASE_WALLVERTICAL_EASTSTAIRN: u16 = 12;
const BASE_WALLCORNER_EASTSTAIRN_SOUTHDOOR: u16 = 13;
const BASE_WALLCORNER_EASTSTAIRS: u16 = 14;
const BASE_WALLVERTICAL_EASTSTAIRS: u16 = 15;
const BASE_WALLCORNER_EASTSTAIRS_SOUTHDOOR: u16 = 16;
const BASE_STAIRN: u16 = 17;
const BASE_STAIRNFULL: u16 = 18;
const BASE_STAIRS: u16 = 19;
const BASE_STAIRSFULL: u16 = 20;
const BASE_STAIRE: u16 = 21;
const BASE_STAIREFULL: u16 = 22;
const BASE_STAIRW: u16 = 23;
const BASE_STAIRWFULL: u16 = 24;
const TOP_EMPTY_SOUTHWALLCORNER: u16 = 25;
const TOP_EMPTY_SOUTHSTAIRN: u16 = 26;
const TOP_EMPTY_SOUTHSTAIRE: u16 = 27;
const TOP_EMPTY_SOUTHSTAIRW: u16 = 28;
const TOP_EMPTY_SOUTHWALLHORIZONTAL: u16 = 29;
const TOP_EMPTY_SOUTHSTAIRNFULL: u16 = 30;
const TOP_EMPTY_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 31;
const TOP_EMPTY_SOUTHEASTSTAIRW: u16 = 32;
const TOP_EMPTY_SOUTHSTAIRN_SOUTHEASTSTAIRW: u16 = 33;
const TOP_EMPTY_SOUTHSTAIRE_SOUTHEASTSTAIRW: u16 = 34;
const TOP_EMPTY_SOUTHSTAIRW_SOUTHEASTSTAIRW: u16 = 35;
const TOP_EMPTY_SOUTHEASTSTAIRWFULL: u16 = 36;
const TOP_EMPTY_SOUTHWALLHORIZONTAL_SOUTHEASTSTAIRWFULL: u16 = 37;
const TOP_EMPTY_SOUTHSTAIRN_SOUTHEASTSTAIRWFULL: u16 = 38;
const TOP_EMPTY_SOUTHSTAIRE_SOUTHEASTSTAIRWFULL: u16 = 39;
const TOP_EMPTY_SOUTHSTAIRW_SOUTHEASTSTAIRWFULL: u16 = 40;
const TOP_EMPTY_EASTSTAIRW: u16 = 41;
const TOP_EMPTY_SOUTHWALLCORNER_EASTSTAIRW: u16 = 42;
const TOP_EMPTY_SOUTHSTAIRN_EASTSTAIRW: u16 = 43;
const TOP_EMPTY_SOUTHSTAIRE_EASTSTAIRW: u16 = 44;
const TOP_EMPTY_SOUTHSTAIRW_EASTSTAIRW: u16 = 45;
const TOP_EMPTY_EASTSTAIRWFULL: u16 = 46;
const TOP_EMPTY_SOUTHWALLHORIZONTAL_EASTSTAIRWFULL: u16 = 47;
const TOP_EMPTY_SOUTHSTAIRNFULL_EASTSTAIRWFULL: u16 = 48;
const TOP_EMPTY_SOUTHSTAIRE_EASTSTAIRWFULL: u16 = 49;
const TOP_EMPTY_SOUTHSTAIRW_EASTSTAIRWFULL: u16 = 50;
const TOP_EMPTY_SOUTHWALLCORNER_SOUTHEASTSTAIRN_EASTSTAIRW: u16 = 51;
const TOP_EMPTY_SOUTHSTAIRNFULL_SOUTHEASTSTAIRN_EASTSTAIRW: u16 = 52;
const TOP_WALLCORNER: u16 = 53;
const TOP_WALLVERTICAL_SOUTHWALLCORNER: u16 = 54;
const TOP_WALLCORNER_SOUTHDOOR: u16 = 55;
const TOP_WALLVERTICAL_SOUTHSTAIRN: u16 = 56;
const TOP_WALLCORNER_SOUTHSTAIREFULL: u16 = 57;
const TOP_WALLCORNER_SOUTHSTAIRWFULL: u16 = 58;
const TOP_WALLVERTICAL_SOUTHWALLHORIZONTAL: u16 = 59;
const TOP_WALLVERTICAL_SOUTHSTAIRNFULL: u16 = 60;
const TOP_WALLVERTICAL_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 61;
const TOP_WALLCORNER_SOUTHEASTSTAIRW: u16 = 62;
const TOP_WALLCORNER_SOUTHDOOR_SOUTHEASTSTAIRW: u16 = 63;
const TOP_WALLVERTICAL_SOUTHSTAIRN_SOUTHEASTSTAIRW: u16 = 64;
const TOP_WALLCORNER_SOUTHSTAIREFULL_SOUTHEASTSTAIRW: u16 = 65;
const TOP_WALLCORNER_SOUTHSTAIRWFULL_SOUTHEASTSTAIRW: u16 = 66;
const TOP_WALLHORIZONTAL: u16 = 67;
const TOP_WALLINVERSECORNER_SOUTHWALLCORNER: u16 = 68;
const TOP_WALLHORIZONTAL_SOUTHDOOR: u16 = 69;
const TOP_WALLINVERSECORNER_SOUTHSTAIRN: u16 = 70;
const TOP_WALLHORIZONTAL_SOUTHSTAIREFULL: u16 = 71;
const TOP_WALLHORIZONTAL_SOUTHSTAIRWFULL: u16 = 72;
const TOP_WALLFULL: u16 = 73;
const TOP_WALLFULL_SOUTHSTAIRNFULL: u16 = 74;
const TOP_WALLINVERSECORNER_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 75;
const TOP_WALLHORIZONTAL_SOUTHEASTSTAIRWFULL: u16 = 76;
const TOP_WALLFULL_SOUTHWALLHORIZONTAL: u16 = 73;
const TOP_WALLHORIZONTAL_SOUTHDOOR_SOUTHEASTSTAIRWFULL: u16 = 77;
const TOP_WALLINVERSECORNER_SOUTHSTAIRN_SOUTHEASTSTAIRWFULL: u16 = 78;
const TOP_WALLHORIZONTAL_SOUTHSTAIREFULL_SOUTHEASTSTAIRWFULL: u16 = 79;
const TOP_WALLHORIZONTAL_SOUTHSTAIRWFULL_SOUTHEASTSTAIRWFULL: u16 = 80;
const TOP_WALLCORNER_EASTDOOR: u16 = 81;
const TOP_WALLVERTICAL_SOUTHWALLCORNER_EASTDOOR: u16 = 82;
const TOP_WALLCORNER_SOUTHDOOR_EASTDOOR: u16 = 83;
const TOP_WALLVERTICAL_SOUTHSTAIRN_EASTDOOR: u16 = 84;
const TOP_WALLCORNER_SOUTHSTAIREFULL_EASTDOOR: u16 = 85;
const TOP_WALLCORNER_SOUTHSTAIRWFULL_EASTDOOR: u16 = 86;
const TOP_WALLVERTICAL_SOUTHWALLHORIZONTAL_EASTDOOR: u16 = 87;
const TOP_WALLVERTICAL_SOUTHSTAIRNFULL_EASTDOOR: u16 = 88;
const TOP_WALLVERTICAL_SOUTHWALLCORNER_SOUTHEASTSTAIRN_EASTDOOR: u16 = 89;
const TOP_WALLCORNER_SOUTHEASTSTAIRW_EASTDOOR: u16 = 90;
const TOP_WALLCORNER_SOUTHDOOR_SOUTHEASTSTAIRW_EASTDOOR: u16 = 91;
const TOP_WALLVERTICAL_SOUTHSTAIRN_SOUTHEASTSTAIRW_EASTDOOR: u16 = 92;
const TOP_WALLCORNER_SOUTHSTAIREFULL_SOUTHEASTSTAIRW_EASTDOOR: u16 = 93;
const TOP_WALLCORNER_SOUTHSTAIRWFULL_SOUTHEASTSTAIRW_EASTDOOR: u16 = 94;
const TOP_DOOR_SOUTHWALLCORNER: u16 = 95;
const TOP_DOOR_SOUTHWALLHORIZONTAL: u16 = 96;
const TOP_DOOR_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 97;
const TOP_DOOR_SOUTHWALLCORNER_EASTSTAIRW: u16 = 98;
const TOP_DOOR_SOUTHWALLHORIZONTAL_EASTSTAIRWFULL: u16 = 99;
const TOP_DOOR_SOUTHWALLCORNER_SOUTHEASTSTAIRN_EASTSTAIRW: u16 = 100;
const TOP_STAIRS: u16 = 101;
const TOP_STAIRS_SOUTHWALLCORNER: u16 = 102;
const TOP_STAIRS_SOUTHSTAIRE: u16 = 103;
const TOP_STAIRS_SOUTHSTAIRW: u16 = 104;
const TOP_STAIRS_SOUTHWALLHORIZONTAL: u16 = 105;
const TOP_STAIRS_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 106;
const TOP_STAIRS_SOUTHEASTSTAIRW: u16 = 107;
const TOP_STAIRS_SOUTHSTAIRE_SOUTHEASTSTAIRW: u16 = 108;
const TOP_STAIRS_SOUTHSTAIRW_SOUTHEASTSTAIRW: u16 = 109;
const TOP_STAIRSFULL: u16 = 110;
const TOP_STAIRSFULL_SOUTHWALLCORNER: u16 = 111;
const TOP_STAIRSFULL_SOUTHSTAIRN: u16 = 112;
const TOP_STAIRSFULL_SOUTHSTAIRE: u16 = 113;
const TOP_STAIRSFULL_SOUTHSTAIRW: u16 = 114;
const TOP_STAIRSFULL_SOUTHWALLHORIZONTAL: u16 = 115;
const TOP_STAIRSFULL_SOUTHWALLCORNER_SOUTHEASTSTAIRN: u16 = 116;
const TOP_STAIRSFULL_SOUTHEASTSTAIRWFULL: u16 = 117;
const TOP_STAIRSFULL_SOUTHWALLHORIZONTAL_SOUTHEASTSTAIRWFULL: u16 = 118;
const TOP_STAIRSFULL_SOUTHSTAIRE_SOUTHEASTSTAIRWFULL: u16 = 119;
const TOP_STAIRSFULL_SOUTHSTAIRW_SOUTHEASTSTAIRWFULL: u16 = 120;
const TOP_STAIRSFULL_SOUTHEASTSTAIRW: u16 = 121;
const TOP_STAIRSFULL_SOUTHSTAIRN_SOUTHEASTSTAIRW: u16 = 122;
const TOP_STAIRSFULL_SOUTHSTAIRE_SOUTHEASTSTAIRW: u16 = 123;
const TOP_STAIRSFULL_SOUTHSTAIRW_SOUTHEASTSTAIRW: u16 = 124;
const TOP_STAIRS_EASTSTAIRW: u16 = 125;
const TOP_STAIRS_SOUTHWALLCORNER_EASTSTAIRW: u16 = 126;
const TOP_STAIRS_SOUTHSTAIRE_EASTSTAIRW: u16 = 127;
const TOP_STAIRS_SOUTHSTAIRW_EASTSTAIRW: u16 = 128;
const TOP_STAIRS_EASTSTAIRWFULL: u16 = 129;
const TOP_STAIRS_SOUTHWALLHORIZONTAL_EASTSTAIRWFULL: u16 = 130;
const TOP_STAIRS_SOUTHSTAIRE_EASTSTAIRWFULL: u16 = 131;
const TOP_STAIRS_SOUTHSTAIRW_EASTSTAIRWFULL: u16 = 132;
const TOP_STAIRS_SOUTHWALLCORNER_SOUTHEASTSTAIRN_EASTSTAIRW: u16 = 133;
const TOP_EMPTY_SOUTHSTAIREFULL: u16 = 134;
const TOP_EMPTY_SOUTHSTAIREFULL_SOUTHEASTSTAIRW: u16 = 135;
const TOP_EMPTY_SOUTHSTAIREFULL_SOUTHEASTSTAIRWFULL: u16 = 136;
const TOP_EMPTY_SOUTHSTAIREFULL_EASTSTAIRW: u16 = 137;
const TOP_EMPTY_SOUTHSTAIREFULL_EASTSTAIRWFULL: u16 = 138;
const TOP_EMPTY_SOUTHSTAIRWFULL: u16 = 139;
const TOP_EMPTY_SOUTHSTAIRWFULL_SOUTHEASTSTAIRW: u16 = 140;
const TOP_EMPTY_SOUTHSTAIRWFULL_SOUTHEASTSTAIRWFULL: u16 = 141;
const TOP_EMPTY_SOUTHSTAIRWFULL_EASTSTAIRW: u16 = 142;
const TOP_EMPTY_SOUTHSTAIRWFULL_EASTSTAIRWFULL: u16 = 143;

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
        (Wall, Empty | StairE, _, StairN | Wall) => BASE_WALLVERTICAL,
        (Wall, Empty | StairE, Door | Empty | StairE | StairN | StairS | StairW, Door) => {
            BASE_WALLCORNER_SOUTHDOOR
        }
        (Wall, StairW | Wall, _, Empty | StairE | StairS | StairW)
        | (Wall, StairW | Wall, Wall, Door) => BASE_WALLHORIZONTAL,
        (Wall, StairW | Wall, Door | Empty | StairE | StairN | StairS | StairW, StairN | Wall) => {
            BASE_WALLINVERSECORNER
        }
        (Wall, StairW | Wall, Door | Empty | StairE | StairN | StairS | StairW, Door) => {
            BASE_WALLHORIZONTAL_SOUTHDOOR
        }
        (Wall, StairW | Wall, Wall, StairN | Wall) => BASE_WALLFULL,
        (Wall, Door, _, Empty | StairE | StairS | StairW) | (Wall, Door, Wall, Door) => {
            BASE_WALLCORNER_EASTDOOR
        }
        (Wall, Door, _, StairN | Wall) => BASE_WALLVERTICAL_EASTDOOR,
        (Wall, Door, Door | Empty | StairE | StairN | StairS | StairW, Door) => {
            BASE_WALLCORNER_EASTDOOR_SOUTHDOOR
        }
        (Wall, StairN, _, Empty | StairE | StairS | StairW) | (Wall, StairN, Wall, Door) => {
            BASE_WALLCORNER_EASTSTAIRN
        }
        (Wall, StairN, _, StairN | Wall) => BASE_WALLVERTICAL_EASTSTAIRN,
        (Wall, StairN, Door | Empty | StairE | StairN | StairS | StairW, Door) => {
            BASE_WALLCORNER_EASTSTAIRN_SOUTHDOOR
        }
        (Wall, StairS, _, Empty | StairE | StairS | StairW) | (Wall, StairS, Wall, Door) => {
            BASE_WALLCORNER_EASTSTAIRS
        }
        (Wall, StairS, _, StairN | Wall) => BASE_WALLVERTICAL_EASTSTAIRS,
        (Wall, StairS, Door | Empty | StairE | StairN | StairS | StairW, Door) => {
            BASE_WALLCORNER_EASTSTAIRS_SOUTHDOOR
        }
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
        (
            Door | Empty | StairE | StairN | StairW,
            Door | Empty | StairE | StairN | StairS | Wall,
            Door | Empty | StairE | StairN | StairS | Wall,
            Door | Empty | StairS,
        ) => TOP_EMPTY,
        (
            Empty | StairE | StairN | StairW,
            Door | Empty | StairE | StairN | StairS,
            Door | Empty | StairE | StairS,
            Wall,
        )
        | (Door | Empty | StairE | StairN | StairW, Wall, Door | Empty | StairE | StairS, Wall) => {
            TOP_EMPTY_SOUTHWALLCORNER
        }
        (
            Door | Empty | StairE | StairN | StairS | StairW,
            Door | Empty | StairE | StairN,
            Door | Empty | StairE | StairS,
            StairN,
        )
        | (
            Door | Empty | StairE | StairN | StairW,
            StairS | Wall,
            Door | Empty | StairE | StairS,
            StairN,
        ) => TOP_EMPTY_SOUTHSTAIRN,
        (
            Door | Empty | StairN | StairW,
            Door | Empty | StairE | StairN | StairS | Wall,
            Door | Empty | StairE | StairN | StairS | Wall,
            StairE,
        ) => TOP_EMPTY_SOUTHSTAIRE,
        (
            Door | Empty | StairE | StairN,
            Door | Empty | StairE | StairN | StairS | Wall,
            Door | Empty | StairE | StairN | StairS | Wall,
            StairW,
        ) => TOP_EMPTY_SOUTHSTAIRW,
        (
            Empty | StairE | StairN | StairW,
            Door | Empty | StairE | StairN | StairS,
            StairW | Wall,
            Wall,
        )
        | (Door | Empty | StairE | StairN | StairW, Wall, Wall, Wall) => {
            TOP_EMPTY_SOUTHWALLHORIZONTAL
        }
        (
            Door | Empty | StairE | StairN | StairS | StairW,
            Door | Empty | StairE | StairN | StairS | Wall,
            StairN | Wall,
            StairN,
        ) => TOP_EMPTY_SOUTHSTAIRNFULL,
        (
            Empty | StairE | StairN | StairW,
            Door | Empty | StairE | StairN | StairS,
            StairN,
            Wall,
        )
        | (Door | Empty | StairE | StairN | StairW, Wall, StairN, Wall) => {
            TOP_EMPTY_SOUTHWALLCORNER_SOUTHEASTSTAIRN
        }
        (
            Door | Empty | StairE | StairN | StairW,
            Door | Empty | StairE | StairN | StairS,
            StairW,
            Door | Empty | StairS,
        ) => TOP_EMPTY_SOUTHEASTSTAIRW,
        (
            Door | Empty | StairE | StairN | StairS | StairW,
            Door | Empty | StairE | StairN,
            StairW,
            StairN,
        )
        | (Door | Empty | StairE | StairN | StairW, StairS, StairW, StairN) => {
            TOP_EMPTY_SOUTHSTAIRN_SOUTHEASTSTAIRW
        }
        (
            Door | Empty | StairN | StairW,
            Door | Empty | StairE | StairN | StairS,
            StairW,
            StairE,
        ) => TOP_EMPTY_SOUTHSTAIRE_SOUTHEASTSTAIRW,
        (
            Door | Empty | StairE | StairN,
            Door | Empty | StairE | StairN | StairS,
            StairW,
            StairW,
        ) => TOP_EMPTY_SOUTHSTAIRW_SOUTHEASTSTAIRW,
        (Door | Empty | StairE | StairN | StairW, Wall, StairW, Door | Empty | StairS) => {
            TOP_EMPTY_SOUTHEASTSTAIRWFULL
        }
        (Door | Empty | StairE | StairN | StairW, Wall, StairW, Wall) => {
            TOP_EMPTY_SOUTHWALLHORIZONTAL_SOUTHEASTSTAIRWFULL
        }
        (Door | Empty | StairE | StairN | StairS | StairW, Wall, StairW, StairN) => {
            TOP_EMPTY_SOUTHSTAIRN_SOUTHEASTSTAIRWFULL
        }
        (Door | Empty | StairN | StairW, Wall, StairW, StairE) => {
            TOP_EMPTY_SOUTHSTAIRE_SOUTHEASTSTAIRWFULL
        }
        (Door | Empty | StairE | StairN, Wall, StairW, StairW) => {
            TOP_EMPTY_SOUTHSTAIRW_SOUTHEASTSTAIRWFULL
        }
        (
            Door | Empty | StairE | StairN | StairW,
            StairW,
            Door | Empty | StairE | StairN | StairS,
            Door | Empty | StairS,
        ) => TOP_EMPTY_EASTSTAIRW,
        (Empty | StairE | StairN | StairW, StairW, Door | Empty | StairE | StairS, Wall) => {
            TOP_EMPTY_SOUTHWALLCORNER_EASTSTAIRW
        }
        (
            Door | Empty | StairE | StairN | StairS | StairW,
            StairW,
            Door | Empty | StairE | StairS,
            StairN,
        ) => TOP_EMPTY_SOUTHSTAIRN_EASTSTAIRW,
        (
            Door | Empty | StairN | StairW,
            StairW,
            Door | Empty | StairE | StairN | StairS,
            StairE,
        ) => TOP_EMPTY_SOUTHSTAIRE_EASTSTAIRW,
        (
            Door | Empty | StairE | StairN,
            StairW,
            Door | Empty | StairE | StairN | StairS,
            StairW,
        ) => TOP_EMPTY_SOUTHSTAIRW_EASTSTAIRW,
        (Door | Empty | StairE | StairN | StairW, StairW, StairW | Wall, Door | Empty | StairS) => {
            TOP_EMPTY_EASTSTAIRWFULL
        }
        (Empty | StairE | StairN | StairW, StairW, StairW | Wall, Wall) => {
            TOP_EMPTY_SOUTHWALLHORIZONTAL_EASTSTAIRWFULL
        }
        (Door | Empty | StairE | StairN | StairS | StairW, StairW, StairW | Wall, StairN) => {
            TOP_EMPTY_SOUTHSTAIRNFULL_EASTSTAIRWFULL
        }
        (Door | Empty | StairN | StairW, StairW, StairW | Wall, StairE) => {
            TOP_EMPTY_SOUTHSTAIRE_EASTSTAIRWFULL
        }
        (Door | Empty | StairE | StairN, StairW, StairW | Wall, StairW) => {
            TOP_EMPTY_SOUTHSTAIRW_EASTSTAIRWFULL
        }
        (Empty | StairE | StairN | StairW, StairW, StairN, Wall) => {
            TOP_EMPTY_SOUTHWALLCORNER_SOUTHEASTSTAIRN_EASTSTAIRW
        }
        (Door | Empty | StairE | StairN | StairS | StairW, StairW, StairN, StairN) => {
            TOP_EMPTY_SOUTHSTAIRNFULL_SOUTHEASTSTAIRN_EASTSTAIRW
        }
        (
            Wall,
            Empty | StairE | StairN | StairS,
            Door | Empty | StairE | StairN | StairS | Wall,
            Empty | StairS,
        )
        | (Wall, Empty | StairE | StairN | StairS, Wall, Door) => TOP_WALLCORNER,
        (Wall, Empty | StairE | StairN | StairS, Door | Empty | StairE | StairS, Wall) => {
            TOP_WALLVERTICAL_SOUTHWALLCORNER
        }
        (Wall, Empty | StairE | StairN | StairS, Door | Empty | StairE | StairN | StairS, Door) => {
            TOP_WALLCORNER_SOUTHDOOR
        }
        (Wall, Empty | StairE | StairN | StairS, Door | Empty | StairE | StairS, StairN) => {
            TOP_WALLVERTICAL_SOUTHSTAIRN
        }
        (
            Wall,
            Empty | StairE | StairN | StairS,
            Door | Empty | StairE | StairN | StairS | Wall,
            StairE,
        ) => TOP_WALLCORNER_SOUTHSTAIREFULL,
        (
            Wall,
            Empty | StairE | StairN | StairS,
            Door | Empty | StairE | StairN | StairS | Wall,
            StairW,
        ) => TOP_WALLCORNER_SOUTHSTAIRWFULL,
        (Wall, Empty | StairE | StairN | StairS, StairW | Wall, Wall) => {
            TOP_WALLVERTICAL_SOUTHWALLHORIZONTAL
        }
        (Wall, Empty | StairE | StairN | StairS, StairN | Wall, StairN) => {
            TOP_WALLVERTICAL_SOUTHSTAIRNFULL
        }
        (Wall, Empty | StairE | StairN | StairS, StairN, Wall) => {
            TOP_WALLVERTICAL_SOUTHWALLCORNER_SOUTHEASTSTAIRN
        }
        (Wall, Empty | StairE | StairN | StairS, StairW, Empty | StairS) => {
            TOP_WALLCORNER_SOUTHEASTSTAIRW
        }
        (Wall, Empty | StairE | StairN | StairS, StairW, Door) => {
            TOP_WALLCORNER_SOUTHDOOR_SOUTHEASTSTAIRW
        }
        (Wall, Empty | StairE | StairN | StairS, StairW, StairN) => {
            TOP_WALLVERTICAL_SOUTHSTAIRN_SOUTHEASTSTAIRW
        }
        (Wall, Empty | StairE | StairN | StairS, StairW, StairE) => {
            TOP_WALLCORNER_SOUTHSTAIREFULL_SOUTHEASTSTAIRW
        }
        (Wall, Empty | StairE | StairN | StairS, StairW, StairW) => {
            TOP_WALLCORNER_SOUTHSTAIRWFULL_SOUTHEASTSTAIRW
        }
        (Wall, StairW | Wall, Door | Empty | StairE | StairN | StairS, Empty | StairS)
        | (Wall, Wall, Wall, Door | Empty | StairS) => TOP_WALLHORIZONTAL,
        (Wall, StairW | Wall, Door | Empty | StairE | StairS, Wall) => {
            TOP_WALLINVERSECORNER_SOUTHWALLCORNER
        }
        (Wall, StairW | Wall, Door | Empty | StairE | StairN | StairS, Door) => {
            TOP_WALLHORIZONTAL_SOUTHDOOR
        }
        (Wall, StairW | Wall, Door | Empty | StairE | StairS, StairN) => {
            TOP_WALLINVERSECORNER_SOUTHSTAIRN
        }
        (Wall, StairW | Wall, Door | Empty | StairE | StairN | StairS, StairE)
        | (Wall, Wall, Wall, StairE) => TOP_WALLHORIZONTAL_SOUTHSTAIREFULL,
        (Wall, StairW | Wall, Door | Empty | StairE | StairN | StairS, StairW)
        | (Wall, Wall, Wall, StairW) => TOP_WALLHORIZONTAL_SOUTHSTAIRWFULL,
        (Wall, Wall, Wall, Wall) => TOP_WALLFULL,
        (Wall, Wall, Wall, StairN) | (Wall, StairW | Wall, StairN, StairN) => {
            TOP_WALLFULL_SOUTHSTAIRNFULL
        }
        (Wall, StairW | Wall, StairN, Wall) => {
            TOP_WALLINVERSECORNER_SOUTHWALLCORNER_SOUTHEASTSTAIRN
        }
        (Wall, StairW | Wall, StairW, Empty | StairS)
        | (Wall, StairW, Wall, Door | Empty | StairS) => TOP_WALLHORIZONTAL_SOUTHEASTSTAIRWFULL,
        (Wall, StairW | Wall, StairW, Wall) | (Wall, StairW, Wall, Wall) => {
            TOP_WALLFULL_SOUTHWALLHORIZONTAL
        }
        (Wall, StairW | Wall, StairW, Door) => TOP_WALLHORIZONTAL_SOUTHDOOR_SOUTHEASTSTAIRWFULL,
        (Wall, StairW | Wall, StairW, StairN) | (Wall, StairW, Wall, StairN) => {
            TOP_WALLINVERSECORNER_SOUTHSTAIRN_SOUTHEASTSTAIRWFULL
        }
        (Wall, StairW | Wall, StairW, StairE) | (Wall, StairW, Wall, StairE) => {
            TOP_WALLHORIZONTAL_SOUTHSTAIREFULL_SOUTHEASTSTAIRWFULL
        }
        (Wall, StairW | Wall, StairW, StairW) | (Wall, StairW, Wall, StairW) => {
            TOP_WALLHORIZONTAL_SOUTHSTAIRWFULL_SOUTHEASTSTAIRWFULL
        }
        (Wall, Door, Door | Empty | StairE | StairN | StairS | Wall, Empty | StairS)
        | (Wall, Door, Wall, Door) => TOP_WALLCORNER_EASTDOOR,
        (Wall, Door, Door | Empty | StairE | StairS, Wall) => {
            TOP_WALLVERTICAL_SOUTHWALLCORNER_EASTDOOR
        }
        (Wall, Door, Door | Empty | StairE | StairN | StairS, Door) => {
            TOP_WALLCORNER_SOUTHDOOR_EASTDOOR
        }
        (Wall, Door, Door | Empty | StairE | StairS, StairN) => {
            TOP_WALLVERTICAL_SOUTHSTAIRN_EASTDOOR
        }
        (Wall, Door, Door | Empty | StairE | StairN | StairS | Wall, StairE) => {
            TOP_WALLCORNER_SOUTHSTAIREFULL_EASTDOOR
        }
        (Wall, Door, Door | Empty | StairE | StairN | StairS | Wall, StairW) => {
            TOP_WALLCORNER_SOUTHSTAIRWFULL_EASTDOOR
        }
        (Wall, Door, StairW | Wall, Wall) => TOP_WALLVERTICAL_SOUTHWALLHORIZONTAL_EASTDOOR,
        (Wall, Door, StairN | Wall, StairN) => TOP_WALLVERTICAL_SOUTHSTAIRNFULL_EASTDOOR,
        (Wall, Door, StairN, Wall) => TOP_WALLVERTICAL_SOUTHWALLCORNER_SOUTHEASTSTAIRN_EASTDOOR,
        (Wall, Door, StairW, Empty | StairS) => TOP_WALLCORNER_SOUTHEASTSTAIRW_EASTDOOR,
        (Wall, Door, StairW, Door) => TOP_WALLCORNER_SOUTHDOOR_SOUTHEASTSTAIRW_EASTDOOR,
        (Wall, Door, StairW, StairN) => TOP_WALLVERTICAL_SOUTHSTAIRN_SOUTHEASTSTAIRW_EASTDOOR,
        (Wall, Door, StairW, StairE) => TOP_WALLCORNER_SOUTHSTAIREFULL_SOUTHEASTSTAIRW_EASTDOOR,
        (Wall, Door, StairW, StairW) => TOP_WALLCORNER_SOUTHSTAIRWFULL_SOUTHEASTSTAIRW_EASTDOOR,
        (Door, Door | Empty | StairE | StairN | StairS, Door | Empty | StairE | StairS, Wall) => {
            TOP_DOOR_SOUTHWALLCORNER
        }
        (Door, Door | Empty | StairE | StairN | StairS, StairW | Wall, Wall) => {
            TOP_DOOR_SOUTHWALLHORIZONTAL
        }
        (Door, Door | Empty | StairE | StairN | StairS, StairN, Wall) => {
            TOP_DOOR_SOUTHWALLCORNER_SOUTHEASTSTAIRN
        }
        (Door, StairW, Door | Empty | StairE | StairS, Wall) => TOP_DOOR_SOUTHWALLCORNER_EASTSTAIRW,
        (Door, StairW, StairW | Wall, Wall) => TOP_DOOR_SOUTHWALLHORIZONTAL_EASTSTAIRWFULL,
        (Door, StairW, StairN, Wall) => TOP_DOOR_SOUTHWALLCORNER_SOUTHEASTSTAIRN_EASTSTAIRW,
        (
            StairS,
            Door | Empty | StairE | StairN,
            Door | Empty | StairE | StairN | StairS | Wall,
            Door | Empty | StairS,
        ) => TOP_STAIRS,
        (StairS, Door | Empty | StairE | StairN, Door | Empty | StairE | StairS, Wall) => {
            TOP_STAIRS_SOUTHWALLCORNER
        }
        (
            StairS,
            Door | Empty | StairE | StairN,
            Door | Empty | StairE | StairN | StairS | Wall,
            StairE,
        ) => TOP_STAIRS_SOUTHSTAIRE,
        (
            StairS,
            Door | Empty | StairE | StairN,
            Door | Empty | StairE | StairN | StairS | Wall,
            StairW,
        ) => TOP_STAIRS_SOUTHSTAIRW,
        (StairS, Door | Empty | StairE | StairN, StairW | Wall, Wall) => {
            TOP_STAIRS_SOUTHWALLHORIZONTAL
        }
        (StairS, Door | Empty | StairE | StairN, StairN, Wall) => {
            TOP_STAIRS_SOUTHWALLCORNER_SOUTHEASTSTAIRN
        }
        (StairS, Door | Empty | StairE | StairN, StairW, Door | Empty | StairS) => {
            TOP_STAIRS_SOUTHEASTSTAIRW
        }
        (StairS, Door | Empty | StairE | StairN, StairW, StairE) => {
            TOP_STAIRS_SOUTHSTAIRE_SOUTHEASTSTAIRW
        }
        (StairS, Door | Empty | StairE | StairN, StairW, StairW) => {
            TOP_STAIRS_SOUTHSTAIRW_SOUTHEASTSTAIRW
        }
        (
            StairS,
            StairS | Wall,
            Door | Empty | StairE | StairN | StairS | Wall,
            Door | Empty | StairS,
        ) => TOP_STAIRSFULL,
        (StairS, StairS | Wall, Door | Empty | StairE | StairS, Wall) => {
            TOP_STAIRSFULL_SOUTHWALLCORNER
        }
        (StairS, StairS | Wall, Door | Empty | StairE | StairS, StairN) => {
            TOP_STAIRSFULL_SOUTHSTAIRN
        }
        (StairS, StairS | Wall, Door | Empty | StairE | StairN | StairS | Wall, StairE) => {
            TOP_STAIRSFULL_SOUTHSTAIRE
        }
        (StairS, StairS | Wall, Door | Empty | StairE | StairN | StairS | Wall, StairW) => {
            TOP_STAIRSFULL_SOUTHSTAIRW
        }
        (StairS, StairS | Wall, Wall, Wall) | (StairS, StairS, StairW, Wall) => {
            TOP_STAIRSFULL_SOUTHWALLHORIZONTAL
        }
        (StairS, StairS | Wall, StairN, Wall) => TOP_STAIRSFULL_SOUTHWALLCORNER_SOUTHEASTSTAIRN,
        (StairS, Wall, StairW, Door | Empty | StairS) => TOP_STAIRSFULL_SOUTHEASTSTAIRWFULL,
        (StairS, Wall, StairW, Wall) => TOP_STAIRSFULL_SOUTHWALLHORIZONTAL_SOUTHEASTSTAIRWFULL,
        (StairS, Wall, StairW, StairE) => TOP_STAIRSFULL_SOUTHSTAIRE_SOUTHEASTSTAIRWFULL,
        (StairS, Wall, StairW, StairW) => TOP_STAIRSFULL_SOUTHSTAIRW_SOUTHEASTSTAIRWFULL,
        (StairS, StairS, StairW, Door | Empty | StairS) => TOP_STAIRSFULL_SOUTHEASTSTAIRW,
        (StairS, StairS, StairW, StairN) => TOP_STAIRSFULL_SOUTHSTAIRN_SOUTHEASTSTAIRW,
        (StairS, StairS, StairW, StairE) => TOP_STAIRSFULL_SOUTHSTAIRE_SOUTHEASTSTAIRW,
        (StairS, StairS, StairW, StairW) => TOP_STAIRSFULL_SOUTHSTAIRW_SOUTHEASTSTAIRW,
        (StairS, StairW, Door | Empty | StairE | StairN | StairS, Door | Empty | StairS) => {
            TOP_STAIRS_EASTSTAIRW
        }
        (StairS, StairW, Door | Empty | StairE | StairS, Wall) => {
            TOP_STAIRS_SOUTHWALLCORNER_EASTSTAIRW
        }
        (StairS, StairW, Door | Empty | StairE | StairN | StairS, StairE) => {
            TOP_STAIRS_SOUTHSTAIRE_EASTSTAIRW
        }
        (StairS, StairW, Door | Empty | StairE | StairN | StairS, StairW) => {
            TOP_STAIRS_SOUTHSTAIRW_EASTSTAIRW
        }
        (StairS, StairW, StairW | Wall, Door | Empty | StairS) => TOP_STAIRS_EASTSTAIRWFULL,
        (StairS, StairW, StairW | Wall, Wall) => TOP_STAIRS_SOUTHWALLHORIZONTAL_EASTSTAIRWFULL,
        (StairS, StairW, StairW | Wall, StairE) => TOP_STAIRS_SOUTHSTAIRE_EASTSTAIRWFULL,
        (StairS, StairW, StairW | Wall, StairW) => TOP_STAIRS_SOUTHSTAIRW_EASTSTAIRWFULL,
        (StairS, StairW, StairN, Wall) => TOP_STAIRS_SOUTHWALLCORNER_SOUTHEASTSTAIRN_EASTSTAIRW,
        (
            StairE,
            Door | Empty | StairE | StairN | StairS | Wall,
            Door | Empty | StairE | StairN | StairS | Wall,
            StairE,
        ) => TOP_EMPTY_SOUTHSTAIREFULL,
        (StairE, Door | Empty | StairE | StairN | StairS, StairW, StairE) => {
            TOP_EMPTY_SOUTHSTAIREFULL_SOUTHEASTSTAIRW
        }
        (StairE, Wall, StairW, StairE) => TOP_EMPTY_SOUTHSTAIREFULL_SOUTHEASTSTAIRWFULL,
        (StairE, StairW, Door | Empty | StairE | StairN | StairS, StairE) => {
            TOP_EMPTY_SOUTHSTAIREFULL_EASTSTAIRW
        }
        (StairE, StairW, StairW | Wall, StairE) => TOP_EMPTY_SOUTHSTAIREFULL_EASTSTAIRWFULL,
        (
            StairW,
            Door | Empty | StairE | StairN | StairS | Wall,
            Door | Empty | StairE | StairN | StairS | Wall,
            StairW,
        ) => TOP_EMPTY_SOUTHSTAIRWFULL,
        (StairW, Door | Empty | StairE | StairN | StairS, StairW, StairW) => {
            TOP_EMPTY_SOUTHSTAIRWFULL_SOUTHEASTSTAIRW
        }
        (StairW, Wall, StairW, StairW) => TOP_EMPTY_SOUTHSTAIRWFULL_SOUTHEASTSTAIRWFULL,
        (StairW, StairW, Door | Empty | StairE | StairN | StairS, StairW) => {
            TOP_EMPTY_SOUTHSTAIRWFULL_EASTSTAIRW
        }
        (StairW, StairW, StairW | Wall, StairW) => TOP_EMPTY_SOUTHSTAIRWFULL_EASTSTAIRWFULL,
    }
}
