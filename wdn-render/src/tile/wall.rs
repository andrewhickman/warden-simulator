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
const BASE_WALLCORNER_EASTSTAIRNORTH: u16 = 11;
const BASE_WALLVERTICAL_EASTSTAIRNORTH: u16 = 12;
const BASE_WALLCORNER_EASTSTAIRNORTH_SOUTHDOOR: u16 = 13;
const BASE_WALLCORNER_EASTSTAIRSOUTH: u16 = 14;
const BASE_WALLVERTICAL_EASTSTAIRSOUTH: u16 = 15;

const BASE_WALLCORNER_EASTSTAIRSOUTH_SOUTHDOOR: u16 = 16;
const BASE_STAIRNORTHCORNER: u16 = 17;
const BASE_STAIRNORTHHORIZONTAL: u16 = 18;
const BASE_STAIRSOUTHCORNER: u16 = 19;
const BASE_STAIRSOUTHVERTICAL: u16 = 20;
const BASE_STAIRSOUTHCORNER_SOUTHDOOR: u16 = 21;
const BASE_STAIRSOUTHHORIZONTAL: u16 = 22;
const BASE_STAIRSOUTHINVERSECORNER: u16 = 23;
const BASE_STAIRSOUTHHORIZONTAL_SOUTHDOOR: u16 = 24;
const BASE_STAIRSOUTHFULL: u16 = 25;
const BASE_STAIREASTCORNER: u16 = 26;
const BASE_STAIREASTVERTICAL: u16 = 27;
const BASE_STAIREASTHORIZONTAL: u16 = 28;
const BASE_STAIREASTINVERSECORNER: u16 = 29;
const BASE_STAIREASTFULL: u16 = 30;
const BASE_STAIREASTCORNER_EASTDOOR: u16 = 31;

const BASE_STAIREASTVERTICAL_EASTDOOR: u16 = 32;
const BASE_STAIREASTCORNER_EASTSTAIRNORTH: u16 = 33;
const BASE_STAIREASTVERTICAL_EASTSTAIRNORTH: u16 = 34;
const BASE_STAIREASTCORNER_EASTSTAIRSOUTH: u16 = 35;
const BASE_STAIREASTVERTICAL_EASTSTAIRSOUTH: u16 = 36;
const BASE_STAIRWESTCORNER: u16 = 37;
const BASE_STAIRWESTVERTICAL: u16 = 38;
const TOP_WALLCORNER: u16 = 39;
const TOP_STAIRNORTHCORNER: u16 = 40;
const TOP_STAIREASTCORNER: u16 = 41;
const TOP_STAIRWESTCORNER: u16 = 42;
const TOP_WALLHORIZONTAL: u16 = 43;
const TOP_STAIRNORTHHORIZONTAL: u16 = 44;
const TOP_STAIREASTHORIZONTAL: u16 = 45;
const TOP_WALLCORNER_EASTSTAIRNORTH: u16 = 46;
const TOP_STAIREASTCORNER_EASTSTAIRNORTH: u16 = 47;

const TOP_STAIRNORTHVERTICAL: u16 = 48;
const TOP_STAIREASTVERTICAL_OVERWALLCORNER: u16 = 49;
const TOP_STAIRWESTVERTICAL: u16 = 50;
const TOP_WALLHORIZONTAL_OVERWALLVERTICAL: u16 = 51;
const TOP_STAIRNORTHFULL_OVERWALLVERTICAL: u16 = 52;
const TOP_STAIREASTFULL_OVERWALLCORNER: u16 = 53;
const TOP_EMPTY_EASTSTAIRNORTH_OVERWALLVERTICAL: u16 = 54;
const TOP_STAIREASTVERTICAL_EASTSTAIRNORTH_OVERWALLCORNER: u16 = 55;
const TOP_WALLHORIZONTAL_OVERWALLFULL: u16 = 56;
const TOP_STAIRNORTHFULL_OVERWALLFULL: u16 = 57;
const TOP_STAIREASTFULL_OVERWALLHORIZONTAL: u16 = 58;
const TOP_EMPTY_EASTSTAIRNORTH_OVERWALLFULL: u16 = 59;
const TOP_STAIREASTVERTICAL_EASTSTAIRNORTH_OVERWALLHORIZONTAL: u16 = 60;
const TOP_WALLHORIZONTAL_OVERWALLVERTICALEASTDOOR: u16 = 61;
const TOP_WALLHORIZONTAL_OVERWALLVERTICALEASTSTAIRNORTH: u16 = 62;
const TOP_WALLHORIZONTAL_OVERWALLVERTICALEASTSTAIRSOUTH: u16 = 63;

