use std::collections::VecDeque;

use bevy_ecs::{
    entity::{EntityHashMap, EntityHashSet},
    lifecycle::HookContext,
    prelude::*,
    world::DeferredWorld,
};
use bevy_platform::collections::{HashMap, HashSet};
use parking_lot::Mutex;
use wdn_physics::tile::{
    CHUNK_SIZE, TileChunkOffset, TileChunkPosition,
    storage::{TileChunk, TileMap},
};

#[derive(Component)]
#[component(immutable, on_add = LayerRegion::on_add)]
pub struct LayerRegion {
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

#[derive(Message)]
pub struct TileChunkSectionsChanged {
    chunk: Entity,
    changed: HashSet<TileChunkOffset>,
}

pub fn update_tile_chunk_sections(
    mut chunks: Query<(Entity, &TileChunk, &mut TileChunkSections), Changed<TileChunk>>,
    events: MessageWriter<TileChunkSectionsChanged>,
) {
    let events = Mutex::new(events);

    chunks.par_iter_mut().for_each(|(id, chunk, mut sections)| {
        let mut parents = TileChunkSectionParents::default();

        for offset in TileChunkOffset::iter() {
            let tile = chunk.get(offset);

            if tile.is_solid() {
                parents.remove(offset);
            } else {
                parents.insert(offset);
            }
        }

        let mut changed_sections = HashSet::new();

        for offset in TileChunkOffset::iter() {
            let prev_parent = sections.parents.get(offset);
            let parent = parents.find(offset);

            match (prev_parent, parent) {
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

        if !changed_sections.is_empty() {
            sections.parents = parents;

            events.lock().write(TileChunkSectionsChanged {
                chunk: id,
                changed: changed_sections,
            });
        }
    });
}

pub fn update_layer_regions(
    mut commands: Commands,
    regions: Query<&LayerRegion>,
    mut chunks: Query<(&TileChunk, &mut TileChunkSections)>,
    mut events: MessageReader<TileChunkSectionsChanged>,
    map: Res<TileMap>,
) -> Result {
    if events.is_empty() {
        return Ok(());
    }

    // Sections which no longer exist. Their ids may overlap with some regions in `invalid_sections`
    let mut removed_sections = HashSet::new();
    // Sections in the queue to be processed
    let mut invalid_sections = HashSet::new();
    // Regions that need to be removed at the end
    let mut invalid_regions = EntityHashSet::new();

    // Mark invalid sections and regions

    for event in events.read() {
        if let Ok((_, mut chunk)) = chunks.get_mut(event.chunk) {
            for section in &event.changed {
                if let Some(region) = chunk.sections.remove(section) {
                    invalid_regions.insert(region.region);
                    removed_sections.insert((event.chunk, *section));
                }
            }

            for offset in TileChunkOffset::iter() {
                if let Some(section) = chunk.parents.get(offset)
                    && event.changed.contains(&section)
                {
                    chunk
                        .sections
                        .entry(section)
                        .or_insert_with(|| TileChunkSection::new(Entity::PLACEHOLDER))
                        .insert(offset);

                    invalid_sections.insert((event.chunk, section));
                }
            }
        }
    }

    for &region in &invalid_regions {
        if let Ok(region) = regions.get(region) {
            for (chunk, sections) in region.sections() {
                for &section in sections {
                    if !removed_sections.contains(&(chunk, section)) {
                        invalid_sections.insert((chunk, section));
                    }
                }
            }
        }
    }

    let invalid_sections = invalid_sections;
    drop(removed_sections);

    // flood fill regions from invalid sections

    let mut visited = HashSet::with_capacity(invalid_sections.len());
    let mut queue = VecDeque::with_capacity(CHUNK_SIZE);

    for &(chunk, section) in &invalid_sections {
        if !visited.insert((chunk, section)) {
            continue;
        }

        queue.clear();
        queue.push_back((chunk, section));

        let mut region_sections = EntityHashMap::<Vec<TileChunkOffset>>::new();

        while let Some((current_chunk_id, current_section)) = queue.pop_front() {
            let (position, current_chunk) = chunks.get(current_chunk_id)?;
            let current_section_data = current_chunk.section(current_section);

            if !invalid_sections.contains(&(current_chunk_id, current_section)) {
                debug_assert_ne!(current_section_data.region, Entity::PLACEHOLDER);
                invalid_regions.insert(current_section_data.region);
            }

            region_sections
                .entry(current_chunk_id)
                .or_default()
                .push(current_section);

            current_section_data.for_each_neighbor(
                position.position(),
                |neighbor_position, neighbor_offset| {
                    if let Some(neighbor_chunk) = map.get(neighbor_position) {
                        if let Ok((_, neighbor_chunk_sections)) = chunks.get(neighbor_chunk) {
                            if let Some(neighbor_section) =
                                neighbor_chunk_sections.parents.get(neighbor_offset)
                            {
                                if visited.insert((neighbor_chunk, neighbor_section)) {
                                    queue.push_back((neighbor_chunk, neighbor_section));
                                }
                            }
                        }
                    }
                },
            );
        }

        commands.spawn(LayerRegion {
            sections: region_sections,
        });
    }

    for layer in invalid_regions {
        commands.entity(layer).try_despawn();
    }

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

    fn section(&self, offset: TileChunkOffset) -> &TileChunkSection {
        &self.sections[&offset]
    }

    fn section_mut(&mut self, offset: TileChunkOffset) -> &mut TileChunkSection {
        self.sections.get_mut(&offset).unwrap()
    }
}

impl TileChunkSection {
    fn new(region: Entity) -> Self {
        Self {
            tiles: Vec::new(),
            edges: 0,
            region,
        }
    }

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
        mut f: impl FnMut(TileChunkPosition, TileChunkOffset),
    ) {
        self.edges().iter().for_each(|&offset| {
            if offset.x() == 0 {
                f(
                    TileChunkPosition::new(position.layer(), position.x() - 1, position.y()),
                    TileChunkOffset::new((CHUNK_SIZE - 1) as u16, offset.y()),
                );
            } else if offset.x() == (CHUNK_SIZE - 1) as u16 {
                f(
                    TileChunkPosition::new(position.layer(), position.x() + 1, position.y()),
                    TileChunkOffset::new(0, offset.y()),
                );
            }

            if offset.y() == 0 {
                f(
                    TileChunkPosition::new(position.layer(), position.x(), position.y() - 1),
                    TileChunkOffset::new(offset.x(), (CHUNK_SIZE - 1) as u16),
                );
            } else if offset.y() == (CHUNK_SIZE - 1) as u16 {
                f(
                    TileChunkPosition::new(position.layer(), position.x(), position.y() + 1),
                    TileChunkOffset::new(offset.x(), 0),
                );
            }
        });
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
    fn find_north(&mut self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        self.find(offset.north()?)
    }

    fn find_west(&mut self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        self.find(offset.west()?)
    }

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
            parent = self.find(parent).expect("Parent should exist");
            self.parents[offset.index()] = Some(parent);
        }

        Some(parent)
    }

    fn insert(&mut self, offset: TileChunkOffset) {
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
