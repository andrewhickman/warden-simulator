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

use crate::{
    assets::AssetHandles,
    layers::{BASE_LAYER, TOP_LAYER},
};

pub const SPRITE_CHUNK_SIZE: u16 = 16;

pub const DIRT_OFFSET: u16 = 0;
pub const WALL_TOP_OFFSET: u16 = DIRT_OFFSET + 256;
pub const WALL_BASE_OFFSET: u16 = WALL_TOP_OFFSET + 13;

pub struct TilePlugin;

#[derive(Resource)]
pub struct TileChunkMesh(Handle<Mesh>);

#[derive(Clone, Component, Default, Debug)]
#[require(Transform, Visibility)]
#[component(on_add = TileChunkSprites::on_add)]
pub struct TileChunkSprites {
    base_material: Handle<TilemapChunkMaterial>,
    base: Handle<Image>,
    top_material: Handle<TilemapChunkMaterial>,
    top: Handle<Image>,
}

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileChunkMesh>();

        app.add_systems(PostUpdate, update_chunk_data.before(AssetEventSystems));

        app.register_required_components::<Layer, Visibility>();
        app.register_required_components::<TileChunk, TileChunkSprites>();
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
    mut query: Query<(&TileChunk, &TileChunkSprites), Changed<TileChunk>>,
    mut materials: ResMut<Assets<TilemapChunkMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    query.iter_mut().for_each(|(chunk, sprites)| {
        // Mark materials as changed
        materials.get_mut(&sprites.base_material);
        materials.get_mut(&sprites.top_material);

        update_chunk_image(&mut images, sprites.base.id(), chunk, pack_tile_base);
        update_chunk_image(&mut images, sprites.top.id(), chunk, pack_tile_top);
    });
}

impl TileChunkSprites {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let chunk = world.get::<TileChunk>(context.entity).unwrap();
        let position = chunk.position();

        let (base, base_material) = spawn_chunk_image(
            &mut world,
            context.entity,
            Transform::from_xyz(
                chunk_coord_transform(position.x()),
                chunk_coord_transform(position.y()),
                BASE_LAYER,
            ),
            pack_tile_base,
        );
        let (top, top_material) = spawn_chunk_image(
            &mut world,
            context.entity,
            Transform::from_xyz(
                chunk_coord_transform(position.x()),
                chunk_coord_transform(position.y()) + 1.0,
                TOP_LAYER,
            ),
            pack_tile_top,
        );

        *world.get_mut::<TileChunkSprites>(context.entity).unwrap() = TileChunkSprites {
            base_material,
            base,
            top_material,
            top,
        };
    }
}

fn spawn_chunk_image(
    world: &mut DeferredWorld,
    id: Entity,
    transform: Transform,
    pack: impl Fn(TileChunkOffset, Tile) -> PackedTileData,
) -> (Handle<Image>, Handle<TilemapChunkMaterial>) {
    let mesh = world.resource::<TileChunkMesh>().0.clone();

    let chunk = world.get::<TileChunk>(id).unwrap();
    let packed_data = chunk
        .tiles()
        .map(|(offset, tile)| pack(offset, tile))
        .collect::<Vec<PackedTileData>>();

    let tileset = world.resource::<AssetHandles>().tileset();
    let tile_data = world
        .resource_mut::<Assets<Image>>()
        .add(make_chunk_tile_data_image(
            &UVec2::splat(CHUNK_SIZE as u32),
            &packed_data,
        ));

    let material = world
        .resource_mut::<Assets<TilemapChunkMaterial>>()
        .add(TilemapChunkMaterial {
            alpha_mode: AlphaMode2d::Opaque,
            tileset,
            tile_data: tile_data.clone(),
        });

    world.commands().spawn((
        ChildOf(id),
        Mesh2d(mesh),
        MeshMaterial2d(material.clone()),
        transform,
    ));

    (tile_data, material)
}

fn update_chunk_image(
    images: &mut Assets<Image>,
    id: AssetId<Image>,
    chunk: &TileChunk,
    pack: impl Fn(TileChunkOffset, Tile) -> PackedTileData,
) {
    let Some(image) = images.get_mut(id) else {
        error!("image asset not found for chunk {chunk:?}");
        return;
    };

    let Some(data) = image.data.as_mut() else {
        error!("image data not found for chunk {chunk:?}");
        return;
    };

    data.clear();
    for (offset, tile) in chunk.tiles() {
        let packed = pack(offset, tile);
        data.extend_from_slice(bytemuck::bytes_of(&packed));
    }
}

