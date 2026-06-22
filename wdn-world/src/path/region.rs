use std::{collections::VecDeque, ops::Index, u32};

use bevy_ecs::{
    entity::{EntityHashSet, hash_set},
    prelude::*,
};
use bevy_log::error;
use bevy_platform::collections::HashMap;
use smallvec::SmallVec;
use wdn_physics::tile::{
    adjacency::Adjacency,
    index::TileIndex,
    material::{TileMaterial, TileMoveSpeed},
    position::{TileLayerOffset, TilePosition},
    storage::{TileChunk, TileData, TileMap},
};

use crate::path::{
    flow::{AddedFlowFields, FlowField},
    section::{TileChunkSectionChanges, TileChunkSections},
};

#[derive(Component)]
#[require(RegionTiles)]
pub struct Region {
    layer: Entity,
    sections: SmallVec<[(Entity, TileLayerOffset); 2]>,
}

#[derive(Component, Default)]
pub struct RegionTiles {
    tiles: Vec<RegionTile>,
    tile_index: HashMap<TileLayerOffset, RegionTileIndex>,
    doors: Vec<RegionDoor>,
}

pub type RegionTileIndex = u32;

#[derive(Debug, Clone, Copy)]
pub struct RegionTile {
    position: TileLayerOffset,
    move_speed: TileMoveSpeed,
    material: TileMaterial,
    adjacency: Adjacency,
    north: RegionTileIndex,
    east: RegionTileIndex,
    south: RegionTileIndex,
    west: RegionTileIndex,
}

#[derive(Clone, Copy, Debug)]
pub struct RegionDoor {
    index: RegionTileIndex,
    position: TileLayerOffset,
    door: Entity,
    flow_field: Entity,
    adjacency: Adjacency,
}

#[derive(Default, Resource)]
pub struct AddedRegions {
    added_regions: EntityHashSet,
}

pub fn update_regions(
    mut commands: Commands,
    regions: Query<&Region>,
    chunks: Query<(&TileChunk, &TileChunkSections)>,
    mut changes: ResMut<TileChunkSectionChanges>,
    map: Res<TileMap>,
    mut queue: Local<VecDeque<(Entity, TilePosition)>>,
    mut added_regions: ResMut<AddedRegions>,
) -> Result {
    let TileChunkSectionChanges {
        ref mut removed_sections,
        ref mut invalid_sections,
        ref mut invalid_regions,
    } = *changes;

    for &region in invalid_regions.iter() {
        let region = regions.get(region)?;
        for (chunk_id, section) in region.sections() {
            if !removed_sections.contains(&section) {
                invalid_sections.insert(section, chunk_id);
            }
        }
    }

    removed_sections.clear();
    let visited_sections = removed_sections;

    for (&section, &chunk_id) in invalid_sections.iter() {
        if !visited_sections.insert(section) {
            continue;
        }

        debug_assert!(queue.is_empty());
        queue.push_back((chunk_id, section));

        let mut region_sections = SmallVec::new();

        while let Some((current_chunk_id, current_section)) = queue.pop_front() {
            let (current_chunk, current_chunk_sections) = chunks.get(current_chunk_id)?;
            let current_section_data =
                current_chunk_sections.section(current_section.chunk_offset());

            if !invalid_sections.contains_key(&current_section) {
                invalid_regions.insert(current_section_data.region());
            }

            region_sections.push((current_chunk_id, current_section.layer_offset()));

            current_section_data.visit_neighbors(current_chunk, |neighbor| {
                if let Some(neighbor_chunk_id) = map.get(neighbor.chunk_position()) {
                    let (_, neighbor_chunk_sections) = chunks.get(neighbor_chunk_id)?;
                    let neighbor_section_offset = neighbor_chunk_sections
                        .section_id(neighbor.chunk_offset())
                        .ok_or("neighbor section not found")?;
                    let neighbor_section =
                        TilePosition::from((neighbor.chunk_position(), neighbor_section_offset));
                    if visited_sections.insert(neighbor_section) {
                        queue.push_back((neighbor_chunk_id, neighbor_section));
                    }
                }

                Ok(())
            })?;
        }

        let region = commands
            .spawn((
                Region {
                    layer: section.layer(),
                    sections: region_sections,
                },
                ChildOf(section.layer()),
            ))
            .id();
        added_regions.insert(region);
    }

    for &region in invalid_regions.iter() {
        commands.entity(region).try_despawn();
    }

    invalid_regions.clear();
    invalid_sections.clear();
    visited_sections.clear();
    Ok(())
}

