use wdn_enumerate::*;
use wdn_physics::tile::{
    material::{TileKind, TileMaterialFlags},
    position::TilePosition,
    storage::TileStorage,
};

pub fn mid_offsets(storage: &TileStorage, tile: TilePosition) -> (u16, u16) {
    let left = MidSprite::resolve(
        get_tile_variant(storage, tile),
        get_tile_variant(storage, tile.north()),
        get_tile_variant(storage, tile.north().west()),
        get_tile_variant(storage, tile.west()),
        get_tile_variant(storage, tile.south().west()),
        get_tile_variant(storage, tile.south()),
    );
    let right = MidSprite::resolve(
        get_tile_variant(storage, tile),
        get_tile_variant(storage, tile.north().east()),
        get_tile_variant(storage, tile.north()),
        get_tile_variant(storage, tile.east()),
        get_tile_variant(storage, tile.south().east()),
        get_tile_variant(storage, tile.south()),
    );

    (left.id(), right.id())
}

pub fn top_offset(storage: &TileStorage, tile: TilePosition) -> (u16, u16) {
    let left = TopSprite::resolve(
        get_tile_variant(storage, tile),
        get_tile_variant(storage, tile.north()),
        get_tile_variant(storage, tile.north().west()),
        get_tile_variant(storage, tile.west()),
        get_tile_variant(storage, tile.south().west()),
        get_tile_variant(storage, tile.south()),
    );
    let right = TopSprite::resolve(
        get_tile_variant(storage, tile),
        get_tile_variant(storage, tile.north().east()),
        get_tile_variant(storage, tile.north()),
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
            if material
                .flags()
                .contains(TileMaterialFlags::DOOR_DIRECTION_VERTICAL)
            {
                TileVariant::DoorH
            } else {
                TileVariant::DoorV
            }
        }
        TileKind::Stairs => TileVariant::StairN,
    }
}