fn pack_tile_base(offset: TileChunkOffset, tile: Tile) -> PackedTileData {
    let tileset_index = match tile.material() {
        TileMaterial::Empty => DIRT_OFFSET + dirt_sprite_offset(offset),
        TileMaterial::Wall => WALL_BASE_OFFSET + wall_base_sprite_offset(tile.occupancy()),
    };

    PackedTileData::from(TileData {
        tileset_index,
        color: Color::WHITE,
        visible: true,
    })
}

fn pack_tile_top(_: TileChunkOffset, tile: Tile) -> PackedTileData {
    let tileset_index = match tile.material() {
        TileMaterial::Empty => return PackedTileData::from(None),
        TileMaterial::Wall => WALL_TOP_OFFSET + wall_top_sprite_offset(tile.occupancy()),
    };

    PackedTileData::from(TileData {
        tileset_index,
        color: Color::WHITE,
        visible: true,
    })
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

fn wall_top_sprite_offset(occupancy: TileOccupancy) -> u16 {
    const LOOKUP: [u8; 256] = [
        0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3,
        2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1,
        2, 3, 2, 1, 4, 5, 4, 5, 6, 7, 6, 7, 4, 5, 4, 5, 6, 7, 6, 7, 4, 5, 4, 5, 6, 7, 6, 7, 4, 5,
        4, 5, 6, 7, 6, 7, 4, 5, 4, 5, 6, 7, 6, 7, 4, 5, 4, 5, 6, 7, 6, 7, 4, 5, 4, 5, 6, 7, 6, 7,
        4, 5, 4, 5, 6, 7, 6, 7, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3,
        2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 0, 1, 0, 1,
        2, 3, 2, 1, 0, 1, 0, 1, 2, 3, 2, 1, 4, 1, 4, 1, 6, 7, 6, 1, 4, 1, 4, 1, 6, 7, 6, 1, 4, 1,
        4, 1, 6, 7, 6, 1, 4, 1, 4, 1, 6, 7, 6, 1, 4, 1, 4, 1, 6, 7, 6, 1, 4, 1, 4, 1, 6, 7, 6, 1,
        4, 1, 4, 1, 6, 7, 6, 1, 4, 1, 4, 1, 6, 7, 6, 1,
    ];

    LOOKUP[occupancy.bits() as usize] as u16
}

#[test]
fn test_tile_sprite_index_bottom() {
    use std::collections::{HashMap, hash_map};

    let mut patterns: HashMap<TileOccupancy, u16> = HashMap::new();

    let bottom_mask = TileOccupancy::SOUTH
        | TileOccupancy::SOUTH_WEST
        | TileOccupancy::SOUTH_EAST
        | TileOccupancy::WEST
        | TileOccupancy::EAST;

    for i in 0..=255u8 {
        let occupancy = TileOccupancy::from_bits_retain(i) & bottom_mask;
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
        | TileOccupancy::NORTH_WEST
        | TileOccupancy::NORTH_EAST
        | TileOccupancy::WEST
        | TileOccupancy::EAST;

    for i in 0..=255u8 {
        let occupancy = TileOccupancy::from_bits_retain(i) & top_mask;
        let mut normal = occupancy;

        if normal.contains(TileOccupancy::NORTH)
            && (!normal.contains(TileOccupancy::EAST) || normal.contains(TileOccupancy::NORTH_EAST))
            && (!normal.contains(TileOccupancy::WEST) || normal.contains(TileOccupancy::NORTH_WEST))
        {
            normal = TileOccupancy::NORTH;
        }

        normal.remove(TileOccupancy::NORTH_WEST | TileOccupancy::NORTH_EAST);

        let index = patterns.len() as u16;

        match patterns.entry(normal) {
            hash_map::Entry::Occupied(entry) => {
                assert_eq!(wall_top_sprite_offset(occupancy), *entry.get() as u16);
            }
            hash_map::Entry::Vacant(entry) => {
                assert_eq!(wall_top_sprite_offset(occupancy), index);
                entry.insert(index);
            }
        }
    }

    assert_eq!(patterns.len(), 8);
}
