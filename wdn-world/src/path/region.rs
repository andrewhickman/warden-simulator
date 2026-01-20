use std::collections::VecDeque;

use bevy_ecs::{
    entity::{EntityHashMap, EntityHashSet},
    lifecycle::HookContext,
    prelude::*,
    world::DeferredWorld,
};
use bevy_platform::collections::{HashMap, HashSet, hash_map};
use wdn_physics::tile::{
    CHUNK_SIZE, TileChunkOffset, TileChunkPosition,
    storage::{TileChunk, TileMap},
};

#[derive(Component)]
#[component(immutable, on_add = LayerRegion::on_add)]
pub struct LayerRegion {
    // todo key by TileChunkPosition?
    sections: EntityHashMap<Vec<TileChunkOffset>>,
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
    removed_sections: HashSet<(Entity, TileChunkOffset)>,
    invalid_sections: HashMap<(Entity, TileChunkOffset), TileChunkPosition>,
    invalid_regions: EntityHashSet,
    queue: VecDeque<(Entity, TileChunkOffset, TileChunkPosition)>,
}

pub fn update_tile_chunk_sections(
    mut commands: Commands,
    regions: Query<&LayerRegion>,
    mut chunks: ParamSet<(
        Query<(Entity, &TileChunk, &mut TileChunkSections), Changed<TileChunk>>,
        Query<&TileChunk>,
        Query<&TileChunkSections>,
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

    // TODO parallelize this?
    chunks
        .p0()
        .iter_mut()
        .for_each(|(chunk_id, chunk, mut chunk_sections)| {
            let mut parents = TileChunkSectionParents::default();

            for offset in TileChunkOffset::iter() {
                let tile = chunk.get(offset);

                if tile.is_solid() {
                    parents.remove(offset);
                } else {
                    parents.insert(offset);
                }
            }

            // TODO surely this can be removed?
            let mut changed_sections = HashSet::new();

            for offset in TileChunkOffset::iter() {
                let prev_section = chunk_sections.parents.get(offset);
                let section = parents.find(offset);

                match (prev_section, section) {
                    (Some(prev_parent), Some(parent)) if parent != prev_parent => {
                        changed_sections.insert(prev_parent);
                        changed_sections.insert(parent);
                    }
                    (Some(prev_parent), None) => {
                        changed_sections.insert(prev_parent);
                    }
                    (None, Some(parent)) => {
                        changed_sections.insert(parent);
                    }
                    (Some(_), Some(_)) | (None, None) => {}
                }
            }

            if changed_sections.is_empty() {
                return;
            }

            for &section in &changed_sections {
                if let Some(region) = chunk_sections.sections.remove(&section) {
                    removed_sections.insert((chunk_id, section));
                    invalid_regions.insert(region.region);
                }
            }

            for offset in TileChunkOffset::iter() {
                // let prev_section = chunk_sections.parents.get(offset);
                let section = parents.get(offset);

                if let Some(section) = section
                    && changed_sections.contains(&section)
                {
                    match chunk_sections.sections.entry(section) {
                        hash_map::Entry::Vacant(entry) => {
                            entry.insert(TileChunkSection::default()).insert(offset);
                            invalid_sections.insert((chunk_id, section), chunk.position());
                        }
                        hash_map::Entry::Occupied(entry) => entry.into_mut().insert(offset),
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
        for (chunk, sections) in region.sections() {
            let position = chunks.p1().get(chunk)?.position();
            for &section in sections {
                if !removed_sections.contains(&(chunk, section)) {
                    invalid_sections.insert((chunk, section), position);
                }
            }
        }
    }

    removed_sections.clear();
    let visited_sections = removed_sections;

    let chunks = chunks.p2();
    for (&(chunk, section), &position) in invalid_sections.iter() {
        if !visited_sections.insert((chunk, section)) {
            continue;
        }

        debug_assert!(queue.is_empty());
        queue.push_back((chunk, section, position));

        let mut region_sections = EntityHashMap::<Vec<TileChunkOffset>>::new();

        while let Some((current_chunk_id, current_section, current_position)) = queue.pop_front() {
            let current_chunk = chunks.get(current_chunk_id)?;
            let current_section_data = current_chunk.section(current_section);

            if !invalid_sections.contains_key(&(current_chunk_id, current_section)) {
                // todo remove
                debug_assert_ne!(current_section_data.region, Entity::PLACEHOLDER);
                invalid_regions.insert(current_section_data.region);
            }

            region_sections
                .entry(current_chunk_id)
                .or_default()
                .push(current_section);

            current_section_data.for_each_neighbor(
                current_position,
                |neighbor_position, neighbor_offset| {
                    if let Some(neighbor_chunk) = map.get(neighbor_position) {
                        let neighbor_chunk_sections = chunks.get(neighbor_chunk)?;
                        if let Some(neighbor_section) =
                            neighbor_chunk_sections.parents.get(neighbor_offset)
                        {
                            if visited_sections.insert((neighbor_chunk, neighbor_section)) {
                                queue.push_back((
                                    neighbor_chunk,
                                    neighbor_section,
                                    neighbor_position,
                                ));
                            }
                        }
                    }

                    Ok(())
                },
            )?;
        }

        commands.spawn((
            LayerRegion {
                sections: region_sections,
            },
            ChildOf(position.layer()),
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
    pub fn sections(&self) -> impl Iterator<Item = (Entity, &[TileChunkOffset])> {
        self.sections
            .iter()
            .map(|(&chunk, sections)| (chunk, sections.as_slice()))
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        // TODO avoid the clone
        let chunks = world
            .get::<LayerRegion>(context.entity)
            .unwrap()
            .sections
            .clone();
        for (&chunk, sections) in &chunks {
            let mut chunk = world.get_mut::<TileChunkSections>(chunk).unwrap();
            for &section in sections {
                chunk.section_mut(section).region = context.entity;
            }
        }
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
        position: TileChunkPosition,
        mut f: impl FnMut(TileChunkPosition, TileChunkOffset) -> Result,
    ) -> Result {
        self.edges().iter().try_for_each(|&offset| {
            if offset.x() == 0 {
                f(
                    TileChunkPosition::new(position.layer(), position.x() - 1, position.y()),
                    TileChunkOffset::new((CHUNK_SIZE - 1) as u16, offset.y()),
                )?;
            } else if offset.x() == (CHUNK_SIZE - 1) as u16 {
                f(
                    TileChunkPosition::new(position.layer(), position.x() + 1, position.y()),
                    TileChunkOffset::new(0, offset.y()),
                )?;
            }

            if offset.y() == 0 {
                f(
                    TileChunkPosition::new(position.layer(), position.x(), position.y() - 1),
                    TileChunkOffset::new(offset.x(), (CHUNK_SIZE - 1) as u16),
                )?;
            } else if offset.y() == (CHUNK_SIZE - 1) as u16 {
                f(
                    TileChunkPosition::new(position.layer(), position.x(), position.y() + 1),
                    TileChunkOffset::new(offset.x(), 0),
                )?;
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

    fn find_north(&mut self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        self.find(offset.north()?)
    }

    fn find_west(&mut self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        self.find(offset.west()?)
    }

    fn insert(&mut self, offset: TileChunkOffset) {
        // TODO normalize here

        match (self.find_west(offset), self.find_north(offset)) {
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
