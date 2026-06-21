use core::fmt;
use std::{collections::VecDeque, ops::Index, u32};

use bevy_ecs::{
    entity::{EntityHashSet, hash_set},
    prelude::*,
};
use bevy_log::error;
use bevy_platform::collections::{HashMap, HashSet, hash_map};
use smallvec::SmallVec;
use wdn_physics::tile::{
    CHUNK_SIZE, CHUNK_SIZE_SQUARED,
    adjacency::Adjacency,
    index::TileIndex,
    material::{TileMaterial, TileMoveSpeed},
    position::{TileChunkOffset, TileLayerOffset, TilePosition},
    storage::{TileChunk, TileData, TileMap},
};

use crate::path::flow::{AddedFlowFields, FlowField};

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

#[derive(Default, Resource)]
pub struct AddedRegions {
    added_regions: EntityHashSet,
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
                invalid_regions.insert(current_section_data.region);
            }

            region_sections.push((current_chunk_id, current_section.layer_offset()));

            current_section_data.visit_neighbors(current_chunk, |neighbor| {
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

                for &tile_offset in &section.tiles {
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
            .region = trigger.entity;
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

impl TileChunkSections {
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

    fn section_mut(&mut self, offset: TileChunkOffset) -> &mut TileChunkSection {
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

    fn visit_neighbors(
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
