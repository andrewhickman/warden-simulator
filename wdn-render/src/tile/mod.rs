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

pub struct TilePlugin;

#[derive(Resource)]
pub struct TileChunkMesh(Handle<Mesh>);

#[derive(Clone, Component, Default, Debug)]
#[require(Transform, Visibility)]
pub struct TileChunkSprites {
    base: Handle<TileChunkMaterial>,
    mid: Handle<TileChunkMaterial>,
    top: Handle<TileChunkMaterial>,
}

#[derive(SystemParam)]
pub struct TileChunkSpriteParam<'w, 's> {
    commands: Commands<'w, 's>,
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
    assets: Res<AssetHandles>,
    mut chunks: Query<
        (Entity, &TileChunk, &mut Transform, &mut TileChunkSprites),
        (Changed<TileChunk>, Without<Position>, Without<Door>),
    >,
) {
    chunks
        .iter_mut()
        .for_each(|(id, chunk, mut transform, mut sprites)| {
            let position = chunk.position();
            if sprites.is_added() {
                *transform = chunk_transform(position);

                sprites.base = param.spawn_chunk_material(id, assets.base_tileset(), GROUND_DEPTH);
                sprites.mid =
                    param.spawn_chunk_material(id, assets.wall_tileset(), WALL_BASE_DEPTH);
                sprites.top = param.spawn_chunk_material(id, assets.wall_tileset(), WALL_TOP_DEPTH);
            }

            param.update_chunk_material(sprites.base.id(), chunk, pack_base_tile);
            param.update_chunk_material(sprites.mid.id(), chunk, pack_mid_tile);
            param.update_chunk_material(sprites.top.id(), chunk, pack_top_tile);
        });
}

impl TileChunkSpriteParam<'_, '_> {
    fn spawn_chunk_material(
        &mut self,
        id: Entity,
        tileset: Handle<Image>,
        depth: f32,
    ) -> Handle<TileChunkMaterial> {
        let tile_data = self.images.add(make_tile_chunk_image());
        let material = self.materials.add(TileChunkMaterial { tileset, tile_data });

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

fn pack_base_tile(_: TileChunkOffset, tile: TileData) -> [PackedTileData; 2] {
    let offset = match tile.material().kind() {
        TileKind::Empty | TileKind::Stairs => 1,
        _ => 2,
    };

    [
        PackedTileData::new(offset, 0, false),
        PackedTileData::new(offset, 0, false),
    ]
}

fn pack_mid_tile(_: TileChunkOffset, tile: TileData) -> [PackedTileData; 2] {
    let (left, right) =
        wall::mid_offsets(tile.kind(), tile.wall_adjacency(), tile.door_adjacency());

    [
        PackedTileData::new(left, 0, true),
        PackedTileData::new(right, 0, false),
    ]
}

fn pack_top_tile(_: TileChunkOffset, tile: TileData) -> [PackedTileData; 2] {
    let (left, right) = wall::top_offset(tile.kind(), tile.wall_adjacency());

    [
        PackedTileData::new(left, 0, true),
        PackedTileData::new(right, 0, false),
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
