use wdn_physics::tile::{
    material::TileKind,
    position::{TileChunkOffset, TileChunkPosition, TilePosition},
    storage::{TileData, TileStorage},
};
use wdn_world::{door::DoorDirection, stair::Stair};

const EMPTY: u16 = 0;
const EMPTY_SOUTHWALLCORNER: u16 = 1;
const EMPTY_SOUTHWALLHORIZONTAL: u16 = 2;
const EMPTY_SOUTHSTAIRN: u16 = 3;
const EMPTY_SOUTHSTAIRNWALL: u16 = 4;
const EMPTY_SOUTHSTAIRNDOUBLE: u16 = 5;
const EMPTY_SOUTHSTAIRSWALL: u16 = 6;
const EMPTY_SOUTHSTAIRE: u16 = 7;
const EMPTY_SOUTHSTAIRW: u16 = 8;
const WALLCORNER: u16 = 9;
const WALLHORIZONTAL: u16 = 10;
const WALLCORNER_EASTDOOR: u16 = 11;
const WALLVERTICAL: u16 = 12;
const WALLINVERSECORNER: u16 = 13;
const WALLVERTICAL_EASTDOOR: u16 = 14;
const WALLVERTICAL_SOUTHEASTWALL: u16 = 15;

const WALLFULL: u16 = 16;
const WALLVERTICAL_SOUTHEASTWALL_EASTDOOR: u16 = 17;
const WALLCORNER_SOUTHDOOR: u16 = 18;
const WALLHORIZONTAL_SOUTHDOOR: u16 = 19;
const WALLCORNER_SOUTHDOOR_EASTDOOR: u16 = 20;
const WALLCORNER_SOUTHSTAIRN: u16 = 21;
const WALLHORIZONTAL_SOUTHSTAIRN: u16 = 22;
const WALLCORNER_SOUTHSTAIRN_EASTDOOR: u16 = 23;
const WALLCORNER_SOUTHSTAIRNWALL: u16 = 24;
const WALLHORIZONTAL_SOUTHSTAIRNWALL: u16 = 25;
const WALLCORNER_SOUTHSTAIRNWALL_EASTDOOR: u16 = 26;
const WALLCORNER_SOUTHSTAIRNDOUBLE: u16 = 27;
const WALLHORIZONTAL_SOUTHSTAIRNDOUBLE: u16 = 28;
const WALLCORNER_SOUTHSTAIRNDOUBLE_EASTDOOR: u16 = 29;
const WALLCORNER_SOUTHSTAIRSWALL: u16 = 30;
const WALLHORIZONTAL_SOUTHSTAIRSWALL: u16 = 31;

const WALLCORNER_SOUTHSTAIRSWALL_EASTDOOR: u16 = 32;
const WALLCORNER_SOUTHSTAIRE: u16 = 33;
const WALLHORIZONTAL_SOUTHSTAIRE: u16 = 34;
const WALLCORNER_SOUTHSTAIRE_EASTDOOR: u16 = 35;
const WALLCORNER_SOUTHSTAIRW: u16 = 36;
const WALLHORIZONTAL_SOUTHSTAIRW: u16 = 37;
const WALLCORNER_SOUTHSTAIRW_EASTDOOR: u16 = 38;
const DOOR_SOUTHWALLCORNER: u16 = 39;
const DOOR_SOUTHWALLHORIZONTAL: u16 = 40;
const STAIRN: u16 = 41;
const STAIRN_EASTWALL: u16 = 42;
const STAIRN_EASTSTAIR: u16 = 43;
const STAIRN_SOUTHWALLCORNER: u16 = 44;
const STAIRN_SOUTHWALLCORNER_EASTWALL: u16 = 45;
const STAIRN_SOUTHWALLCORNER_EASTSTAIR: u16 = 46;
const STAIRN_SOUTHWALLHORIZONTAL: u16 = 47;

const STAIRN_SOUTHWALLHORIZONTAL_EASTWALL: u16 = 48;
const STAIRN_SOUTHWALLHORIZONTAL_EASTSTAIR: u16 = 49;
const STAIRN_SOUTHSTAIRN: u16 = 50;
const STAIRN_SOUTHSTAIRN_EASTWALL: u16 = 51;
const STAIRN_SOUTHSTAIRN_EASTSTAIR: u16 = 52;
const STAIRN_SOUTHSTAIRNWALL: u16 = 53;
const STAIRN_SOUTHSTAIRNWALL_EASTWALL: u16 = 54;
const STAIRN_SOUTHSTAIRNWALL_EASTSTAIR: u16 = 55;
const STAIRN_SOUTHSTAIRNDOUBLE: u16 = 56;
const STAIRN_SOUTHSTAIRNDOUBLE_EASTWALL: u16 = 57;
const STAIRN_SOUTHSTAIRNDOUBLE_EASTSTAIR: u16 = 58;
const STAIRN_SOUTHSTAIRSWALL: u16 = 59;
const STAIRN_SOUTHSTAIRSWALL_EASTWALL: u16 = 60;
const STAIRN_SOUTHSTAIRSWALL_EASTSTAIR: u16 = 61;
const STAIRN_SOUTHSTAIRE: u16 = 62;
const STAIRN_SOUTHSTAIRE_EASTWALL: u16 = 63;