const TOP_STAIRNORTHFULL_OVERWALLVERTICALEASTSTAIRSOUTH: u16 = 64;
const TOP_EMPTY_EASTSTAIRNORTH_OVERWALLVERTICALEASTSTAIRSOUTH: u16 = 65;
const TOP_WALLCORNER_NORTHDOOR: u16 = 66;
const TOP_STAIRNORTHCORNER_NORTHDOOR: u16 = 67;
const TOP_WALLHORIZONTAL_NORTHDOOR: u16 = 68;
const TOP_STAIRNORTHHORIZONTAL_NORTHDOOR: u16 = 69;
const TOP_WALLCORNER_EASTSTAIRNORTH_NORTHDOOR: u16 = 70;
const TOP_WALLCORNER_OVERSTAIRSOUTHVERTICAL: u16 = 71;
const TOP_WALLHORIZONTAL_OVERSTAIRSOUTHVERTICAL: u16 = 72;
const TOP_WALLCORNER_EASTSTAIRNORTH_OVERSTAIRSOUTHVERTICAL: u16 = 73;
const TOP_WALLHORIZONTAL_OVERSTAIRSOUTHFULL: u16 = 74;
const TOP_STAIRNORTHFULL_OVERSTAIRSOUTHFULL: u16 = 75;
const TOP_WALLCORNER_EASTSTAIRNORTH_OVERSTAIRSOUTHFULL: u16 = 76;
const TOP_WALLCORNER_OVERSTAIREASTVERTICAL: u16 = 77;
const TOP_STAIRNORTHCORNER_OVERSTAIREASTVERTICAL: u16 = 78;
const TOP_WALLHORIZONTAL_OVERSTAIREASTVERTICAL: u16 = 79;

