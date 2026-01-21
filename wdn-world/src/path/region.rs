use std::{collections::VecDeque, mem::take};

use bevy_ecs::{entity::EntityHashSet, lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_platform::collections::{HashMap, HashSet, hash_map};
use wdn_physics::tile::{
    CHUNK_SIZE, TileChunkOffset, TilePosition,
    storage::{TileChunk, TileMap, TileOccupancy},
};

#[derive(Component)]
#[component(on_add = LayerRegion::on_add)]
pub struct LayerRegion {
    sections: Vec<(Entity, TilePosition)>,
}

#[derive(Component, Default, Debug)]
pub struct TileChunkSections {
    sections: HashMap<TileChunkOffset, TileChunkSection>,
    parents: TileChunkSectionParents,
}

#[derive(Debug)]
struct TileChunkSection {
    tiles: Vec<TileChunkOffset>,
    edges: usize,
    region: Entity,
}

#[derive(Debug)]
struct TileChunkSectionParents {
    parents: [Option<TileChunkOffset>; CHUNK_SIZE * CHUNK_SIZE],
}

#[derive(Default)]
pub struct TileChunkSectionChanges {
    removed_sections: HashSet<TilePosition>,
    invalid_sections: HashMap<TilePosition, Entity>,
    invalid_regions: EntityHashSet,
    queue: VecDeque<(Entity, TilePosition)>,
}

pub fn update_tile_chunk_sections(
    mut commands: Commands,
    regions: Query<&LayerRegion>,
    mut chunks: ParamSet<(
        Query<(Entity, &TileChunk, &mut TileChunkSections), Changed<TileChunk>>,
        Query<(&TileChunk, &TileChunkSections)>,
    )>,
    mut changes: Local<TileChunkSectionChanges>,
    map: Res<TileMap>,
) -> Result {
    let TileChunkSectionChanges {
        ref mut removed_sections,
        ref mut invalid_sections,
        ref mut invalid_regions,
        ref mut queue,
    } = *changes;

    chunks
        .p0()
        .iter_mut()
        .for_each(|(chunk_id, chunk, mut chunk_sections)| {
            let position = chunk.position();
            let mut parents = TileChunkSectionParents::default();

            for offset in TileChunkOffset::iter() {
                let tile = chunk.get(offset);

                if tile.is_solid() {
                    parents.remove(offset);
                } else {
                    parents.insert(offset);
                }
            }

            for offset in TileChunkOffset::iter() {
                let prev_parent = chunk_sections.parents.get(offset);
                let parent = parents.find(offset);

                if prev_parent != parent {
                    for parent in [prev_parent, parent] {
                        if let Some(parent) = parent {
                            if let Some(region) = chunk_sections.sections.remove(&parent) {
                                removed_sections.insert(TilePosition::from((position, parent)));
                                invalid_regions.insert(region.region);
                            }
                        }
                    }
                }
            }

            for offset in TileChunkOffset::iter() {
                let parent = parents.get(offset);

                if let Some(parent) = parent {
                    match chunk_sections.sections.entry(parent) {
                        hash_map::Entry::Vacant(entry) => {
                            entry.insert(TileChunkSection::default()).insert(offset);
                            invalid_sections
                                .insert(TilePosition::from((position, parent)), chunk_id);
                        }
                        hash_map::Entry::Occupied(entry) => {
                            if invalid_sections
                                .contains_key(&TilePosition::from((position, parent)))
                            {
                                entry.into_mut().insert(offset);
                            }
                        }
                    }
                }
            }

            chunk_sections.parents = parents;
        });

    if removed_sections.is_empty() && invalid_sections.is_empty() {
        debug_assert!(invalid_regions.is_empty());
        return Ok(());
    }

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

    let chunks = chunks.p1();
    for (&section, &chunk_id) in invalid_sections.iter() {
        if !visited_sections.insert(section) {
            continue;
        }

        debug_assert!(queue.is_empty());
        queue.push_back((chunk_id, section));

        let mut region_sections = Vec::new();

        while let Some((current_chunk_id, current_section)) = queue.pop_front() {
            let (current_chunk, current_chunk_sections) = chunks.get(current_chunk_id)?;
            let current_section_data =
                current_chunk_sections.section(current_section.chunk_offset());

            if !invalid_sections.contains_key(&current_section) {
                invalid_regions.insert(current_section_data.region);
            }

            region_sections.push((current_chunk_id, current_section));

            current_section_data.for_each_neighbor(current_chunk, |neighbor| {
                if let Some(neighbor_chunk_id) = map.get(neighbor.chunk_position()) {
                    let (_, neighbor_chunk_sections) = chunks.get(neighbor_chunk_id)?;
                    let neighbor_section_offset = neighbor_chunk_sections
                        .parents
                        .get(neighbor.chunk_offset())
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

        commands.spawn((
            LayerRegion {
                sections: region_sections,
            },
            ChildOf(section.layer()),
        ));
    }

    for &layer in invalid_regions.iter() {
        commands.entity(layer).try_despawn();
    }

    invalid_regions.clear();
    invalid_sections.clear();
    visited_sections.clear();
    Ok(())
}

impl LayerRegion {
    pub fn sections(&self) -> impl Iterator<Item = (Entity, TilePosition)> {
        self.sections.iter().copied()
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let sections = take(
            &mut world
                .get_mut::<LayerRegion>(context.entity)
                .unwrap()
                .sections,
        );

        for &(chunk_id, section) in &sections {
            world
                .get_mut::<TileChunkSections>(chunk_id)
                .unwrap()
                .section_mut(section.chunk_offset())
                .region = context.entity;
        }

        world
            .get_mut::<LayerRegion>(context.entity)
            .unwrap()
            .sections = sections;
    }
}

impl TileChunkSections {
    pub fn region(&self, offset: TileChunkOffset) -> Option<Entity> {
        let section = self.parents.get(offset)?;
        Some(self.sections[&section].region)
    }

    pub fn tiles(&self, offset: TileChunkOffset) -> Option<&[TileChunkOffset]> {
        let section = self.parents.get(offset)?;
        Some(&self.sections[&section].tiles)
    }

    pub fn sections(&self) -> impl Iterator<Item = TileChunkOffset> + '_ {
        self.sections.keys().copied()
    }

    fn section(&self, offset: TileChunkOffset) -> &TileChunkSection {
        &self.sections[&offset]
    }

    fn section_mut(&mut self, offset: TileChunkOffset) -> &mut TileChunkSection {
        self.sections.get_mut(&offset).unwrap()
    }
}

impl TileChunkSection {
    fn insert(&mut self, offset: TileChunkOffset) {
        let index = self.tiles.len();
        self.tiles.push(offset);
        if offset.on_chunk_edge() {
            self.tiles.swap(self.edges, index);
            self.edges += 1;
        }
    }

    fn edges(&self) -> &[TileChunkOffset] {
        &self.tiles[..self.edges]
    }

    fn for_each_neighbor(
        &self,
        chunk: &TileChunk,
        mut f: impl FnMut(TilePosition) -> Result,
    ) -> Result {
        let chunk_position = chunk.position();
        self.edges().iter().try_for_each(|&offset| {
            let edge = CHUNK_SIZE as u16 - 1;
            let occupancy = chunk.get(offset).occupancy();
            let position = TilePosition::from((chunk_position, offset));

            if offset.x() == 0 && !occupancy.contains(TileOccupancy::WEST) {
                f(position.west())?;
            } else if offset.x() == edge && !occupancy.contains(TileOccupancy::EAST) {
                f(position.east())?;
            }

            if offset.y() == 0 && !occupancy.contains(TileOccupancy::SOUTH) {
                f(position.south())?;
            } else if offset.y() == edge && !occupancy.contains(TileOccupancy::NORTH) {
                f(position.north())?;
            }

            Ok(())
        })
    }
}

impl Default for TileChunkSection {
    fn default() -> Self {
        Self {
            tiles: Vec::new(),
            edges: 0,
            region: Entity::PLACEHOLDER,
        }
    }
}

impl Default for TileChunkSectionParents {
    fn default() -> Self {
        Self {
            parents: [None; CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl TileChunkSectionParents {
    fn get(&self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        let parent = self.parents[offset.index()]?;
        debug_assert_eq!(
            self.parents[parent.index()],
            Some(parent),
            "Parents should normalized"
        );
        Some(parent)
    }

    fn find(&mut self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        let mut parent = self.parents[offset.index()]?;

        if parent != offset && self.parents[parent.index()] != Some(parent) {
            parent = self.find(parent).expect("parent should exist");
            self.parents[offset.index()] = Some(parent);
        }

        Some(parent)
    }

    fn find_south(&mut self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        self.find(offset.south()?)
    }

    fn find_west(&mut self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        self.find(offset.west()?)
    }

    fn insert(&mut self, offset: TileChunkOffset) {
        match (self.find_west(offset), self.find_south(offset)) {
            (Some(west_parent), Some(north_parent)) => {
                if west_parent != north_parent {
                    self.parents[west_parent.index()] = Some(north_parent);
                }
                self.parents[offset.index()] = Some(north_parent);
            }
            (Some(west_parent), None) => {
                self.parents[offset.index()] = Some(west_parent);
            }
            (None, Some(north_parent)) => {
                self.parents[offset.index()] = Some(north_parent);
            }
            (None, None) => {
                self.parents[offset.index()] = Some(offset);
            }
        }
    }

    fn remove(&mut self, offset: TileChunkOffset) {
        self.parents[offset.index()] = None;
    }
}