const STAIRN_SOUTHSTAIRE_EASTSTAIR: u16 = 64;
const STAIRN_SOUTHSTAIRW: u16 = 65;
const STAIRN_SOUTHSTAIRW_EASTWALL: u16 = 66;
const STAIRN_SOUTHSTAIRW_EASTSTAIR: u16 = 67;
const STAIRS: u16 = 68;
const STAIRS_EASTWALL: u16 = 69;
const STAIRS_EASTSTAIR: u16 = 70;
const STAIRS_SOUTHWALLCORNER: u16 = 71;
const STAIRS_SOUTHWALLCORNER_EASTWALL: u16 = 72;
const STAIRS_SOUTHWALLCORNER_EASTSTAIR: u16 = 73;
const STAIRS_SOUTHWALLHORIZONTAL: u16 = 74;
const STAIRS_SOUTHWALLHORIZONTAL_EASTWALL: u16 = 75;
const STAIRS_SOUTHWALLHORIZONTAL_EASTSTAIR: u16 = 76;
const STAIRS_SOUTHSTAIRN: u16 = 77;
const STAIRS_SOUTHSTAIRN_EASTWALL: u16 = 78;
const STAIRS_SOUTHSTAIRN_EASTSTAIR: u16 = 79;

const STAIRS_SOUTHSTAIRNWALL: u16 = 80;
const STAIRS_SOUTHSTAIRNWALL_EASTWALL: u16 = 81;
const STAIRS_SOUTHSTAIRNWALL_EASTSTAIR: u16 = 82;
const STAIRS_SOUTHSTAIRNDOUBLE: u16 = 83;
const STAIRS_SOUTHSTAIRNDOUBLE_EASTWALL: u16 = 84;
const STAIRS_SOUTHSTAIRNDOUBLE_EASTSTAIR: u16 = 85;
const STAIRS_SOUTHSTAIRSWALL: u16 = 86;
const STAIRS_SOUTHSTAIRSWALL_EASTWALL: u16 = 87;
const STAIRS_SOUTHSTAIRSWALL_EASTSTAIR: u16 = 88;
const STAIRS_SOUTHSTAIRE: u16 = 89;
const STAIRS_SOUTHSTAIRE_EASTWALL: u16 = 90;
const STAIRS_SOUTHSTAIRE_EASTSTAIR: u16 = 91;
const STAIRS_SOUTHSTAIRW: u16 = 92;
const STAIRS_SOUTHSTAIRW_EASTWALL: u16 = 93;
const STAIRS_SOUTHSTAIRW_EASTSTAIR: u16 = 94;
const STAIRE: u16 = 95;

const STAIRE_SOUTHWALLCORNER: u16 = 96;
const STAIRE_SOUTHWALLHORIZONTAL: u16 = 97;
const STAIRE_SOUTHWALLFULL: u16 = 98;
const STAIRE_SOUTHSTAIRN: u16 = 99;
const STAIRE_SOUTHSTAIRNWALL: u16 = 100;
const STAIRE_SOUTHSTAIRNWALLFULL: u16 = 101;
const STAIRE_SOUTHSTAIRNDOUBLE: u16 = 102;
const STAIRE_SOUTHSTAIRSWALL: u16 = 103;
const STAIRE_SOUTHSTAIRE: u16 = 104;
const STAIRE_SOUTHSTAIRW: u16 = 105;
const STAIRW: u16 = 106;
const STAIRW_NORTHWALL: u16 = 107;
const STAIRW_NORTHSTAIR: u16 = 108;
const STAIRW_SOUTHWALLCORNER: u16 = 109;
const STAIRW_SOUTHWALLCORNER_NORTHWALL: u16 = 110;
const STAIRW_SOUTHWALLCORNER_NORTHSTAIR: u16 = 111;

