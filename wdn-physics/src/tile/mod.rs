pub mod index;
pub mod storage;
#[cfg(test)]
mod tests;

use std::fmt;

use bevy_app::prelude::*;
use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use bevy_math::{I16Vec2, prelude::*};
use nonmax::NonMaxU16;

use crate::tile::{index::TileIndex, storage::TileMap};

pub const CHUNK_SIZE: usize = 32;

pub struct TilePlugin;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Component)]
#[component(immutable, on_insert = TilePosition::on_insert, on_replace = TilePosition::on_replace)]
pub struct TilePosition {
    layer: Entity,
    position: IVec2,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileChunkPosition {
    layer: Entity,
    position: I16Vec2,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TileChunkOffset(NonMaxU16);

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileIndex>().init_resource::<TileMap>();
    }
}

impl TilePosition {
    pub fn new(layer: Entity, x: i32, y: i32) -> Self {
        TilePosition {
            layer,
            position: IVec2::new(x, y),
        }
    }

    pub fn from_chunk_position_and_offset(
        chunk_position: TileChunkPosition,
        chunk_offset: TileChunkOffset,
    ) -> Self {
        TilePosition::new(
            chunk_position.layer(),
            chunk_position.x() as i32 * CHUNK_SIZE as i32 + chunk_offset.x() as i32,
            chunk_position.y() as i32 * CHUNK_SIZE as i32 + chunk_offset.y() as i32,
        )
    }

    pub fn from_vec(layer: Entity, position: IVec2) -> Self {
        TilePosition { layer, position }
    }

    pub fn floor(layer: Entity, position: Vec2) -> Self {
        TilePosition::new(layer, floor(position.x), floor(position.y))
    }

    pub fn with_offset(&self, offset: IVec2) -> Self {
        TilePosition::from_vec(self.layer(), self.position() + offset)
    }

    pub fn layer(&self) -> Entity {
        self.layer
    }

    pub fn position(&self) -> IVec2 {
        self.position
    }

    pub fn x(&self) -> i32 {
        self.position.x
    }

    pub fn y(&self) -> i32 {
        self.position.y
    }

    pub fn north(&self) -> Self {
        self.with_offset(IVec2::new(0, -1))
    }

    pub fn south(&self) -> Self {
        self.with_offset(IVec2::new(0, 1))
    }

    pub fn east(&self) -> Self {
        self.with_offset(IVec2::new(1, 0))
    }

    pub fn west(&self) -> Self {
        self.with_offset(IVec2::new(-1, 0))
    }

    pub fn chunk_position(&self) -> TileChunkPosition {
        TileChunkPosition::new(
            self.layer,
            self.x().div_euclid(CHUNK_SIZE as i32) as i16,
            self.y().div_euclid(CHUNK_SIZE as i32) as i16,
        )
    }

    pub fn chunk_offset(&self) -> TileChunkOffset {
        TileChunkOffset::new(
            self.x().rem_euclid(CHUNK_SIZE as i32) as u16,
            self.y().rem_euclid(CHUNK_SIZE as i32) as u16,
        )
    }

    pub fn on_chunk_edge(&self) -> bool {
        self.chunk_offset().on_chunk_edge()
    }

    pub fn neighborhood(&self) -> [TilePosition; 9] {
        [
            self.with_offset(IVec2::new(-1, -1)),
            self.with_offset(IVec2::new(0, -1)),
            self.with_offset(IVec2::new(1, -1)),
            self.with_offset(IVec2::new(-1, 0)),
            self.with_offset(IVec2::new(0, 0)),
            self.with_offset(IVec2::new(1, 0)),
            self.with_offset(IVec2::new(-1, 1)),
            self.with_offset(IVec2::new(0, 1)),
            self.with_offset(IVec2::new(1, 1)),
        ]
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let tile = *world.get::<TilePosition>(context.entity).unwrap();
        if tile != TilePosition::default() {
            world
                .resource_mut::<TileIndex>()
                .insert(context.entity, tile);
        }
    }

    fn on_replace(mut world: DeferredWorld, context: HookContext) {
        let tile = *world.get::<TilePosition>(context.entity).unwrap();
        if tile != TilePosition::default() {
            world
                .resource_mut::<TileIndex>()
                .remove(context.entity, tile);
        }
    }
}

impl Default for TilePosition {
    fn default() -> Self {
        Self {
            layer: Entity::PLACEHOLDER,
            position: IVec2::ZERO,
        }
    }
}

impl fmt::Debug for TilePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TilePosition")
            .field(&self.layer())
            .field(&self.x())
            .field(&self.y())
            .finish()
    }
}

impl TileChunkPosition {
    pub fn new(layer: Entity, x: i16, y: i16) -> Self {
        TileChunkPosition {
            layer,
            position: I16Vec2::new(x, y),
        }
    }

    pub fn from_vec(layer: Entity, position: I16Vec2) -> Self {
        TileChunkPosition { layer, position }
    }

    pub fn x(&self) -> i16 {
        self.position.x
    }