pub fn update_region_tiles(
    mut regions: Query<(&Region, &mut RegionTiles)>,
    chunks: Query<(&TileChunk, &TileChunkSections)>,
    added_regions: Res<AddedRegions>,
) {
    regions
        .par_iter_many_unique_mut(added_regions.iter())
        .for_each(|(region, mut region_tiles)| {
            for (chunk_id, section_offset) in region.sections() {
                let (chunk, chunk_sections) = chunks.get(chunk_id).unwrap();
                let section = chunk_sections.section(section_offset.chunk_offset());

                region_tiles.reserve(section.size(), 8);

                for &tile_offset in section.tiles() {
                    let tile = chunk.get(tile_offset);
                    let position = TileLayerOffset::from((chunk.position(), tile_offset));

                    let index = region_tiles.insert_tile(position, tile);

                    let door_adjacency = tile.door_adjacency();
                    if !door_adjacency.is_empty() {
                        region_tiles.insert_doors(position, index, door_adjacency);
                    }
                }
            }
        });
}

pub fn update_region_doors(
    mut commands: Commands,
    mut regions: Query<(Entity, &Region, &mut RegionTiles)>,
    index: Res<TileIndex>,
    added_regions: Res<AddedRegions>,
    mut added_flow_fields: ResMut<AddedFlowFields>,
) {
    regions.iter_many_unique_mut(added_regions.iter()).for_each(
        |(region_id, region, mut region_tiles)| {
            let RegionTiles {
                ref mut tiles,
                ref mut doors,
                ..
            } = *region_tiles;

            for door in doors {
                let door_position = tiles[door.index as usize].position();
                let door_adjacency = tiles[door.index as usize].adjacency();

                let Some(door_id) =
                    index.get_tile(TilePosition::from((region.layer(), door_position)))
                else {
                    error!("door tile not found at position {:?}", door_position);
                    continue;
                };

                door.adjacency = door_adjacency;
                door.door = door_id;
                door.flow_field = commands
                    .spawn((
                        ChildOf(region_id),
                        FlowField::new(
                            TilePosition::from((region.layer(), door_position)),
                            door.index,
                            door_adjacency,
                        ),
                    ))
                    .id();

                added_flow_fields.insert(door.flow_field);
            }
        },
    );
}

pub fn regions_added(changes: Res<AddedRegions>) -> bool {
    changes.has_regions()
}

pub fn clear_added_regions(mut changes: ResMut<AddedRegions>) {
    changes.clear();
}

pub fn on_add_region(
    trigger: On<Add, Region>,
    regions: Query<&Region>,
    mut chunks: Query<&mut TileChunkSections>,
) -> Result {
    let region = regions.get(trigger.entity)?;
    for &(chunk_id, section) in &region.sections {
        chunks
            .get_mut(chunk_id)?
            .section_mut(section.chunk_offset())
            .set_region(trigger.entity);
    }

    Ok(())
}

impl Region {
    pub fn layer(&self) -> Entity {
        self.layer
    }

    pub fn sections(&self) -> impl Iterator<Item = (Entity, TilePosition)> {
        self.sections
            .iter()
            .map(|&(chunk_id, position)| (chunk_id, TilePosition::from((self.layer, position))))
    }
}

impl RegionTiles {
    pub fn tiles(&self) -> impl Iterator<Item = (RegionTileIndex, &RegionTile)> {
        self.tiles
            .iter()
            .enumerate()
            .map(|(index, tile)| (index as RegionTileIndex, tile))
    }

    pub fn doors(&self) -> &[RegionDoor] {
        &self.doors
    }

    pub fn get_tile_index(&self, offset: TileLayerOffset) -> Option<RegionTileIndex> {
        self.tile_index.get(&offset).copied()
    }

    pub fn size(&self) -> usize {
        self.tiles.len()
    }

    pub fn door_count(&self) -> usize {
        self.doors.len()
    }

    fn reserve(&mut self, empty: usize, doors: usize) {
        self.tiles.reserve(empty + doors);
        self.tile_index.reserve(empty + doors);
        self.doors.reserve(doors);
    }

    fn insert_tile(&mut self, position: TileLayerOffset, tile: &TileData) -> RegionTileIndex {
        let index = self.tiles.len() as RegionTileIndex;

        let adjacency = tile.adjacency().walls().complement();

        self.tile_index.insert(position, index);
        self.tiles.push(RegionTile {
            position,
            adjacency,
            material: tile.material(),
            move_speed: tile.move_speed(),
            north: u32::MAX,
            east: u32::MAX,
            south: u32::MAX,
            west: u32::MAX,
        });

        if adjacency.contains(Adjacency::NORTH)
            && let Some(&north_index) = self.tile_index.get(&position.north())
        {
            self.tiles[index as usize].north = north_index;
            self.tiles[north_index as usize].south = index;
        }

        if adjacency.contains(Adjacency::EAST)
            && let Some(&east_index) = self.tile_index.get(&position.east())
        {
            self.tiles[index as usize].east = east_index;
            self.tiles[east_index as usize].west = index;
        }

        if adjacency.contains(Adjacency::SOUTH)
            && let Some(&south_index) = self.tile_index.get(&position.south())
        {
            self.tiles[index as usize].south = south_index;
            self.tiles[south_index as usize].north = index;
        }

        if adjacency.contains(Adjacency::WEST)
            && let Some(&west_index) = self.tile_index.get(&position.west())
        {
            self.tiles[index as usize].west = west_index;
            self.tiles[west_index as usize].east = index;
        }

        index
    }