const STAIRW_SOUTHWALLHORIZONTAL: u16 = 112;
const STAIRW_SOUTHWALLHORIZONTAL_NORTHWALL: u16 = 113;
const STAIRW_SOUTHWALLHORIZONTAL_NORTHSTAIR: u16 = 114;
const STAIRW_SOUTHWALLFULL: u16 = 115;
const STAIRW_SOUTHWALLFULL_NORTHWALL: u16 = 116;
const STAIRW_SOUTHWALLFULL_NORTHSTAIR: u16 = 117;
const STAIRW_SOUTHSTAIRN: u16 = 118;
const STAIRW_SOUTHSTAIRN_NORTHWALL: u16 = 119;
const STAIRW_SOUTHSTAIRN_NORTHSTAIR: u16 = 120;
const STAIRW_SOUTHSTAIRNWALL: u16 = 121;
const STAIRW_SOUTHSTAIRNWALL_NORTHWALL: u16 = 122;
const STAIRW_SOUTHSTAIRNWALL_NORTHSTAIR: u16 = 123;
const STAIRW_SOUTHSTAIRNWALLFULL: u16 = 124;
const STAIRW_SOUTHSTAIRNWALLFULL_NORTHWALL: u16 = 125;
const STAIRW_SOUTHSTAIRNWALLFULL_NORTHSTAIR: u16 = 126;
const STAIRW_SOUTHSTAIRNDOUBLE: u16 = 127;

const STAIRW_SOUTHSTAIRNDOUBLE_NORTHWALL: u16 = 128;
const STAIRW_SOUTHSTAIRNDOUBLE_NORTHSTAIR: u16 = 129;
const STAIRW_SOUTHSTAIRSWALL: u16 = 130;
const STAIRW_SOUTHSTAIRSWALL_NORTHWALL: u16 = 131;
const STAIRW_SOUTHSTAIRSWALL_NORTHSTAIR: u16 = 132;
const STAIRW_SOUTHSTAIRE: u16 = 133;
const STAIRW_SOUTHSTAIRE_NORTHWALL: u16 = 134;
const STAIRW_SOUTHSTAIRE_NORTHSTAIR: u16 = 135;
const STAIRW_SOUTHSTAIRW: u16 = 136;
const STAIRW_SOUTHSTAIRW_NORTHWALL: u16 = 137;
const STAIRW_SOUTHSTAIRW_NORTHSTAIR: u16 = 138;

pub fn sprite_offset(
    storage: &TileStorage,
    position: TileChunkPosition,
    offset: TileChunkOffset,
    _tile: TileData,
) -> (u16, u16) {
    let position = TilePosition::from((position, offset));

    let center = get_tile_variant(storage, position);
    let south = get_tile_variant(storage, position.south());
    let north = get_tile_variant(storage, position.north());
    let east = get_tile_variant(storage, position.east());
    let south_east = get_tile_variant(storage, position.south().east());
    let west = get_tile_variant(storage, position.west());
    let south_west = get_tile_variant(storage, position.south().west());

    let right = tile_sprite(center, south, south_east, east, north);
    let left = tile_sprite(
        center.flip_x(),
        south.flip_x(),
        south_west.flip_x(),
        west.flip_x(),
        north.flip_x(),
    );

    (left, right)
}

fn get_tile_variant(storage: &TileStorage, position: TilePosition) -> TileVariant {
    let Some(data) = storage.get(position) else {
        return TileVariant::Empty;
    };

    match data.kind() {
        TileKind::Empty => TileVariant::Empty,
        TileKind::Wall => TileVariant::Wall,
        TileKind::Door => match DoorDirection::from_material_id(data.material().id()) {
            DoorDirection::Horizontal => TileVariant::DoorH,
            DoorDirection::Vertical => TileVariant::DoorV,
        },
        TileKind::Stairs => match Stair::from_material_id(data.material().id()) {
            Stair::North => TileVariant::StairN,
            Stair::South => TileVariant::StairS,
            Stair::East => TileVariant::StairE,
            Stair::West => TileVariant::StairW,
        },
    }
}

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

