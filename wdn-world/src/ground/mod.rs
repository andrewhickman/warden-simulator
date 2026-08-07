use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use wdn_physics::{
    PhysicsSystems,
    tile::{material::TileKind, storage::TileChunk},
};

use crate::{
    WorldSystems,
    path::{
        region::{AddedRegions, Region, regions_added},
        section::TileChunkSections,
    },
};

pub const OUTSIDE_GROUND_ID: u16 = 0;
pub const INSIDE_GROUND_ID: u16 = 4;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            WorldSystems::UpdateGround
                .after(WorldSystems::UpdateRegions)
                .ambiguous_with(WorldSystems::UpdateFlowFields)
                .ambiguous_with(PhysicsSystems::Collisions)
                .ambiguous_with(WorldSystems::ApplyPawnActions),
        );

        app.add_systems(
            FixedUpdate,
            update_ground
                .run_if(regions_added)
                .in_set(WorldSystems::UpdateGround),
        );
    }
}

pub fn update_ground(
    regions: Query<&Region>,
    mut chunks: Query<(&mut TileChunk, &TileChunkSections)>,
    added_regions: Res<AddedRegions>,
) {
    regions.iter_many(added_regions.iter()).for_each(|region| {
        let ground_id = if region.outside() {
            OUTSIDE_GROUND_ID
        } else {
            INSIDE_GROUND_ID
        };

        for (chunk_id, section_id) in region.sections() {
            let (mut chunk, sections) = chunks.get_mut(chunk_id).unwrap();
            let section = sections.section(section_id.chunk_offset());

            for &tile in section.tiles() {
                let material = chunk.get(tile).material();
                debug_assert!(matches!(
                    material.kind(),
                    TileKind::Empty | TileKind::Stairs
                ));
                if material.id() & INSIDE_GROUND_ID != ground_id {
                    chunk.set_material(tile, material.with_id(material.id() ^ INSIDE_GROUND_ID));
                }
            }
        }
    });
}