    pub fn y(&self) -> i16 {
        self.position.y
    }

    pub fn position(&self) -> I16Vec2 {
        self.position
    }

    pub fn layer(&self) -> Entity {
        self.layer
    }
}

impl fmt::Debug for TileChunkPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TileChunkPosition({:?}, {}, {})",
            self.layer(),
            self.x(),
            self.y()
        )
    }
}

impl TileChunkOffset {
    pub fn new(x: u16, y: u16) -> Self {
        TileChunkOffset::from_index(y as usize * CHUNK_SIZE + x as usize)
    }

    pub fn iter() -> impl ExactSizeIterator<Item = TileChunkOffset> {
        (0..(CHUNK_SIZE * CHUNK_SIZE)).map(TileChunkOffset::from_index)
    }

    pub fn x(&self) -> u16 {
        (self.0.get() as usize % CHUNK_SIZE) as u16
    }

    pub fn y(&self) -> u16 {
        (self.0.get() as usize / CHUNK_SIZE) as u16
    }

    pub fn north(&self) -> Option<Self> {
        match self.y().checked_sub(1) {
            Some(y) => Some(TileChunkOffset::new(self.x(), y)),
            None => None,
        }
    }

    pub fn west(&self) -> Option<Self> {
        match self.x().checked_sub(1) {
            Some(x) => Some(TileChunkOffset::new(x, self.y())),
            None => None,
        }
    }

    pub fn index(&self) -> usize {
        self.0.get() as usize
    }

    pub fn from_index(index: usize) -> Self {
        TileChunkOffset::from_index_u16(index as u16)
    }

    pub fn index_u16(&self) -> u16 {
        self.0.get()
    }

    pub fn from_index_u16(index: u16) -> Self {
        TileChunkOffset(NonMaxU16::new(index).unwrap())
    }

    pub fn on_chunk_edge(&self) -> bool {
        self.x() == 0
            || self.x() == (CHUNK_SIZE - 1) as u16
            || self.y() == 0
            || self.y() == (CHUNK_SIZE - 1) as u16
    }
}

impl fmt::Debug for TileChunkOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TileChunkOffset")
            .field(&self.x())
            .field(&self.y())
            .finish()
    }
}

fn floor(value: f32) -> i32 {
    if value.is_sign_negative() {
        (value as i32) - 1
    } else {
        value as i32
    }
}

#[test]
fn test_tile_chunk_offset_index() {
    let offset = TileChunkOffset::new(5, 3);

    assert_eq!(offset.x(), 5);
    assert_eq!(offset.y(), 3);
    assert_eq!(offset.index(), 101);
}

#[test]
fn test_floor() {
    assert_eq!(floor(0.0), 0);
    assert_eq!(floor(-0.0), -1);
    assert_eq!(floor(f32::EPSILON), 0);
    assert_eq!(floor(-f32::EPSILON), -1);
    assert_eq!(floor(0.00001), 0);
    assert_eq!(floor(-0.00001), -1);
    assert_eq!(floor(0.1), 0);
    assert_eq!(floor(-0.1), -1);
    assert_eq!(floor(0.9), 0);
    assert_eq!(floor(-0.9), -1);
    assert_eq!(floor(0.99999), 0);
    assert_eq!(floor(-0.99999), -1);
    assert_eq!(floor(1.0), 1);
    assert_eq!(floor(f32::next_down(1.0)), 0);
    assert_eq!(floor(-1.0), -2);
    assert_eq!(floor(f32::next_up(-1.0)), -1);
    assert_eq!(floor(1.00001), 1);
    assert_eq!(floor(-1.00001), -2);
    assert_eq!(floor(1.1), 1);
    assert_eq!(floor(-1.1), -2);
    assert_eq!(floor(1.5), 1);
    assert_eq!(floor(-1.5), -2);
    assert_eq!(floor(1.9), 1);
    assert_eq!(floor(-1.9), -2);
    assert_eq!(floor(2.0), 2);
    assert_eq!(floor(-2.0), -3);
    assert_eq!(floor(2.5), 2);
    assert_eq!(floor(-2.5), -3);
    assert_eq!(floor(42.0), 42);
    assert_eq!(floor(-42.0), -43);
    assert_eq!(floor(42.7), 42);
    assert_eq!(floor(-42.7), -43);
    assert_eq!(floor(99.99), 99);
    assert_eq!(floor(-99.99), -100);
    assert_eq!(floor(100.0), 100);
    assert_eq!(floor(-100.0), -101);
    assert_eq!(floor(999.99999), 1000);
    assert_eq!(floor(-999.99999), -1001);
    assert_eq!(floor(1000.0), 1000);
    assert_eq!(floor(-1000.0), -1001);
    assert_eq!(floor(1000000.0), 1000000);
    assert_eq!(floor(1000000.5), 1000000);
    assert_eq!(floor(-1000000.0), -1000001);
    assert_eq!(floor(-1000000.5), -1000001);
    assert_eq!(floor(2147483520.0), 2147483520);
    assert_eq!(floor(-2147483520.0), -2147483521);
}