#[deny(clippy::match_same_arms)]
fn tile_sprite(
    center: TileVariant,
    south: TileVariant,
    south_east: TileVariant,
    east: TileVariant,
    north: TileVariant,
) -> u16 {
    use TileVariant::*;

    match (center, south, south_east, east, north) {
        (Empty | DoorH | DoorV, Empty | DoorH | DoorV, _, _, _)
        | (
            Empty | DoorH | DoorV,
            StairS,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            _,
            _,
        ) => EMPTY,
        (Empty | DoorH, Wall, Wall | StairN | StairE | StairS | StairW, _, _) => {
            EMPTY_SOUTHWALLHORIZONTAL
        }
        (Empty | DoorH, Wall, Empty | DoorH | DoorV, _, _) => EMPTY_SOUTHWALLCORNER,
        (Empty | DoorH | DoorV, StairN, Empty | DoorH | DoorV | StairE | StairS | StairW, _, _) => {
            EMPTY_SOUTHSTAIRN
        }
        (Empty | DoorH | DoorV, StairN, Wall, _, _) => EMPTY_SOUTHSTAIRNWALL,
        (Empty | DoorH | DoorV, StairN, StairN, _, _) => EMPTY_SOUTHSTAIRNDOUBLE,
        (Empty | DoorH | DoorV, StairE, _, _, _) => EMPTY_SOUTHSTAIRE,
        (Empty | DoorH | DoorV, StairW, _, _, _) => EMPTY_SOUTHSTAIRW,
        (Empty | DoorH | DoorV, StairS, Wall, _, _) => EMPTY_SOUTHSTAIRSWALL,
        (DoorV, Wall, Wall, _, _) | (DoorV, Wall, StairN | StairE | StairS | StairW, Wall, _) => {
            DOOR_SOUTHWALLHORIZONTAL
        }
        (
            DoorV,
            Wall,
            StairN | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            _,
        )
        | (DoorV, Wall, Empty | DoorH | DoorV, _, _) => DOOR_SOUTHWALLCORNER,
        (Wall, Empty | DoorH, _, Empty | DoorV, _)
        | (
            Wall,
            StairS,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            Empty | DoorV,
            _,
        ) => WALLCORNER,
        (Wall, Empty | DoorH, _, DoorH, _)
        | (Wall, StairS, Empty | DoorH | DoorV | StairN | StairE | StairS | StairW, DoorH, _) => {
            WALLCORNER_EASTDOOR
        }
        (Wall, DoorV, _, Empty | DoorV, _) => WALLCORNER_SOUTHDOOR,
        (Wall, DoorV, _, DoorH, _) => WALLCORNER_SOUTHDOOR_EASTDOOR,
        (Wall, Empty | DoorH, _, Wall | StairN | StairE | StairS | StairW, _)
        | (
            Wall,
            StairS,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            Wall | StairN | StairE | StairS | StairW,
            _,
        ) => WALLHORIZONTAL,
        (Wall, DoorV, _, Wall | StairN | StairE | StairS | StairW, _) => WALLHORIZONTAL_SOUTHDOOR,
        (Wall, Wall, Empty | DoorH | DoorV, Empty | DoorV, _) => WALLVERTICAL,
        (Wall, Wall, Empty | DoorH | DoorV, Wall | StairN | StairE | StairS | StairW, _) => {
            WALLINVERSECORNER
        }
        (Wall, Wall, Empty | DoorH | DoorV, DoorH, _) => WALLVERTICAL_EASTDOOR,
        (Wall, Wall, Wall | StairN | StairE | StairS | StairW, Empty | DoorV, _) => {
            WALLVERTICAL_SOUTHEASTWALL
        }
        (Wall, Wall, Wall | StairN | StairE | StairS | StairW, DoorH, _) => {
            WALLVERTICAL_SOUTHEASTWALL_EASTDOOR
        }
        (
            Wall,
            Wall,
            Wall | StairN | StairE | StairS | StairW,
            Wall | StairN | StairE | StairS | StairW,
            _,
        ) => WALLFULL,
        (Wall, StairN, Empty | DoorH | DoorV | StairE | StairS | StairW, Empty | DoorV, _) => {
            WALLCORNER_SOUTHSTAIRN
        }
        (Wall, StairN, Empty | DoorH | DoorV | StairE | StairS | StairW, DoorH, _) => {
            WALLCORNER_SOUTHSTAIRN_EASTDOOR
        }
        (
            Wall,
            StairN,
            Empty | DoorH | DoorV | StairE | StairS | StairW,
            Wall | StairN | StairE | StairS | StairW,
            _,
        ) => WALLHORIZONTAL_SOUTHSTAIRN,
        (Wall, StairN, Wall, Empty | DoorV, _) => WALLCORNER_SOUTHSTAIRNWALL,
        (Wall, StairN, Wall, DoorH, _) => WALLCORNER_SOUTHSTAIRNWALL_EASTDOOR,
        (Wall, StairN, Wall, Wall | StairN | StairE | StairS | StairW, _) => {
            WALLHORIZONTAL_SOUTHSTAIRNWALL
        }
        (Wall, StairN, StairN, Empty | DoorV, _) => WALLCORNER_SOUTHSTAIRNDOUBLE,
        (Wall, StairN, StairN, DoorH, _) => WALLCORNER_SOUTHSTAIRNDOUBLE_EASTDOOR,
        (Wall, StairN, StairN, Wall | StairN | StairE | StairS | StairW, _) => {
            WALLHORIZONTAL_SOUTHSTAIRNDOUBLE
        }
        (Wall, StairS, Wall, Empty | DoorV, _) => WALLCORNER_SOUTHSTAIRSWALL,
        (Wall, StairS, Wall, DoorH, _) => WALLCORNER_SOUTHSTAIRSWALL_EASTDOOR,
        (Wall, StairS, Wall, Wall | StairN | StairE | StairS | StairW, _) => {
            WALLHORIZONTAL_SOUTHSTAIRSWALL
        }
        (Wall, StairE, _, Empty | DoorV, _) => WALLCORNER_SOUTHSTAIRE,
        (Wall, StairE, _, DoorH, _) => WALLCORNER_SOUTHSTAIRE_EASTDOOR,
        (
            Wall,
            StairE,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            Wall | StairN | StairE | StairS | StairW,
            _,
        ) => WALLHORIZONTAL_SOUTHSTAIRE,
        (Wall, StairE, Wall, Wall | StairN | StairE | StairS | StairW, _) => {
            WALLHORIZONTAL_SOUTHSTAIRE
        }
        (Wall, StairW, _, Empty | DoorV, _) => WALLCORNER_SOUTHSTAIRW,
        (Wall, StairW, _, DoorH, _) => WALLCORNER_SOUTHSTAIRW_EASTDOOR,
        (
            Wall,
            StairW,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            Wall | StairN | StairE | StairS | StairW,
            _,
        ) => WALLHORIZONTAL_SOUTHSTAIRW,
        (Wall, StairW, Wall, Wall | StairN | StairE | StairS | StairW, _) => {
            WALLHORIZONTAL_SOUTHSTAIRW
        }
        (StairN, Empty | DoorH | DoorV, _, Empty | DoorH | DoorV | StairE | StairS | StairW, _)
        | (
            StairN,
            StairS,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairE | StairS | StairW,
            _,
        ) => STAIRN,
        (StairN, Empty | DoorH | DoorV, _, Wall, _)
        | (StairN, StairS, Empty | DoorH | DoorV | StairN | StairE | StairS | StairW, Wall, _) => {
            STAIRN_EASTWALL
        }
        (StairN, Empty | DoorH | DoorV, _, StairN, _)
        | (StairN, StairS, Empty | DoorH | DoorV | StairN | StairE | StairS | StairW, StairN, _) => {
            STAIRN_EASTSTAIR
        }
        (
            StairN,
            Wall,
            Empty | DoorH | DoorV,
            Empty | DoorH | DoorV | StairE | StairS | StairW,
            _,
        ) => STAIRN_SOUTHWALLCORNER,
        (StairN, Wall, Empty | DoorH | DoorV, Wall, _) => STAIRN_SOUTHWALLCORNER_EASTWALL,
        (StairN, Wall, Empty | DoorH | DoorV, StairN, _) => STAIRN_SOUTHWALLCORNER_EASTSTAIR,
        (
            StairN,
            Wall,
            Wall | StairN | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairE | StairS | StairW,
            _,
        ) => STAIRN_SOUTHWALLHORIZONTAL,
        (StairN, Wall, Wall | StairN | StairE | StairS | StairW, Wall, _) => {
            STAIRN_SOUTHWALLHORIZONTAL_EASTWALL
        }
        (StairN, Wall, Wall | StairN | StairE | StairS | StairW, StairN, _) => {
            STAIRN_SOUTHWALLHORIZONTAL_EASTSTAIR
        }
        (
            StairN,
            StairN,
            Empty | DoorH | DoorV | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairE | StairS | StairW,
            _,
        ) => STAIRN_SOUTHSTAIRN,
        (StairN, StairN, Empty | DoorH | DoorV | StairE | StairS | StairW, Wall, _) => {
            STAIRN_SOUTHSTAIRN_EASTWALL
        }
        (StairN, StairN, Empty | DoorH | DoorV | StairE | StairS | StairW, StairN, _) => {
            STAIRN_SOUTHSTAIRN_EASTSTAIR
        }
        (StairN, StairN, Wall, Empty | DoorH | DoorV | StairE | StairS | StairW, _) => {
            STAIRN_SOUTHSTAIRNWALL
        }
        (StairN, StairN, Wall, Wall, _) => STAIRN_SOUTHSTAIRNWALL_EASTWALL,
        (StairN, StairN, Wall, StairN, _) => STAIRN_SOUTHSTAIRNWALL_EASTSTAIR,
        (StairN, StairN, StairN, Empty | DoorH | DoorV | StairE | StairS | StairW, _) => {
            STAIRN_SOUTHSTAIRNDOUBLE
        }
        (StairN, StairN, StairN, Wall, _) => STAIRN_SOUTHSTAIRNDOUBLE_EASTWALL,
        (StairN, StairN, StairN, StairN, _) => STAIRN_SOUTHSTAIRNDOUBLE_EASTSTAIR,
        (StairN, StairS, Wall, Empty | DoorH | DoorV | StairE | StairS | StairW, _) => {
            STAIRN_SOUTHSTAIRSWALL
        }
        (StairN, StairS, Wall, Wall, _) => STAIRN_SOUTHSTAIRSWALL_EASTWALL,
        (StairN, StairS, Wall, StairN, _) => STAIRN_SOUTHSTAIRSWALL_EASTSTAIR,
        (StairN, StairE, _, Empty | DoorH | DoorV | StairE | StairS | StairW, _) => {
            STAIRN_SOUTHSTAIRE
        }
        (StairN, StairW, _, Empty | DoorH | DoorV | StairE | StairS | StairW, _) => {
            STAIRN_SOUTHSTAIRW
        }
        (StairN, StairE, _, Wall, _) => STAIRN_SOUTHSTAIRE_EASTWALL,
        (StairN, StairW, _, Wall, _) => STAIRN_SOUTHSTAIRW_EASTWALL,
        (StairN, StairE, _, StairN, _) => STAIRN_SOUTHSTAIRE_EASTSTAIR,
        (StairN, StairW, _, StairN, _) => STAIRN_SOUTHSTAIRW_EASTSTAIR,
        (StairS, Empty | DoorH | DoorV, _, Empty | DoorH | DoorV | StairN | StairE | StairW, _)
        | (
            StairS,
            StairS,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairN | StairE | StairW,
            _,
        ) => STAIRS,
        (StairS, Empty | DoorH | DoorV, _, Wall, _)
        | (StairS, StairS, Empty | DoorH | DoorV | StairN | StairE | StairS | StairW, Wall, _) => {
            STAIRS_EASTWALL
        }
        (StairS, Empty | DoorH | DoorV, _, StairS, _)
        | (StairS, StairS, Empty | DoorH | DoorV | StairN | StairE | StairS | StairW, StairS, _) => {
            STAIRS_EASTSTAIR
        }
        (
            StairS,
            Wall,
            Empty | DoorH | DoorV,
            Empty | DoorH | DoorV | StairN | StairE | StairW,
            _,
        ) => STAIRS_SOUTHWALLCORNER,
        (StairS, Wall, Empty | DoorH | DoorV, Wall, _) => STAIRS_SOUTHWALLCORNER_EASTWALL,
        (StairS, Wall, Empty | DoorH | DoorV, StairS, _) => STAIRS_SOUTHWALLCORNER_EASTSTAIR,
        (
            StairS,
            Wall,
            Wall | StairN | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairN | StairE | StairW,
            _,
        ) => STAIRS_SOUTHWALLHORIZONTAL,
        (StairS, Wall, Wall | StairN | StairE | StairS | StairW, Wall, _) => {
            STAIRS_SOUTHWALLHORIZONTAL_EASTWALL
        }
        (StairS, Wall, Wall | StairN | StairE | StairS | StairW, StairS, _) => {
            STAIRS_SOUTHWALLHORIZONTAL_EASTSTAIR
        }
        (
            StairS,
            StairN,
            Empty | DoorH | DoorV | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairN | StairE | StairW,
            _,
        ) => STAIRS_SOUTHSTAIRN,
        (StairS, StairN, Empty | DoorH | DoorV | StairE | StairS | StairW, Wall, _) => {
            STAIRS_SOUTHSTAIRN_EASTWALL
        }
        (StairS, StairN, Empty | DoorH | DoorV | StairE | StairS | StairW, StairS, _) => {
            STAIRS_SOUTHSTAIRN_EASTSTAIR
        }
        (StairS, StairN, Wall, Empty | DoorH | DoorV | StairN | StairE | StairW, _) => {
            STAIRS_SOUTHSTAIRNWALL
        }
        (StairS, StairN, Wall, Wall, _) => STAIRS_SOUTHSTAIRNWALL_EASTWALL,
        (StairS, StairN, Wall, StairS, _) => STAIRS_SOUTHSTAIRNWALL_EASTSTAIR,
        (StairS, StairN, StairN, Empty | DoorH | DoorV | StairN | StairE | StairW, _) => {
            STAIRS_SOUTHSTAIRNDOUBLE
        }
        (StairS, StairN, StairN, Wall, _) => STAIRS_SOUTHSTAIRNDOUBLE_EASTWALL,
        (StairS, StairN, StairN, StairS, _) => STAIRS_SOUTHSTAIRNDOUBLE_EASTSTAIR,
        (StairS, StairS, Wall, Empty | DoorH | DoorV | StairN | StairE | StairW, _) => {
            STAIRS_SOUTHSTAIRSWALL
        }
        (StairS, StairS, Wall, Wall, _) => STAIRS_SOUTHSTAIRSWALL_EASTWALL,
        (StairS, StairS, Wall, StairS, _) => STAIRS_SOUTHSTAIRSWALL_EASTSTAIR,
        (StairS, StairE, _, Empty | DoorH | DoorV | StairN | StairE | StairW, _) => {
            STAIRS_SOUTHSTAIRE
        }
        (StairS, StairW, _, Empty | DoorH | DoorV | StairN | StairE | StairW, _) => {
            STAIRS_SOUTHSTAIRW
        }
        (StairS, StairE, _, Wall, _) => STAIRS_SOUTHSTAIRE_EASTWALL,
        (StairS, StairW, _, Wall, _) => STAIRS_SOUTHSTAIRW_EASTWALL,
        (StairS, StairE, _, StairS, _) => STAIRS_SOUTHSTAIRE_EASTSTAIR,
        (StairS, StairW, _, StairS, _) => STAIRS_SOUTHSTAIRW_EASTSTAIR,
        (StairE, Empty | DoorH | DoorV, _, _, _)
        | (StairE, StairS, Empty | DoorH | DoorV | StairN | StairE | StairS | StairW, _, _) => {
            STAIRE
        }
        (StairE, Wall, Empty | DoorH | DoorV, _, _) => STAIRE_SOUTHWALLCORNER,
        (
            StairE,
            Wall,
            Wall | StairN | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            _,
        ) => STAIRE_SOUTHWALLHORIZONTAL,
        (StairE, Wall, Wall | StairN | StairE | StairS | StairW, Wall, _) => STAIRE_SOUTHWALLFULL,
        (StairE, StairN, Empty | DoorH | DoorV | StairE | StairS | StairW, _, _) => {
            STAIRE_SOUTHSTAIRN
        }
        (StairE, StairN, Wall, Empty | DoorH | DoorV | StairN | StairE | StairS | StairW, _) => {
            STAIRE_SOUTHSTAIRNWALL
        }
        (StairE, StairN, Wall, Wall, _) => STAIRE_SOUTHSTAIRNWALLFULL,
        (StairE, StairN, StairN, _, _) => STAIRE_SOUTHSTAIRNDOUBLE,
        (StairE, StairS, Wall, _, _) => STAIRE_SOUTHSTAIRSWALL,
        (StairE, StairE, _, _, _) => STAIRE_SOUTHSTAIRE,
        (StairE, StairW, _, _, _) => STAIRE_SOUTHSTAIRW,
        (StairW, Empty | DoorH | DoorV, _, _, Empty | DoorH | DoorV | StairN | StairE | StairS)
        | (
            StairW,
            StairS,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            _,
            Empty | DoorH | DoorV | StairN | StairE | StairS,
        ) => STAIRW,
        (StairW, Empty | DoorH | DoorV, _, _, Wall)
        | (StairW, StairS, Empty | DoorH | DoorV | StairN | StairE | StairS | StairW, _, Wall) => {
            STAIRW_NORTHWALL
        }
        (StairW, Empty | DoorH | DoorV, _, _, StairW)
        | (StairW, StairS, Empty | DoorH | DoorV | StairN | StairE | StairS | StairW, _, StairW) => {
            STAIRW_NORTHSTAIR
        }
        (
            StairW,
            Wall,
            Empty | DoorH | DoorV,
            _,
            Empty | DoorH | DoorV | StairN | StairE | StairS,
        ) => STAIRW_SOUTHWALLCORNER,
        (StairW, Wall, Empty | DoorH | DoorV, _, Wall) => STAIRW_SOUTHWALLCORNER_NORTHWALL,
        (StairW, Wall, Empty | DoorH | DoorV, _, StairW) => STAIRW_SOUTHWALLCORNER_NORTHSTAIR,
        (
            StairW,
            Wall,
            Wall | StairN | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairN | StairE | StairS,
        ) => STAIRW_SOUTHWALLHORIZONTAL,
        (
            StairW,
            Wall,
            Wall | StairN | StairE | StairS | StairW,
            Wall,
            Empty | DoorH | DoorV | StairN | StairE | StairS,
        ) => STAIRW_SOUTHWALLFULL,
        (
            StairW,
            Wall,
            Wall | StairN | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            Wall,
        ) => STAIRW_SOUTHWALLHORIZONTAL_NORTHWALL,
        (StairW, Wall, Wall | StairN | StairE | StairS | StairW, Wall, Wall) => {
            STAIRW_SOUTHWALLFULL_NORTHWALL
        }
        (
            StairW,
            Wall,
            Wall | StairN | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            StairW,
        ) => STAIRW_SOUTHWALLHORIZONTAL_NORTHSTAIR,
        (StairW, Wall, Wall | StairN | StairE | StairS | StairW, Wall, StairW) => {
            STAIRW_SOUTHWALLFULL_NORTHSTAIR
        }
        (
            StairW,
            StairN,
            Empty | DoorH | DoorV | StairE | StairS | StairW,
            _,
            Empty | DoorH | DoorV | StairN | StairE | StairS,
        ) => STAIRW_SOUTHSTAIRN,
        (StairW, StairN, Empty | DoorH | DoorV | StairE | StairS | StairW, _, Wall) => {
            STAIRW_SOUTHSTAIRN_NORTHWALL
        }
        (StairW, StairN, Empty | DoorH | DoorV | StairE | StairS | StairW, _, StairW) => {
            STAIRW_SOUTHSTAIRN_NORTHSTAIR
        }
        (
            StairW,
            StairN,
            Wall,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            Empty | DoorH | DoorV | StairN | StairE | StairS,
        ) => STAIRW_SOUTHSTAIRNWALL,
        (StairW, StairN, Wall, Wall, Empty | DoorH | DoorV | StairN | StairE | StairS) => {
            STAIRW_SOUTHSTAIRNWALLFULL
        }
        (StairW, StairN, Wall, Empty | DoorH | DoorV | StairN | StairE | StairS | StairW, Wall) => {
            STAIRW_SOUTHSTAIRNWALL_NORTHWALL
        }
        (StairW, StairN, Wall, Wall, Wall) => STAIRW_SOUTHSTAIRNWALLFULL_NORTHWALL,
        (
            StairW,
            StairN,
            Wall,
            Empty | DoorH | DoorV | StairN | StairE | StairS | StairW,
            StairW,
        ) => STAIRW_SOUTHSTAIRNWALL_NORTHSTAIR,
        (StairW, StairN, Wall, Wall, StairW) => STAIRW_SOUTHSTAIRNWALLFULL_NORTHSTAIR,
        (StairW, StairN, StairN, _, Empty | DoorH | DoorV | StairN | StairE | StairS) => {
            STAIRW_SOUTHSTAIRNDOUBLE
        }
        (StairW, StairN, StairN, _, Wall) => STAIRW_SOUTHSTAIRNDOUBLE_NORTHWALL,
        (StairW, StairN, StairN, _, StairW) => STAIRW_SOUTHSTAIRNDOUBLE_NORTHSTAIR,
        (StairW, StairS, Wall, _, Empty | DoorH | DoorV | StairN | StairE | StairS) => {
            STAIRW_SOUTHSTAIRSWALL
        }
        (StairW, StairS, Wall, _, Wall) => STAIRW_SOUTHSTAIRSWALL_NORTHWALL,
        (StairW, StairS, Wall, _, StairW) => STAIRW_SOUTHSTAIRSWALL_NORTHSTAIR,
        (StairW, StairE, _, _, Empty | DoorH | DoorV | StairN | StairE | StairS) => {
            STAIRW_SOUTHSTAIRE
        }
        (StairW, StairE, _, _, Wall) => STAIRW_SOUTHSTAIRE_NORTHWALL,
        (StairW, StairE, _, _, StairW) => STAIRW_SOUTHSTAIRE_NORTHSTAIR,
        (StairW, StairW, _, _, Empty | DoorH | DoorV | StairN | StairE | StairS) => {
            STAIRW_SOUTHSTAIRW
        }
        (StairW, StairW, _, _, Wall) => STAIRW_SOUTHSTAIRW_NORTHWALL,
        (StairW, StairW, _, _, StairW) => STAIRW_SOUTHSTAIRW_NORTHSTAIR,
    }
}

impl TileVariant {
    pub fn flip_x(self) -> Self {
        match self {
            TileVariant::StairE => TileVariant::StairW,
            TileVariant::StairW => TileVariant::StairE,
            _ => self,
        }
    }
}

#[test]
fn test_sprite_offset() {}
