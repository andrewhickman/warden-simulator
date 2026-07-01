use bevy_ecs::prelude::*;

use crate::tile::position::TilePosition;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[require(TilePosition)]
pub struct TileMaterial(u16);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TileKind {
    #[default]
    Empty = 0b00,
    Wall = 0b01,
    Door = 0b10,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TileMoveSpeed {
    Medium = 0b00,
    Slow = 0b01,
    Fast = 0b10,
}

impl TileMaterial {
    pub const EMPTY: Self = TileMaterial::new(TileKind::Empty, TileMoveSpeed::Medium, 0);
    pub const WALL: Self = TileMaterial::new(TileKind::Wall, TileMoveSpeed::Medium, 0);
    pub const DOOR: Self = TileMaterial::new(TileKind::Door, TileMoveSpeed::Medium, 0);

    pub const fn new(kind: TileKind, move_speed: TileMoveSpeed, id: u16) -> Self {
        debug_assert!(id <= 0x0FFF);
        TileMaterial(id | ((move_speed as u16) << 12) | ((kind as u16) << 14))
    }

    pub fn id(&self) -> u16 {
        self.0 & 0x0FFF
    }

    pub fn kind(&self) -> TileKind {
        TileKind::from_bits((self.0 >> 14) & 0b11)
    }

    pub fn move_speed(&self) -> TileMoveSpeed {
        TileMoveSpeed::from_bits((self.0 >> 12) & 0b11)
    }
}

impl Default for TileMaterial {
    fn default() -> Self {
        TileMaterial::new(TileKind::Empty, TileMoveSpeed::Medium, 0)
    }
}

impl TileKind {
    pub fn is_empty(&self) -> bool {
        matches!(self, TileKind::Empty)
    }

    pub fn bits(&self) -> u16 {
        *self as u16
    }

    pub fn from_bits(bits: u16) -> Self {
        match bits {
            0b00 => TileKind::Empty,
            0b01 => TileKind::Wall,
            0b10 => TileKind::Door,
            _ => panic!("invalid TileKind bits: {bits}"),
        }
    }
}

impl TileMoveSpeed {
    pub fn factor(&self) -> f32 {
        match self {
            TileMoveSpeed::Slow => 0.6,
            TileMoveSpeed::Medium => 1.0,
            TileMoveSpeed::Fast => 1.4,
        }
    }

    pub fn bits(&self) -> u16 {
        *self as u16
    }

    pub fn from_bits(bits: u16) -> Self {
        match bits {
            0b00 => TileMoveSpeed::Medium,
            0b01 => TileMoveSpeed::Slow,
            0b10 => TileMoveSpeed::Fast,
            _ => panic!("invalid TileMoveSpeed bits: {bits}"),
        }
    }
}

#[test]
fn test_pack_tile_material() {
    for id in 0..10 {
        for kind in [TileKind::Empty, TileKind::Wall, TileKind::Door] {
            for speed in [
                TileMoveSpeed::Slow,
                TileMoveSpeed::Medium,
                TileMoveSpeed::Fast,
            ] {
                let material = TileMaterial::new(kind, speed, id);
                assert_eq!(material.kind(), kind);
                assert_eq!(material.move_speed(), speed);
                assert_eq!(material.id(), id);
            }
        }
    }
}
