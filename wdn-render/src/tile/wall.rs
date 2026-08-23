use wdn_enumerate::*;
use wdn_physics::tile::{
    material::{TileKind, TileMaterialFlags},
    position::TilePosition,
    storage::TileStorage,
};

pub fn mid_offsets(storage: &TileStorage, tile: TilePosition) -> (u16, u16) {
    let left = MidSprite::resolve(
        get_tile_variant(storage, tile).flip(),
        get_tile_variant(storage, tile.north()).flip(),
        get_tile_variant(storage, tile.north().west()).flip(),
        get_tile_variant(storage, tile.west()).flip(),
        get_tile_variant(storage, tile.south().west()).flip(),
        get_tile_variant(storage, tile.south()).flip(),
    );
    let right = MidSprite::resolve(
        get_tile_variant(storage, tile),
        get_tile_variant(storage, tile.north()),
        get_tile_variant(storage, tile.north().east()),
        get_tile_variant(storage, tile.east()),
        get_tile_variant(storage, tile.south().east()),
        get_tile_variant(storage, tile.south()),
    );

    (left.id(), right.id())
}

pub fn top_offset(storage: &TileStorage, tile: TilePosition) -> (u16, u16) {
    let left = TopSprite::resolve(
        get_tile_variant(storage, tile).flip(),
        get_tile_variant(storage, tile.north()).flip(),
        get_tile_variant(storage, tile.north().west()).flip(),
        get_tile_variant(storage, tile.west()).flip(),
        get_tile_variant(storage, tile.south().west()).flip(),
        get_tile_variant(storage, tile.south()).flip(),
    );
    let right = TopSprite::resolve(
        get_tile_variant(storage, tile),
        get_tile_variant(storage, tile.north()),
        get_tile_variant(storage, tile.north().east()),
        get_tile_variant(storage, tile.east()),
        get_tile_variant(storage, tile.south().east()),
        get_tile_variant(storage, tile.south()),
    );

    (left.id(), right.id())
}

fn get_tile_variant(storage: &TileStorage, tile: TilePosition) -> TileVariant {
    let Some(data) = storage.get(tile) else {
        return TileVariant::Empty;
    };

    let material = data.material();
    match material.kind() {
        TileKind::Empty => TileVariant::Empty,
        TileKind::Wall => TileVariant::Wall,
        TileKind::Door => {
            match material
                .flags()
                .intersection(TileMaterialFlags::DOOR_DIRECTION_MASK)
            {
                TileMaterialFlags::DOOR_DIRECTION_HORIZONTAL => TileVariant::DoorH,
                TileMaterialFlags::DOOR_DIRECTION_VERTICAL => TileVariant::DoorV,
                _ => unreachable!("Invalid door direction for tile at {tile:?}"),
            }
        }
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
