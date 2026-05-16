pub mod material;

use bevy_app::prelude::*;
use bevy_asset::{AssetEventSystems, prelude::*};
use bevy_camera::prelude::*;
use bevy_ecs::prelude::*;
use bevy_image::prelude::*;
use bevy_log::prelude::*;
use bevy_math::prelude::*;
use bevy_mesh::prelude::*;
use bevy_sprite_render::{AlphaMode2d, prelude::*};
use bevy_transform::prelude::*;

use wdn_physics::{
    layer::Layer,
    tile::{
        CHUNK_SIZE, TileChunkOffset, TileChunkPosition,
        storage::{Tile, TileChunk, TileMaterial, TileOccupancy},
    },
};

use crate::{
    assets::AssetHandles,
    layers::BASE_LAYER,
    tile::material::{
        PackedTileData, TileChunkMaterial, TileChunkMaterialPlugin, make_tile_chunk_image,
    },
};

pub const SPRITE_CHUNK_SIZE: u16 = 16;

pub const DIRT_OFFSET: u16 = 0;
pub const WALL_TOP_OFFSET: u16 = DIRT_OFFSET + 256;
pub const WALL_BASE_OFFSET: u16 = WALL_TOP_OFFSET + 13;

pub struct TilePlugin;

#[derive(Resource)]
pub struct TileChunkMesh(Handle<Mesh>);

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileChunkMesh>();

        app.add_systems(PostUpdate, update_chunk.before(AssetEventSystems));

        app.register_required_components::<Layer, Visibility>();
        app.register_required_components::<TileChunk, Mesh2d>();
        app.register_required_components::<TileChunk, MeshMaterial2d<TileChunkMaterial>>();

        app.add_plugins(TileChunkMaterialPlugin);
    }
}

impl FromWorld for TileChunkMesh {
    fn from_world(world: &mut World) -> Self {
        let handle = world
            .resource_mut::<Assets<Mesh>>()
            .add(Rectangle::from_length(CHUNK_SIZE as f32));
        TileChunkMesh(handle)
    }
}

pub fn update_chunk(
    assets: Res<AssetHandles>,
    mesh: Res<TileChunkMesh>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TileChunkMaterial>>,
    mut chunks: Query<
        (
            Ref<TileChunk>,
            &mut Transform,
            &mut Mesh2d,
            &mut MeshMaterial2d<TileChunkMaterial>,
        ),
        Changed<TileChunk>,
    >,
) {
    chunks.iter_mut().for_each(
        |(chunk, mut transform, mut mesh_handle, mut material_handle)| {
            if chunk.is_added() {
                let position = chunk.position();
                *transform = chunk_transform(position);

                mesh_handle.0 = mesh.0.clone();

                let image = images.add(make_tile_chunk_image());
                material_handle.0 = materials.add(TileChunkMaterial {
                    alpha_mode: AlphaMode2d::Blend,
                    tileset: assets.tileset(),
                    tile_data: image,
                });
            }

            let Some(material) = materials.get_mut(material_handle.id()) else {
                error!("material asset not found for chunk {chunk:?}");
                return;
            };

            let Some(image) = images.get_mut(material.tile_data.id()) else {
                error!("image asset not found for chunk {chunk:?}");
                return;
            };

            let Some(data) = image.data.as_mut() else {
                error!("image data not found for chunk {chunk:?}");
                return;
            };

            data.clear();
            for (offset, tile) in chunk.tiles() {
                let packed = pack_tile(offset, tile);
                data.extend_from_slice(bytemuck::bytes_of(&packed));
            }
        },
    );
}

fn pack_tile(offset: TileChunkOffset, tile: Tile) -> PackedTileData {
    let base = match tile.material() {
        TileMaterial::Empty => DIRT_OFFSET + dirt_sprite_offset(offset),
        TileMaterial::Wall => WALL_BASE_OFFSET + wall_base_sprite_offset(tile.occupancy()),
    };

    let top = WALL_TOP_OFFSET + wall_top_sprite_offset(tile.is_solid(), tile.occupancy());

    PackedTileData { base, top }
}

fn chunk_transform(position: TileChunkPosition) -> Transform {
    Transform::from_xyz(
        chunk_coord_transform(position.x()),
        chunk_coord_transform(position.y()),
        BASE_LAYER,
    )
}

fn chunk_coord_transform(d: i16) -> f32 {
    d as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0
}

fn dirt_sprite_offset(position: TileChunkOffset) -> u16 {
    (SPRITE_CHUNK_SIZE - 1 - position.y().rem_euclid(SPRITE_CHUNK_SIZE)) * SPRITE_CHUNK_SIZE
        + position.x().rem_euclid(SPRITE_CHUNK_SIZE)
}