    fn insert_doors(
        &mut self,
        position: TileLayerOffset,
        index: RegionTileIndex,
        door_adjacency: Adjacency,
    ) {
        if door_adjacency.contains(Adjacency::NORTH) {
            let door_position = position.north();
            let door_index = self.insert_door(door_position);
            self.tiles[index as usize].north = door_index;
            self.tiles[door_index as usize].south = index;
            self.tiles[door_index as usize]
                .adjacency
                .insert(Adjacency::SOUTH);
        }

        if door_adjacency.contains(Adjacency::EAST) {
            let door_position = position.east();
            let door_index = self.insert_door(door_position);
            self.tiles[index as usize].east = door_index;
            self.tiles[door_index as usize].west = index;
            self.tiles[door_index as usize]
                .adjacency
                .insert(Adjacency::WEST);
        }

        if door_adjacency.contains(Adjacency::SOUTH) {
            let door_position = position.south();
            let door_index = self.insert_door(door_position);
            self.tiles[index as usize].south = door_index;
            self.tiles[door_index as usize].north = index;
            self.tiles[door_index as usize]
                .adjacency
                .insert(Adjacency::NORTH);
        }

        if door_adjacency.contains(Adjacency::WEST) {
            let door_position = position.west();
            let door_index = self.insert_door(door_position);
            self.tiles[index as usize].west = door_index;
            self.tiles[door_index as usize].east = index;
            self.tiles[door_index as usize]
                .adjacency
                .insert(Adjacency::EAST);
        }
    }

    fn insert_door(&mut self, position: TileLayerOffset) -> RegionTileIndex {
        *self.tile_index.entry(position).or_insert_with(|| {
            let index = self.tiles.len() as RegionTileIndex;
            self.doors.push(RegionDoor {
                index,
                position,
                door: Entity::PLACEHOLDER,
                flow_field: Entity::PLACEHOLDER,
                adjacency: Adjacency::NONE,
            });
            self.tiles.push(RegionTile {
                position,
                adjacency: Adjacency::NONE,
                material: TileMaterial::Door,
                move_speed: TileMoveSpeed::Medium,
                north: u32::MAX,
                east: u32::MAX,
                south: u32::MAX,
                west: u32::MAX,
            });
            index
        })
    }
}

impl Index<RegionTileIndex> for RegionTiles {
    type Output = RegionTile;

    fn index(&self, index: RegionTileIndex) -> &Self::Output {
        &self.tiles[index as usize]
    }
}

impl RegionTile {
    pub fn position(&self) -> TileLayerOffset {
        self.position
    }

    pub fn adjacency(&self) -> Adjacency {
        self.adjacency
    }

    pub fn material(&self) -> TileMaterial {
        self.material
    }

    pub fn is_door(&self) -> bool {
        self.material == TileMaterial::Door
    }

    pub fn move_speed(&self) -> TileMoveSpeed {
        self.move_speed
    }

    pub fn north(&self) -> Option<RegionTileIndex> {
        if self.north == u32::MAX {
            None
        } else {
            Some(self.north)
        }
    }

    pub fn east(&self) -> Option<RegionTileIndex> {
        if self.east == u32::MAX {
            None
        } else {
            Some(self.east)
        }
    }

    pub fn south(&self) -> Option<RegionTileIndex> {
        if self.south == u32::MAX {
            None
        } else {
            Some(self.south)
        }
    }

    pub fn west(&self) -> Option<RegionTileIndex> {
        if self.west == u32::MAX {
            None
        } else {
            Some(self.west)
        }
    }
}

impl RegionDoor {
    pub fn index(&self) -> RegionTileIndex {
        self.index
    }

    pub fn position(&self) -> TileLayerOffset {
        self.position
    }

    pub fn door(&self) -> Entity {
        self.door
    }

    pub fn adjacency(&self) -> Adjacency {
        self.adjacency
    }

    pub fn flow_field(&self) -> Entity {
        self.flow_field
    }
}

impl AddedRegions {
    pub fn insert(&mut self, region: Entity) {
        self.added_regions.insert(region);
    }

    pub fn clear(&mut self) {
        self.added_regions.clear();
    }

    pub fn has_regions(&self) -> bool {
        !self.added_regions.is_empty()
    }

    pub fn iter(&'_ self) -> hash_set::Iter<'_> {
        self.added_regions.iter()
    }
}
