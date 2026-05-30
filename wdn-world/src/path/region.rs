use core::fmt;
use std::{collections::VecDeque, mem::take};

use bevy_ecs::{entity::EntityHashSet, lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_platform::collections::{HashMap, HashSet, hash_map};
use wdn_physics::tile::{
    CHUNK_SIZE, CHUNK_SIZE_SQUARED,
    adjacency::{DoorAdjacency, WallAdjacency},
    material::TileMaterial,
    position::{TileChunkOffset, TileChunkPosition, TilePosition},
    storage::{TileChunk, TileMap},
};

use crate::path::map::LayerRegionMap;

#[derive(Component)]
#[require(LayerRegionMap)]
#[component(on_add = LayerRegion::on_add)]
pub struct LayerRegion {
    sections: Vec<(Entity, TilePosition)>,
}

#[derive(Component, Default, Debug)]
pub struct TileChunkSections {
    sections: HashMap<TileChunkOffset, TileChunkSection>,
    set: TileChunkDisjointSet,
}

#[derive(Debug)]
struct TileChunkSection {
    tiles: Vec<TileChunkOffset>,
    doors: HashMap<TilePosition, DoorAdjacency>,
    edges: usize,
    region: Entity,
}

#[derive(Debug)]
struct TileChunkDisjointSet {
    entries: [TileChunkDisjointSetEntry; CHUNK_SIZE_SQUARED],
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct TileChunkDisjointSetEntry(u16);

#[derive(Default, Resource)]
pub struct TileChunkSectionChanges {
    removed_sections: HashSet<TilePosition>,
    invalid_sections: HashMap<TilePosition, Entity>,
    invalid_regions: EntityHashSet,
}

pub fn update_chunk_sections(
    mut chunks: Query<(Entity, &TileChunk, &mut TileChunkSections), Changed<TileChunk>>,
    mut changes: ResMut<TileChunkSectionChanges>,
) -> Result {
    let TileChunkSectionChanges {
        ref mut removed_sections,
        ref mut invalid_sections,
        ref mut invalid_regions,
    } = *changes;

    chunks
        .iter_mut()
        .for_each(|(chunk_id, chunk, mut chunk_sections)| {
            let position = chunk.position();
            let mut set = TileChunkDisjointSet::default();

            for offset in TileChunkOffset::iter() {
                let tile = chunk.get(offset);

                if tile.material() == TileMaterial::Empty {
                    set.insert(offset, tile.door_adjacency());
                } else {
                    set.remove(offset);
                }
            }

            for offset in TileChunkOffset::iter() {
                set.find(offset);

                let prev_set_entry = chunk_sections.set.get(offset);
                let set_entry = set.get(offset);

                if prev_set_entry != set_entry {
                    for section_id in prev_set_entry.invalid_sections(set_entry) {
                        if let Some(section) = chunk_sections.sections.remove(&section_id) {
                            removed_sections.insert(TilePosition::from((position, section_id)));
                            invalid_regions.insert(section.region);
                        }
                    }
                }
            }

            for offset in TileChunkOffset::iter() {
                let set_entry = set.get(offset);

                if let Some(section_id) = set_entry.try_section() {
                    match chunk_sections.sections.entry(section_id) {
                        hash_map::Entry::Vacant(entry) => {
                            entry.insert(TileChunkSection::default()).insert(
                                position,
                                offset,
                                set_entry.door_adjacency(),
                            );
                            invalid_sections
                                .insert(TilePosition::from((position, section_id)), chunk_id);
                        }
                        hash_map::Entry::Occupied(entry) => {
                            if invalid_sections
                                .contains_key(&TilePosition::from((position, section_id)))
                            {
                                entry.into_mut().insert(
                                    position,
                                    offset,
                                    set_entry.door_adjacency(),
                                );
                            } else {
                                debug_assert!(entry.get().tiles.contains(&offset));
                            }
                        }
                    }
                }
            }

            chunk_sections.set = set;
        });

    Ok(())
}

pub fn chunk_sections_changed(changes: Res<TileChunkSectionChanges>) -> bool {
    if changes.removed_sections.is_empty() && changes.invalid_sections.is_empty() {
        debug_assert!(changes.invalid_regions.is_empty());
        false
    } else {
        true
    }
}

pub fn update_regions(
    mut commands: Commands,
    regions: Query<&LayerRegion>,
    chunks: Query<(&TileChunk, &TileChunkSections)>,
    mut changes: ResMut<TileChunkSectionChanges>,
    map: Res<TileMap>,
    mut queue: Local<VecDeque<(Entity, TilePosition)>>,
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
                        .set
                        .get_section(neighbor.chunk_offset())
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

    for &region in invalid_regions.iter() {
        commands.entity(region).try_despawn();
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
        let section = self.set.get_section(offset)?;
        Some(self.sections[&section].region)
    }

    pub fn tiles(&self, offset: TileChunkOffset) -> Option<&[TileChunkOffset]> {
        let section = self.set.get_section(offset)?;
        Some(&self.sections[&section].tiles)
    }

    pub fn doors(
        &self,
        offset: TileChunkOffset,
    ) -> Option<impl Iterator<Item = (TilePosition, DoorAdjacency)> + '_> {
        let section = self.set.get_section(offset)?;
        Some(
            self.sections[&section]
                .doors
                .iter()
                .map(|(&pos, &adj)| (pos, adj)),
        )
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
    fn insert(
        &mut self,
        position: TileChunkPosition,
        offset: TileChunkOffset,
        doors: DoorAdjacency,
    ) {
        let index = self.tiles.len();
        self.tiles.push(offset);
        if offset.on_chunk_edge() {
            self.tiles.swap(self.edges, index);
            self.edges += 1;
        }

        if doors != DoorAdjacency::NONE {
            let center = TilePosition::from((position, offset));
            if doors.contains(DoorAdjacency::WEST) {
                self.doors
                    .entry(center.west())
                    .or_default()
                    .insert(DoorAdjacency::WEST);
            }

            if doors.contains(DoorAdjacency::SOUTH) {
                self.doors
                    .entry(center.south())
                    .or_default()
                    .insert(DoorAdjacency::SOUTH);
            }

            if doors.contains(DoorAdjacency::EAST) {
                self.doors
                    .entry(center.east())
                    .or_default()
                    .insert(DoorAdjacency::EAST);
            }

            if doors.contains(DoorAdjacency::NORTH) {
                self.doors
                    .entry(center.north())
                    .or_default()
                    .insert(DoorAdjacency::NORTH);
            }
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
            let tile = chunk.get(offset);
            let walls = tile.wall_adjacency();
            let doors = tile.door_adjacency();
            let position = TilePosition::from((chunk_position, offset));

            if offset.x() == 0
                && !walls.contains(WallAdjacency::WEST)
                && !doors.contains(DoorAdjacency::WEST)
            {
                f(position.west())?;
            } else if offset.x() == edge
                && !walls.contains(WallAdjacency::EAST)
                && !doors.contains(DoorAdjacency::EAST)
            {
                f(position.east())?;
            }

            if offset.y() == 0
                && !walls.contains(WallAdjacency::SOUTH)
                && !doors.contains(DoorAdjacency::SOUTH)
            {
                f(position.south())?;
            } else if offset.y() == edge
                && !walls.contains(WallAdjacency::NORTH)
                && !doors.contains(DoorAdjacency::NORTH)
            {
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
            doors: HashMap::new(),
            edges: 0,
            region: Entity::PLACEHOLDER,
        }
    }
}

impl Default for TileChunkDisjointSet {
    fn default() -> Self {
        Self {
            entries: [TileChunkDisjointSetEntry::EMPTY; CHUNK_SIZE_SQUARED],
        }
    }
}

impl TileChunkDisjointSet {
    fn get(&self, offset: TileChunkOffset) -> TileChunkDisjointSetEntry {
        let entry = self.entries[offset.index()];
        debug_assert!(
            entry.is_solid()
                || (self.entries[entry.section().index()].section() == entry.section()),
            "sections should be normalized"
        );

        entry
    }

    fn get_section(&self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        self.get(offset).try_section()
    }

    fn find(&mut self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        let mut section = self.entries[offset.index()].try_section()?;
        if section != offset && self.entries[section.index()].section() != section {
            section = self.find(section).expect("section not found");
            self.entries[offset.index()].set_section(section);
        }

        Some(section)
    }

    fn find_south(&mut self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        self.find(offset.south()?)
    }

    fn find_west(&mut self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        self.find(offset.west()?)
    }

    fn insert(&mut self, offset: TileChunkOffset, doors: DoorAdjacency) {
        let section = match (self.find_west(offset), self.find_south(offset)) {
            (Some(west_section), Some(north_section)) => {
                if west_section != north_section {
                    self.entries[west_section.index()].set_section(north_section);
                }
                north_section
            }
            (Some(west_section), None) => west_section,
            (None, Some(north_section)) => north_section,
            (None, None) => offset,
        };

        self.entries[offset.index()] = TileChunkDisjointSetEntry::new(section, doors);
    }

    fn remove(&mut self, offset: TileChunkOffset) {
        self.entries[offset.index()] = TileChunkDisjointSetEntry::SOLID;
    }
}

impl TileChunkDisjointSetEntry {
    const EMPTY: TileChunkDisjointSetEntry = TileChunkDisjointSetEntry(0);
    const SOLID: TileChunkDisjointSetEntry = TileChunkDisjointSetEntry(u16::MAX);

    fn new(section: TileChunkOffset, doors: DoorAdjacency) -> Self {
        TileChunkDisjointSetEntry(section.index_u16() | (doors.bits() as u16) << 10)
    }

    fn is_solid(self) -> bool {
        self.0 == u16::MAX
    }

    fn section(self) -> TileChunkOffset {
        TileChunkOffset::from_index_u16(self.0 & 0x3FF)
    }

    fn try_section(self) -> Option<TileChunkOffset> {
        if self.is_solid() {
            None
        } else {
            Some(self.section())
        }
    }

    fn set_section(&mut self, section: TileChunkOffset) {
        self.0 = (self.0 & !0x3FF) | section.index_u16();
    }

    fn door_adjacency(self) -> DoorAdjacency {
        DoorAdjacency::from_bits_retain((self.0 >> 10) as u8)
    }

    fn invalid_sections(self, other: Self) -> impl Iterator<Item = TileChunkOffset> {
        [self.try_section(), other.try_section()]
            .into_iter()
            .flatten()
    }
}

impl fmt::Debug for TileChunkDisjointSetEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_solid() {
            f.write_str("TileChunkDisjointSetEntry::SOLID")
        } else {
            f.debug_tuple("TileChunkDisjointSetEntry")
                .field(&self.section().x())
                .field(&self.section().y())
                .field(&self.door_adjacency())
                .finish()
        }
    }
}

#[test]
fn pack_disjoint_set_entry() {
    for offset in TileChunkOffset::iter() {
        for doors in DoorAdjacency::values() {
            let entry = TileChunkDisjointSetEntry::new(offset, doors);
            assert_eq!(entry.section(), offset);
            assert_eq!(entry.door_adjacency(), doors);
        }
    }
}
