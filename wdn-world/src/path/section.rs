use std::fmt;

use bevy_ecs::{entity::EntityHashSet, prelude::*};
use bevy_platform::collections::{HashMap, HashSet, hash_map};
use wdn_physics::tile::{
    CHUNK_SIZE, CHUNK_SIZE_SQUARED,
    adjacency::Adjacency,
    material::TileMaterial,
    position::{TileChunkOffset, TilePosition},
    storage::TileChunk,
};

#[derive(Component, Default, Debug)]
pub struct TileChunkSections {
    sections: HashMap<TileChunkOffset, TileChunkSection>,
    set: TileChunkDisjointSet,
}

#[derive(Debug)]
pub struct TileChunkSection {
    tiles: Vec<TileChunkOffset>,
    edges: usize,
    region: Entity,
}

#[derive(Default, Resource)]
pub struct TileChunkSectionChanges {
    pub(crate) removed_sections: HashSet<TilePosition>,
    pub(crate) invalid_sections: HashMap<TilePosition, Entity>,
    pub(crate) invalid_regions: EntityHashSet,
}

#[derive(Debug)]
struct TileChunkDisjointSet {
    entries: [TileChunkDisjointSetEntry; CHUNK_SIZE_SQUARED],
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct TileChunkDisjointSetEntry(u16);

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
                            entry.insert(TileChunkSection::default()).insert(offset);
                            invalid_sections
                                .insert(TilePosition::from((position, section_id)), chunk_id);
                        }
                        hash_map::Entry::Occupied(entry) => {
                            if invalid_sections
                                .contains_key(&TilePosition::from((position, section_id)))
                            {
                                entry.into_mut().insert(offset);
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

impl TileChunkSections {
    pub fn section_id(&self, offset: TileChunkOffset) -> Option<TileChunkOffset> {
        self.set.get_section(offset)
    }

    pub fn region(&self, offset: TileChunkOffset) -> Option<Entity> {
        let section = self.set.get_section(offset)?;
        Some(self.sections[&section].region)
    }

    pub fn tiles(&self, offset: TileChunkOffset) -> Option<&[TileChunkOffset]> {
        let section = self.set.get_section(offset)?;
        Some(&self.sections[&section].tiles)
    }

    pub fn sections(&self) -> impl Iterator<Item = TileChunkOffset> + '_ {
        self.sections.keys().copied()
    }

    pub fn section(&self, offset: TileChunkOffset) -> &TileChunkSection {
        &self.sections[&offset]
    }

    pub fn section_mut(&mut self, offset: TileChunkOffset) -> &mut TileChunkSection {
        self.sections.get_mut(&offset).unwrap()
    }
}

impl TileChunkSection {
    pub fn size(&self) -> usize {
        self.tiles.len()
    }

    pub fn region(&self) -> Entity {
        self.region
    }

    pub fn set_region(&mut self, entity: Entity) {
        self.region = entity;
    }

    pub fn tiles(&self) -> &[TileChunkOffset] {
        &self.tiles
    }

    pub fn edges(&self) -> &[TileChunkOffset] {
        &self.tiles[..self.edges]
    }

    pub fn visit_neighbors(
        &self,
        chunk: &TileChunk,
        mut f: impl FnMut(TilePosition) -> Result,
    ) -> Result {
        let chunk_position = chunk.position();
        self.edges().iter().try_for_each(|&offset| {
            let edge = CHUNK_SIZE as u16 - 1;
            let tile = chunk.get(offset);
            let adjacency = tile.solid_adjacency();
            let position = TilePosition::from((chunk_position, offset));

            if offset.x() == 0 && !adjacency.contains(Adjacency::WEST) {
                f(position.west())?;
            } else if offset.x() == edge && !adjacency.contains(Adjacency::EAST) {
                f(position.east())?;
            }

            if offset.y() == 0 && !adjacency.contains(Adjacency::SOUTH) {
                f(position.south())?;
            } else if offset.y() == edge && !adjacency.contains(Adjacency::NORTH) {
                f(position.north())?;
            }

            Ok(())
        })
    }

    fn insert(&mut self, offset: TileChunkOffset) {
        let index = self.tiles.len();
        self.tiles.push(offset);
        if offset.on_chunk_edge() {
            self.tiles.swap(self.edges, index);
            self.edges += 1;
        }
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

    fn insert(&mut self, offset: TileChunkOffset, doors: Adjacency) {
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

    fn new(section: TileChunkOffset, doors: Adjacency) -> Self {
        let mut bits = section.index_u16();
        if doors.contains(Adjacency::NORTH) {
            bits |= 1 << 10;
        }
        if doors.contains(Adjacency::EAST) {
            bits |= 1 << 11;
        }
        if doors.contains(Adjacency::SOUTH) {
            bits |= 1 << 12;
        }
        if doors.contains(Adjacency::WEST) {
            bits |= 1 << 13;
        }

        TileChunkDisjointSetEntry(bits)
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

    fn door_adjacency(self) -> Adjacency {
        let mut adjacency = Adjacency::NONE;

        if self.0 & (1 << 10) != 0 {
            adjacency.insert(Adjacency::NORTH);
        }
        if self.0 & (1 << 11) != 0 {
            adjacency.insert(Adjacency::EAST);
        }
        if self.0 & (1 << 12) != 0 {
            adjacency.insert(Adjacency::SOUTH);
        }
        if self.0 & (1 << 13) != 0 {
            adjacency.insert(Adjacency::WEST);
        }

        adjacency
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
fn test_pack_disjoint_set_entry() {
    for offset in TileChunkOffset::iter() {
        for doors in Adjacency::values() {
            let entry = TileChunkDisjointSetEntry::new(offset, doors);
            assert_eq!(entry.section(), offset);
            assert_eq!(
                entry.door_adjacency(),
                doors.intersection(
                    Adjacency::NORTH | Adjacency::EAST | Adjacency::SOUTH | Adjacency::WEST
                )
            );
        }
    }
}
