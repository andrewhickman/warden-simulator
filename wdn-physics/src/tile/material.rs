use std::fmt;

use bevy_ecs::prelude::*;

use crate::tile::position::TilePosition;

pub const TILE_SLOW_SPEED: f32 = 0.6;
pub const TILE_MEDIUM_SPEED: f32 = 1.0;
pub const TILE_FAST_SPEED: f32 = 1.4;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
#[require(TilePosition)]
pub struct TileMaterial(TileMaterialFlags);

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TileMaterialFlags: u16 {
        const ID_MASK = (1 << 10) - 1;

        const GROUND_INSIDE_MASK = 0b1 << 10;
        const GROUND_INSIDE = 0b1 << 10;
        const GROUND_OUTSIDE = 0b0 << 10;

        const STAIR_DIRECTION_MASK = 0b11 << 10;
        const STAIR_DIRECTION_NORTH = 0b00 << 10;
        const STAIR_DIRECTION_EAST = 0b01 << 10;
        const STAIR_DIRECTION_SOUTH = 0b10 << 10;
        const STAIR_DIRECTION_WEST = 0b11 << 10;

        const DOOR_DIRECTION_MASK = 0b1 << 11;
        const DOOR_DIRECTION_HORIZONTAL = 0b0 << 11;
        const DOOR_DIRECTION_VERTICAL = 0b1 << 11;

        const MOVE_SPEED_MASK = 0b11 << 12;
        const MOVE_SPEED_MEDIUM = 0b00 << 12;
        const MOVE_SPEED_FAST = 0b01 << 12;
        const MOVE_SPEED_SLOW = 0b10 << 12;

        const KIND_MASK = 0b11 << 14;
        const KIND_EMPTY = 0b00 << 14;
        const KIND_WALL = 0b01 << 14;
        const KIND_DOOR = 0b10 << 14;
        const KIND_STAIRS = 0b11 << 14;
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TileKind {
    #[default]
    Empty = 0b00,
    Wall = 0b01,
    Door = 0b10,
    Stairs = 0b11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TileMoveSpeed {
    Medium = 0b00,
    Slow = 0b01,
    Fast = 0b10,
}

impl TileMaterial {
    pub const EMPTY: Self = TileMaterial(TileMaterialFlags::KIND_EMPTY);
    pub const WALL: Self = TileMaterial(TileMaterialFlags::KIND_WALL);
    pub const DOOR: Self = TileMaterial(TileMaterialFlags::KIND_DOOR);
    pub const STAIR: Self = TileMaterial(TileMaterialFlags::KIND_STAIRS);
    pub const SLOW: Self =
        TileMaterial(TileMaterialFlags::KIND_EMPTY.union(TileMaterialFlags::MOVE_SPEED_SLOW));
    pub const FAST: Self =
        TileMaterial(TileMaterialFlags::KIND_EMPTY.union(TileMaterialFlags::MOVE_SPEED_FAST));

    pub const fn new(kind: TileKind, id: u16, move_speed: TileMoveSpeed) -> Self {
        TileMaterial(kind.flags())
            .with_id(id)
            .with_move_speed(move_speed)
    }

    pub const fn empty(id: u16, move_speed: TileMoveSpeed) -> Self {
        TileMaterial::EMPTY.with_id(id).with_move_speed(move_speed)
    }

    pub const fn wall(id: u16) -> Self {
        TileMaterial::WALL.with_id(id)
    }

    pub const fn door(id: u16, move_speed: TileMoveSpeed) -> Self {
        TileMaterial::DOOR.with_id(id).with_move_speed(move_speed)
    }

    pub const fn stairs(id: u16, move_speed: TileMoveSpeed) -> Self {
        TileMaterial::STAIR.with_id(id).with_move_speed(move_speed)
    }

    pub const fn kind(&self) -> TileKind {
        match self.0.intersection(TileMaterialFlags::KIND_MASK) {
            TileMaterialFlags::KIND_EMPTY => TileKind::Empty,
            TileMaterialFlags::KIND_WALL => TileKind::Wall,
            TileMaterialFlags::KIND_DOOR => TileKind::Door,
            TileMaterialFlags::KIND_STAIRS => TileKind::Stairs,
            _ => unreachable!(),
        }
    }

    pub const fn id(self) -> u16 {
        self.0.intersection(TileMaterialFlags::ID_MASK).bits()
    }

    pub fn set_id(&mut self, id: u16) {
        *self = self.with_id(id);
    }

    pub const fn with_id(self, id: u16) -> Self {
        let id = TileMaterialFlags::from_bits_retain(id);
        debug_assert!(TileMaterialFlags::ID_MASK.contains(id));
        TileMaterial(self.0.difference(TileMaterialFlags::ID_MASK).union(id))
    }

    pub fn flags(&self) -> TileMaterialFlags {
        self.0
    }

    pub fn set_flags(&mut self, mask: TileMaterialFlags, flags: TileMaterialFlags) {
        *self = self.with_flags(mask, flags);
    }

    pub fn with_flags(self, mask: TileMaterialFlags, flags: TileMaterialFlags) -> Self {
        TileMaterial(self.0.difference(mask).union(flags))
    }

    pub const fn move_speed(&self) -> TileMoveSpeed {
        match self.0.intersection(TileMaterialFlags::MOVE_SPEED_MASK) {
            TileMaterialFlags::MOVE_SPEED_SLOW => TileMoveSpeed::Slow,
            TileMaterialFlags::MOVE_SPEED_FAST => TileMoveSpeed::Fast,
            _ => TileMoveSpeed::Medium,
        }
    }

    pub const fn with_move_speed(self, move_speed: TileMoveSpeed) -> Self {
        TileMaterial(
            self.0
                .difference(TileMaterialFlags::MOVE_SPEED_MASK)
                .union(move_speed.flags()),
        )
    }
}

impl Default for TileMaterial {
    fn default() -> Self {
        TileMaterial::EMPTY
    }
}

impl fmt::Debug for TileMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TileMaterial")
            .field("kind", &self.kind())
            .field("move_speed", &self.move_speed())
            .field("id", &self.id())
            .finish()
    }
}

impl TileKind {
    pub fn values() -> impl Iterator<Item = TileKind> {
        [
            TileKind::Empty,
            TileKind::Wall,
            TileKind::Door,
            TileKind::Stairs,
        ]
        .into_iter()
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, TileKind::Empty)
    }

    const fn flags(&self) -> TileMaterialFlags {
        match self {
            TileKind::Empty => TileMaterialFlags::KIND_EMPTY,
            TileKind::Wall => TileMaterialFlags::KIND_WALL,
            TileKind::Door => TileMaterialFlags::KIND_DOOR,
            TileKind::Stairs => TileMaterialFlags::KIND_STAIRS,
        }
    }
}

impl TileMoveSpeed {
    pub fn factor(&self) -> f32 {
        match self {
            TileMoveSpeed::Slow => TILE_SLOW_SPEED,
            TileMoveSpeed::Medium => TILE_MEDIUM_SPEED,
            TileMoveSpeed::Fast => TILE_FAST_SPEED,
        }
    }

    const fn flags(&self) -> TileMaterialFlags {
        match self {
            TileMoveSpeed::Medium => TileMaterialFlags::MOVE_SPEED_MEDIUM,
            TileMoveSpeed::Slow => TileMaterialFlags::MOVE_SPEED_SLOW,
            TileMoveSpeed::Fast => TileMaterialFlags::MOVE_SPEED_FAST,
        }
    }
}
