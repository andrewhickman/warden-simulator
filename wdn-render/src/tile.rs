use bevy_app::prelude::*;
use bevy_asset::{AssetEventSystems, prelude::*};
use bevy_camera::prelude::*;
use bevy_color::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_image::prelude::*;
use bevy_log::prelude::*;
use bevy_math::prelude::*;
use bevy_mesh::prelude::*;
use bevy_sprite_render::{
    AlphaMode2d, PackedTileData, TileData, TilemapChunkMaterial, make_chunk_tile_data_image,
    prelude::*,
};
use bevy_transform::prelude::*;

use wdn_physics::{
    layer::Layer,
    tile::{
        CHUNK_SIZE, TileChunkOffset, TileChunkPosition,
        storage::{Tile, TileChunk, TileMaterial, TileOccupancy},
    },
};

use crate::assets::AssetHandles;

pub const SPRITE_CHUNK_SIZE: u16 = 16;

pub const DIRT_OFFSET: u16 = 0;
pub const WALL_OFFSET: u16 = 256;

pub struct TilePlugin;

#[derive(Resource)]
pub struct TileChunkMesh(Handle<Mesh>);

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Mesh2d, MeshMaterial2d<TilemapChunkMaterial>, Transform)]
#[component(on_add = TileChunkSprite::on_add)]
pub struct TileChunkSprite;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileChunkMesh>();

        app.add_systems(PostUpdate, update_chunk_data.before(AssetEventSystems));

        app.register_required_components::<Layer, Visibility>();
        app.register_required_components::<TileChunk, TileChunkSprite>();
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

pub fn update_chunk_data(
    mut query: Query<(&TileChunk, &MeshMaterial2d<TilemapChunkMaterial>), Changed<TileChunk>>,
    mut materials: ResMut<Assets<TilemapChunkMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    query.iter_mut().for_each(|(chunk, material)| {
        let Some(material) = materials.get_mut(material.id()) else {
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
            let packed = pack_tile_chunk(offset, tile);
            data.extend_from_slice(bytemuck::bytes_of(&packed));
        }
    });
}

impl TileChunkSprite {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let mesh = world.resource::<TileChunkMesh>().0.clone();

        let chunk = world.get::<TileChunk>(context.entity).unwrap();
        let position = chunk.position();
        let packed_data = chunk
            .tiles()
            .map(|(offset, tile)| pack_tile_chunk(offset, tile))
            .collect::<Vec<PackedTileData>>();

        let tileset = world.resource::<AssetHandles>().tileset();
        let tile_data = world
            .resource_mut::<Assets<Image>>()
            .add(make_chunk_tile_data_image(
                &UVec2::splat(CHUNK_SIZE as u32),
                &packed_data,
            ));

        let material =
            world
                .resource_mut::<Assets<TilemapChunkMaterial>>()
                .add(TilemapChunkMaterial {
                    alpha_mode: AlphaMode2d::Opaque,
                    tileset,
                    tile_data,
                });

        let mut chunk = world.entity_mut(context.entity);
        chunk
            .get_mut::<MeshMaterial2d<TilemapChunkMaterial>>()
            .unwrap()
            .0 = material;
        chunk.get_mut::<Mesh2d>().unwrap().0 = mesh;
        *chunk.get_mut::<Transform>().unwrap() = tile_chunk_transform(position);
    }
}

fn pack_tile_chunk(offset: TileChunkOffset, tile: Tile) -> PackedTileData {
    let tileset_index = match tile.material() {
        TileMaterial::Empty => DIRT_OFFSET + dirt_sprite_offset(offset),
        TileMaterial::Wall => WALL_OFFSET + wall_sprite_offset(tile.occupancy()),
    };

    PackedTileData::from(TileData {
        tileset_index,
        color: Color::WHITE,
        visible: true,
    })
}

fn tile_chunk_transform(position: TileChunkPosition) -> Transform {
    Transform::from_xyz(
        position.x() as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0,
        position.y() as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0,
        0.0,
    )
}

fn dirt_sprite_offset(position: TileChunkOffset) -> u16 {
    (SPRITE_CHUNK_SIZE - 1 - position.y().rem_euclid(SPRITE_CHUNK_SIZE)) * SPRITE_CHUNK_SIZE
        + position.x().rem_euclid(SPRITE_CHUNK_SIZE)
}

fn wall_sprite_offset(occupancy: TileOccupancy) -> u16 {
    const LOOKUP: [u8; 256] = [
        0, 1, 0, 1, 2, 3, 2, 4, 0, 1, 0, 1, 2, 3, 2, 4, 5, 6, 5, 6, 7, 8, 7, 9, 5, 6, 5, 6, 10, 11,
        10, 12, 0, 1, 0, 1, 2, 3, 2, 4, 0, 1, 0, 1, 2, 3, 2, 4, 5, 6, 5, 6, 7, 8, 7, 9, 5, 6, 5, 6,
        10, 11, 10, 12, 13, 14, 13, 14, 15, 16, 15, 17, 13, 14, 13, 14, 15, 16, 15, 17, 18, 19, 18,
        19, 20, 21, 20, 22, 18, 19, 18, 19, 23, 24, 23, 25, 13, 14, 13, 14, 15, 16, 15, 17, 13, 14,
        13, 14, 15, 16, 15, 17, 26, 27, 26, 27, 28, 29, 28, 30, 26, 27, 26, 27, 31, 32, 31, 33, 0,
        1, 0, 1, 2, 3, 2, 4, 0, 1, 0, 1, 2, 3, 2, 4, 5, 6, 5, 6, 7, 8, 7, 9, 5, 6, 5, 6, 10, 11,
        10, 12, 0, 1, 0, 1, 2, 3, 2, 4, 0, 1, 0, 1, 2, 3, 2, 4, 5, 6, 5, 6, 7, 8, 7, 9, 5, 6, 5, 6,
        10, 11, 10, 12, 13, 34, 13, 34, 15, 35, 15, 36, 13, 34, 13, 34, 15, 35, 15, 36, 18, 37, 18,
        37, 20, 38, 20, 39, 18, 37, 18, 37, 23, 40, 23, 41, 13, 34, 13, 34, 15, 35, 15, 36, 13, 34,
        13, 34, 15, 35, 15, 36, 26, 42, 26, 42, 28, 43, 28, 44, 26, 42, 26, 42, 31, 45, 31, 46,
    ];

    LOOKUP[occupancy.bits() as usize] as u16
}

#[test]
fn test_tile_sprite_index() {
    use std::collections::{HashMap, hash_map};

    let mut patterns: HashMap<TileOccupancy, u16> = HashMap::new();

    for i in 0..=255u8 {
        let occupancy = TileOccupancy::from_bits_retain(i);
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
                assert_eq!(wall_sprite_offset(occupancy), *entry.get() as u16,);
            }
            hash_map::Entry::Vacant(entry) => {
                assert_eq!(wall_sprite_offset(occupancy), index);
                entry.insert(index);
            }
        }
    }

    assert_eq!(patterns.len(), 47);
}
