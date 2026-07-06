pub mod material;
pub mod wall;

use bevy_app::prelude::*;
use bevy_asset::prelude::*;
use bevy_camera::prelude::*;
use bevy_ecs::{prelude::*, system::SystemParam};
use bevy_image::prelude::*;
use bevy_log::prelude::*;
use bevy_math::prelude::*;
use bevy_mesh::prelude::*;
use bevy_sprite_render::prelude::*;
use bevy_transform::prelude::*;

use wdn_physics::{
    kinematics::Position,
    tile::{
        CHUNK_SIZE,
        material::TileKind,
        position::{TileChunkOffset, TileChunkPosition},
        storage::{TileChunk, TileData},
    },
};
use wdn_world::door::Door;

use crate::{
    RenderSystems,
    assets::AssetHandles,
    depth::{GROUND_DEPTH, WALL_BASE_DEPTH, WALL_TOP_DEPTH},
    tile::material::{
        PackedTileData, TileChunkMaterial, TileChunkMaterialPlugin, make_tile_chunk_image,
    },
};

pub const SPRITE_CHUNK_SIZE: u16 = 16;

pub const DIRT_OFFSET: u16 = 0;
pub const WALL_OFFSET: u16 = DIRT_OFFSET + 512;

pub struct TilePlugin;

#[derive(Resource)]
pub struct TileChunkMesh(Handle<Mesh>);

#[derive(Clone, Component, Default, Debug)]
#[require(Transform, Visibility)]
pub struct TileChunkSprites {
    base: Handle<TileChunkMaterial>,
    top: Handle<TileChunkMaterial>,
}

#[derive(SystemParam)]
pub struct TileChunkSpriteParam<'w, 's> {
    commands: Commands<'w, 's>,
    assets: Res<'w, AssetHandles>,
    mesh: Res<'w, TileChunkMesh>,
    materials: ResMut<'w, Assets<TileChunkMaterial>>,
    images: ResMut<'w, Assets<Image>>,
}

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileChunkMesh>();

        app.add_systems(Update, update_chunk.in_set(RenderSystems::RenderTiles));

        app.register_required_components::<TileChunk, TileChunkSprites>();

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
    mut param: TileChunkSpriteParam,
    mut chunks: Query<
        (Entity, &TileChunk, &mut Transform, &mut TileChunkSprites),
        (Changed<TileChunk>, Without<Position>, Without<Door>),
    >,
) {
    chunks
        .iter_mut()
        .for_each(|(id, chunk, mut transform, mut sprites)| {
            if sprites.is_added() {
                let position = chunk.position();
                *transform = chunk_transform(position);

                sprites.base = param.spawn_chunk_material(id, GROUND_DEPTH);
                sprites.top = param.spawn_chunk_material(id, WALL_BASE_DEPTH);
            }

            param.update_chunk_material(sprites.base.id(), chunk, pack_ground_tile);
            param.update_chunk_material(sprites.top.id(), chunk, pack_wall_tile);
        });
}

impl TileChunkSpriteParam<'_, '_> {
    fn spawn_chunk_material(&mut self, id: Entity, depth: f32) -> Handle<TileChunkMaterial> {
        let tile_data = self.images.add(make_tile_chunk_image());
        let material = self.materials.add(TileChunkMaterial {
            tileset: self.assets.tileset(),
            tile_data,
        });

        self.commands.spawn((
            ChildOf(id),
            Mesh2d(self.mesh.0.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(0.0, 0.0, depth),
        ));

        material
    }

    fn update_chunk_material(
        &mut self,
        material: AssetId<TileChunkMaterial>,
        chunk: &TileChunk,
        pack: impl Fn(TileChunkOffset, TileData) -> [PackedTileData; 2],
    ) {
        let Some(material) = self.materials.get_mut(material) else {
            error!("material asset not found for chunk {chunk:?}");
            return;
        };

        let Some(mut image) = self.images.get_mut(material.tile_data.id()) else {
            error!("image asset not found for chunk {chunk:?}");
            return;
        };

        let Some(data) = image.data.as_mut() else {
            error!("image data not found for chunk {chunk:?}");
            return;
        };

        data.clear();
        for (offset, tile) in chunk.tiles() {
            for packed in pack(offset, tile) {
                let bytes = bytemuck::bytes_of(&packed);
                data.extend_from_slice(bytes);
            }
        }
    }
}

fn pack_ground_tile(offset: TileChunkOffset, _tile: TileData) -> [PackedTileData; 2] {
    let (left, right) = dirt_sprite_offsets(offset);

    [
        PackedTileData::new(left, 0, false),
        PackedTileData::new(right, 0, false),
    ]
}

fn pack_wall_tile(_: TileChunkOffset, tile: TileData) -> [PackedTileData; 2] {
    let right = wall::sprite_offset(tile.kind(), tile.wall_adjacency(), tile.door_adjacency());
    let left = wall::sprite_offset(
        tile.kind(),
        tile.wall_adjacency().flip_x(),
        tile.door_adjacency().flip_x(),
    );

    let depth = if tile.kind() == TileKind::Wall {
        0
    } else {
        (WALL_TOP_DEPTH - WALL_BASE_DEPTH) as u16
    };

    [
        PackedTileData::new(left, depth, true),
        PackedTileData::new(right, depth, false),
    ]
}

fn chunk_transform(position: TileChunkPosition) -> Transform {
    Transform::from_xyz(
        chunk_coord_transform(position.x()),
        chunk_coord_transform(position.y()),
        0.0,
    )
}

fn chunk_coord_transform(d: i16) -> f32 {
    d as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0
}

pub fn dirt_sprite_offsets(position: TileChunkOffset) -> (u16, u16) {
    let (x, y) = (position.x() * 2, position.y());

    let y = SPRITE_CHUNK_SIZE - 1 - y.rem_euclid(SPRITE_CHUNK_SIZE);
    let x1 = x.rem_euclid(SPRITE_CHUNK_SIZE * 2);
    let x2 = (x + 1).rem_euclid(SPRITE_CHUNK_SIZE * 2);

    (
        DIRT_OFFSET + y * SPRITE_CHUNK_SIZE * 2 + x1,
        DIRT_OFFSET + y * SPRITE_CHUNK_SIZE * 2 + x2,
    )
}