fn wall_base_sprite_offset(occupancy: TileOccupancy) -> u16 {
    const LOOKUP: [u8; 256] = [
        0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4,
        4, 4, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 2, 2, 2, 2,
        4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 7, 7,
        7, 7, 9, 9, 9, 9, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6, 6, 6, 6, 10, 10, 10, 10, 11, 11,
        11, 11, 10, 10, 10, 10, 12, 12, 12, 12, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 2,
        2, 2, 2, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1,
        1, 2, 2, 2, 2, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5, 5, 6,
        6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 7, 7, 7, 7, 9, 9, 9, 9, 5, 5, 5, 5, 6, 6, 6, 6, 5, 5, 5,
        5, 6, 6, 6, 6, 10, 10, 10, 10, 11, 11, 11, 11, 10, 10, 10, 10, 12, 12, 12, 12,
    ];

    LOOKUP[occupancy.bits() as usize] as u16
}

fn wall_top_sprite_offset(solid: bool, mut occupancy: TileOccupancy) -> u16 {
    occupancy.set(TileOccupancy::NORTH, solid);

    const LOOKUP: [u8; 256] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 2, 3, 2, 3, 2, 0,
        2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 5, 4, 5, 4, 5, 4, 5, 6, 7, 6, 7,
        6, 5, 6, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 2, 3,
        2, 3, 2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 4, 0, 4, 0,
        6, 3, 6, 3, 6, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0,
        1, 0, 2, 3, 2, 3, 2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 5, 4, 5,
        4, 5, 4, 5, 6, 7, 6, 7, 6, 5, 6, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
        1, 0, 1, 0, 1, 0, 2, 3, 2, 3, 2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        4, 0, 4, 0, 4, 0, 4, 0, 6, 3, 6, 3, 6, 0, 6, 0,
    ];

    LOOKUP[occupancy.bits() as usize] as u16
}

#[test]
fn test_tile_sprite_index_bottom() {
    use std::collections::{HashMap, hash_map};

    let mut patterns: HashMap<TileOccupancy, u16> = HashMap::new();

    let base_mask = TileOccupancy::SOUTH
        | TileOccupancy::SOUTH_WEST
        | TileOccupancy::SOUTH_EAST
        | TileOccupancy::WEST
        | TileOccupancy::EAST;

    for i in 0..=255u8 {
        let occupancy = TileOccupancy::from_bits_retain(i) & base_mask;
        let mut normal = occupancy;

        if !normal.contains(TileOccupancy::NORTH | TileOccupancy::WEST) {
            normal.remove(TileOccupancy::NORTH_WEST);
        }

        if !normal.contains(TileOccupancy::NORTH | TileOccupancy::EAST) {
            normal.remove(TileOccupancy::NORTH_EAST);
        }

        if !normal.contains(TileOccupancy::SOUTH | TileOccupancy::WEST) {
            normal.remove(TileOccupancy::SOUTH_WEST);
        }

        if !normal.contains(TileOccupancy::SOUTH | TileOccupancy::EAST) {
            normal.remove(TileOccupancy::SOUTH_EAST);
        }

        let index = patterns.len() as u16;

        match patterns.entry(normal) {
            hash_map::Entry::Occupied(entry) => {
                assert_eq!(wall_base_sprite_offset(occupancy), *entry.get() as u16);
            }
            hash_map::Entry::Vacant(entry) => {
                assert_eq!(wall_base_sprite_offset(occupancy), index);
                entry.insert(index);
            }
        }
    }

    assert_eq!(patterns.len(), 13);
}

#[test]
fn test_tile_sprite_index_top() {
    use std::collections::{HashMap, hash_map};

    let mut patterns: HashMap<TileOccupancy, u16> = HashMap::new();

    let top_mask = TileOccupancy::NORTH
        | TileOccupancy::SOUTH
        | TileOccupancy::SOUTH_WEST
        | TileOccupancy::SOUTH_EAST
        | TileOccupancy::WEST
        | TileOccupancy::EAST;

    for i in 0..=255u8 {
        let occupancy = TileOccupancy::from_bits_retain(i) & top_mask;
        let mut normal = occupancy;

        if !normal.contains(TileOccupancy::SOUTH) {
            normal = TileOccupancy::NONE;
        } else if normal.contains(TileOccupancy::NORTH) {
            if normal.contains(TileOccupancy::EAST) {
                normal.remove(TileOccupancy::SOUTH_EAST | TileOccupancy::EAST);
            }

            if normal.contains(TileOccupancy::WEST) {
                normal.remove(TileOccupancy::SOUTH_WEST | TileOccupancy::WEST);
            }

            if normal == (TileOccupancy::SOUTH | TileOccupancy::NORTH) {
                normal = TileOccupancy::NONE;
            }
        } else {
            normal.remove(TileOccupancy::EAST);
            normal.remove(TileOccupancy::WEST);
        }

        let index = patterns.len() as u16;

        match patterns.entry(normal) {
            hash_map::Entry::Occupied(entry) => {
                assert_eq!(
                    wall_top_sprite_offset(occupancy.contains(TileOccupancy::NORTH), occupancy),
                    *entry.get() as u16
                );
            }
            hash_map::Entry::Vacant(entry) => {
                assert_eq!(
                    wall_top_sprite_offset(occupancy.contains(TileOccupancy::NORTH), occupancy),
                    index
                );
                entry.insert(index);
            }
        }
    }

    assert_eq!(patterns.len(), 8);
}