const TOP_STAIRNORTHHORIZONTAL_OVERSTAIREASTVERTICAL: u16 = 80;
const TOP_WALLCORNER_EASTSTAIRNORTH_OVERSTAIREASTVERTICAL: u16 = 81;
const TOP_WALLHORIZONTAL_OVERSTAIREASTFULL: u16 = 82;
const TOP_STAIRNORTHHORIZONTAL_OVERSTAIREASTFULL: u16 = 83;
const TOP_WALLCORNER_EASTSTAIRNORTH_OVERSTAIREASTFULL: u16 = 84;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum TileVariant {
    Empty,
    Wall,
    DoorH,
    DoorV,
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
        TileKind::Door => match material
            .flags()
            .intersection(TileMaterialFlags::DOOR_DIRECTION_MASK)
        {
            TileMaterialFlags::DOOR_DIRECTION_HORIZONTAL => TileVariant::DoorH,
            TileMaterialFlags::DOOR_DIRECTION_VERTICAL => TileVariant::DoorV,
            _ => unreachable!("Invalid door direction for tile at {tile:?}"),
        },
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
        (DoorH | DoorV | Empty, _, _, _) => EMPTY,
        (Wall, DoorV | Empty | StairE, _, DoorH | Empty | StairE | StairS | StairW) => {
            BASE_WALLCORNER
        }
        (Wall, DoorV | Empty | StairE, _, StairN | Wall) => BASE_WALLVERTICAL,
        (Wall, DoorV | Empty | StairE, _, DoorV) => BASE_WALLCORNER_SOUTHDOOR,
        (Wall, StairW | Wall, _, DoorH | Empty | StairE | StairS | StairW) => BASE_WALLHORIZONTAL,
        (Wall, StairW | Wall, DoorH | DoorV | Empty | StairE | StairS, StairN | Wall) => {
            BASE_WALLINVERSECORNER
        }
        (Wall, StairW | Wall, _, DoorV) => BASE_WALLHORIZONTAL_SOUTHDOOR,
        (Wall, StairW | Wall, StairN | StairW | Wall, StairN | Wall) => BASE_WALLFULL,
        (Wall, DoorH, _, DoorH | Empty | StairE | StairS | StairW) => BASE_WALLCORNER_EASTDOOR,
        (Wall, DoorH, _, StairN | Wall) => BASE_WALLVERTICAL_EASTDOOR,
        (Wall, DoorH, _, DoorV) => BASE_WALLCORNER_EASTDOOR_SOUTHDOOR,
        (Wall, StairN, _, DoorH | Empty | StairE | StairS | StairW) => {
            BASE_WALLCORNER_EASTSTAIRNORTH
        }
        (Wall, StairN, _, StairN | Wall) => BASE_WALLVERTICAL_EASTSTAIRNORTH,
        (Wall, StairN, _, DoorV) => BASE_WALLCORNER_EASTSTAIRNORTH_SOUTHDOOR,
        (Wall, StairS, _, DoorH | Empty | StairE | StairS | StairW) => {
            BASE_WALLCORNER_EASTSTAIRSOUTH
        }
        (Wall, StairS, _, StairN | Wall) => BASE_WALLVERTICAL_EASTSTAIRSOUTH,
        (Wall, StairS, _, DoorV) => BASE_WALLCORNER_EASTSTAIRSOUTH_SOUTHDOOR,
        (StairN, DoorH | DoorV | Empty | StairE | StairS, _, _) => BASE_STAIRNORTHCORNER,
        (StairN, StairN | StairW | Wall, _, _) => BASE_STAIRNORTHHORIZONTAL,
        (
            StairS,
            DoorH | DoorV | Empty | StairE | StairN,
            _,
            DoorH | Empty | StairE | StairS | StairW,
        ) => BASE_STAIRSOUTHCORNER,
        (StairS, DoorH | DoorV | Empty | StairE | StairN, _, StairN | Wall) => {
            BASE_STAIRSOUTHVERTICAL
        }
        (StairS, DoorH | DoorV | Empty | StairE | StairN, _, DoorV) => {
            BASE_STAIRSOUTHCORNER_SOUTHDOOR
        }
        (StairS, StairS | StairW | Wall, _, DoorH | Empty | StairE | StairS | StairW) => {
            BASE_STAIRSOUTHHORIZONTAL
        }
        (
            StairS,
            StairS | StairW | Wall,
            DoorH | DoorV | Empty | StairE | StairS,
            StairN | Wall,
        ) => BASE_STAIRSOUTHINVERSECORNER,
        (StairS, StairS | StairW | Wall, _, DoorV) => BASE_STAIRSOUTHHORIZONTAL_SOUTHDOOR,
        (StairS, StairS | StairW | Wall, StairN | StairW | Wall, StairN | Wall) => {
            BASE_STAIRSOUTHFULL
        }
        (StairE, DoorV | Empty | StairE, _, DoorH | DoorV | Empty | StairS | StairW) => {
            BASE_STAIREASTCORNER
        }
        (StairE, DoorV | Empty | StairE, _, StairE | StairN | Wall) => BASE_STAIREASTVERTICAL,
        (StairE, StairW | Wall, _, DoorH | DoorV | Empty | StairS | StairW) => {
            BASE_STAIREASTHORIZONTAL
        }
        (
            StairE,
            StairW | Wall,
            DoorH | DoorV | Empty | StairE | StairS,
            StairE | StairN | Wall,
        ) => BASE_STAIREASTINVERSECORNER,
        (StairE, StairW | Wall, StairN | StairW | Wall, StairE | StairN | Wall) => {
            BASE_STAIREASTFULL
        }
        (StairE, DoorH, _, DoorH | DoorV | Empty | StairS | StairW) => {
            BASE_STAIREASTCORNER_EASTDOOR
        }
        (StairE, DoorH, _, StairE | StairN | Wall) => BASE_STAIREASTVERTICAL_EASTDOOR,
        (StairE, StairN, _, DoorH | DoorV | Empty | StairS | StairW) => {
            BASE_STAIREASTCORNER_EASTSTAIRNORTH
        }
        (StairE, StairN, _, StairE | StairN | Wall) => BASE_STAIREASTVERTICAL_EASTSTAIRNORTH,
        (StairE, StairS, _, DoorH | DoorV | Empty | StairS | StairW) => {
            BASE_STAIREASTCORNER_EASTSTAIRSOUTH
        }
        (StairE, StairS, _, StairE | StairN | Wall) => BASE_STAIREASTVERTICAL_EASTSTAIRSOUTH,
        (StairW, _, _, DoorH | DoorV | Empty | StairE | StairS) => BASE_STAIRWESTCORNER,
        (StairW, _, _, StairN | StairW | Wall) => BASE_STAIRWESTVERTICAL,
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
        (_, _, _, DoorH | DoorV | Empty | StairS)
        | (Wall, _, DoorH | DoorV | Empty | StairE | StairS, Wall)
        | (StairE, _, DoorH | DoorV | Empty | StairE | StairS | StairW | Wall, StairE)
        | (StairW, _, _, StairW) => EMPTY,
        (DoorH | Empty | StairN | StairW, _, DoorH | DoorV | Empty | StairE | StairS, Wall) => {
            TOP_WALLCORNER
        }
        (DoorH | Empty | StairN | StairW, _, DoorH | DoorV | Empty | StairE | StairS, StairN) => {
            TOP_STAIRNORTHCORNER
        }
        (
            DoorH | DoorV | Empty | StairN | StairW,
            _,
            DoorH | DoorV | Empty | StairE | StairS,
            StairE,
        ) => TOP_STAIREASTCORNER,
        (DoorH | DoorV | Empty | StairE | StairN, _, _, StairW) => TOP_STAIRWESTCORNER,
        (DoorH | Empty | StairN | StairW, _, StairW | Wall, Wall) => TOP_WALLHORIZONTAL,
        (DoorH | Empty | StairN | StairW, _, StairN | StairW | Wall, StairN) => {
            TOP_STAIRNORTHHORIZONTAL
        }
        (DoorH | DoorV | Empty | StairN | StairW, _, StairW | Wall, StairE) => {
            TOP_STAIREASTHORIZONTAL
        }
        (DoorH | Empty | StairN | StairW, _, StairN, Wall) => TOP_WALLCORNER_EASTSTAIRNORTH,
        (DoorH | DoorV | Empty | StairN | StairW, _, StairN, StairE) => {
            TOP_STAIREASTCORNER_EASTSTAIRNORTH
        }
        (StairS | Wall, _, DoorH | DoorV | Empty | StairE | StairS, StairN) => {
            TOP_STAIRNORTHVERTICAL
        }
        (StairS | Wall, _, DoorH | DoorV | Empty | StairE | StairS, StairE) => {
            TOP_STAIREASTVERTICAL_OVERWALLCORNER
        }
        (StairS | Wall, _, _, StairW) => TOP_STAIRWESTVERTICAL,
        (Wall, DoorV | Empty | StairE, StairW | Wall, Wall) => TOP_WALLHORIZONTAL_OVERWALLVERTICAL,
        (
            StairS | Wall,
            DoorH | DoorV | Empty | StairE | StairN,
            StairN | StairW | Wall,
            StairN,
        ) => TOP_STAIRNORTHFULL_OVERWALLVERTICAL,
        (StairS | Wall, DoorH | DoorV | Empty | StairE | StairN, StairW | Wall, StairE) => {
            TOP_STAIREASTFULL_OVERWALLCORNER
        }
        (Wall, DoorH | DoorV | Empty | StairE | StairN, StairN, Wall)
        | (StairE, DoorH | DoorV | Empty | StairE | StairN | StairS, StairN, StairE) => {
            TOP_EMPTY_EASTSTAIRNORTH_OVERWALLVERTICAL
        }
        (StairS | Wall, DoorH | DoorV | Empty | StairE | StairN, StairN, StairE) => {
            TOP_STAIREASTVERTICAL_EASTSTAIRNORTH_OVERWALLCORNER
        }
        (Wall, StairW | Wall, StairW | Wall, Wall) => TOP_WALLHORIZONTAL_OVERWALLFULL,
        (Wall, StairW | Wall, StairN | StairW | Wall, StairN) => TOP_STAIRNORTHFULL_OVERWALLFULL,
        (StairS | Wall, StairS | StairW | Wall, StairW | Wall, StairE) => {
            TOP_STAIREASTFULL_OVERWALLHORIZONTAL
        }
        (Wall, StairW | Wall, StairN, Wall) | (StairE, StairW | Wall, StairN, StairE) => {
            TOP_EMPTY_EASTSTAIRNORTH_OVERWALLFULL
        }
        (StairS | Wall, StairS | StairW | Wall, StairN, StairE) => {
            TOP_STAIREASTVERTICAL_EASTSTAIRNORTH_OVERWALLHORIZONTAL
        }
        (Wall, DoorH, StairW | Wall, Wall) => TOP_WALLHORIZONTAL_OVERWALLVERTICALEASTDOOR,
        (Wall, StairN, StairW | Wall, Wall) => TOP_WALLHORIZONTAL_OVERWALLVERTICALEASTSTAIRNORTH,
        (Wall, StairS, StairW | Wall, Wall) => TOP_WALLHORIZONTAL_OVERWALLVERTICALEASTSTAIRSOUTH,
        (Wall, StairS, StairN | StairW | Wall, StairN) => {
            TOP_STAIRNORTHFULL_OVERWALLVERTICALEASTSTAIRSOUTH
        }
        (Wall, StairS, StairN, Wall) => TOP_EMPTY_EASTSTAIRNORTH_OVERWALLVERTICALEASTSTAIRSOUTH,
        (DoorV, _, DoorH | DoorV | Empty | StairE | StairS, Wall) => TOP_WALLCORNER_NORTHDOOR,
        (DoorV, _, DoorH | DoorV | Empty | StairE | StairS, StairN) => {
            TOP_STAIRNORTHCORNER_NORTHDOOR
        }
        (DoorV, _, StairW | Wall, Wall) => TOP_WALLHORIZONTAL_NORTHDOOR,
        (DoorV, _, StairN | StairW | Wall, StairN) => TOP_STAIRNORTHHORIZONTAL_NORTHDOOR,
        (DoorV, _, StairN, Wall) => TOP_WALLCORNER_EASTSTAIRNORTH_NORTHDOOR,
        (StairS, _, DoorH | DoorV | Empty | StairE | StairS, Wall) => {
            TOP_WALLCORNER_OVERSTAIRSOUTHVERTICAL
        }
        (StairS, DoorH | DoorV | Empty | StairE | StairN, StairW | Wall, Wall) => {
            TOP_WALLHORIZONTAL_OVERSTAIRSOUTHVERTICAL
        }
        (StairS, DoorH | DoorV | Empty | StairE | StairN, StairN, Wall) => {
            TOP_WALLCORNER_EASTSTAIRNORTH_OVERSTAIRSOUTHVERTICAL
        }
        (StairS, StairS | StairW | Wall, StairW | Wall, Wall) => {
            TOP_WALLHORIZONTAL_OVERSTAIRSOUTHFULL
        }
        (StairS, StairS | StairW | Wall, StairN | StairW | Wall, StairN) => {
            TOP_STAIRNORTHFULL_OVERSTAIRSOUTHFULL
        }
        (StairS, StairS | StairW | Wall, StairN, Wall) => {
            TOP_WALLCORNER_EASTSTAIRNORTH_OVERSTAIRSOUTHFULL
        }
        (StairE, _, DoorH | DoorV | Empty | StairE | StairS, Wall) => {
            TOP_WALLCORNER_OVERSTAIREASTVERTICAL
        }
        (StairE, _, DoorH | DoorV | Empty | StairE | StairS, StairN) => {
            TOP_STAIRNORTHCORNER_OVERSTAIREASTVERTICAL
        }
        (StairE, DoorH | DoorV | Empty | StairE | StairN | StairS, StairW | Wall, Wall) => {
            TOP_WALLHORIZONTAL_OVERSTAIREASTVERTICAL
        }
        (
            StairE,
            DoorH | DoorV | Empty | StairE | StairN | StairS,
            StairN | StairW | Wall,
            StairN,
        ) => TOP_STAIRNORTHHORIZONTAL_OVERSTAIREASTVERTICAL,
        (StairE, DoorH | DoorV | Empty | StairE | StairN | StairS, StairN, Wall) => {
            TOP_WALLCORNER_EASTSTAIRNORTH_OVERSTAIREASTVERTICAL
        }
        (StairE, StairW | Wall, StairW | Wall, Wall) => TOP_WALLHORIZONTAL_OVERSTAIREASTFULL,
        (StairE, StairW | Wall, StairN | StairW | Wall, StairN) => {
            TOP_STAIRNORTHHORIZONTAL_OVERSTAIREASTFULL
        }
        (StairE, StairW | Wall, StairN, Wall) => TOP_WALLCORNER_EASTSTAIRNORTH_OVERSTAIREASTFULL,
    }
}
