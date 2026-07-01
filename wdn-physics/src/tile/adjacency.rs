use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bitflags::bitflags;

#[derive(Debug, Default, Clone, Copy, Component, PartialEq, Eq)]
pub struct TileAdjacency {
    pub(crate) walls: Adjacency,
    pub(crate) doors: Adjacency,
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Adjacency : u8 {
        const NONE = 0b0000_0000;
        const NORTH_WEST = 0b0000_0001;
        const NORTH = 0b0000_0010;
        const NORTH_EAST = 0b0000_0100;
        const EAST = 0b0000_1000;
        const SOUTH_EAST = 0b0001_0000;
        const SOUTH = 0b0010_0000;
        const SOUTH_WEST = 0b0100_0000;
        const WEST = 0b1000_0000;
    }
}

impl TileAdjacency {
    pub const NONE: Self = Self {
        walls: Adjacency::NONE,
        doors: Adjacency::NONE,
    };

    pub fn new(walls: Adjacency, doors: Adjacency) -> Self {
        Self { walls, doors }
    }

    pub fn solid(&self) -> Adjacency {
        self.walls | self.doors
    }

    pub fn empty(&self) -> Adjacency {
        self.solid().complement()
    }

    pub fn walls(&self) -> Adjacency {
        self.walls
    }

    pub fn doors(&self) -> Adjacency {
        self.doors
    }
}

impl Adjacency {
    pub(crate) const OFFSETS: [(Adjacency, IVec2); 8] = [
        (Adjacency::NORTH_WEST, IVec2::new(1, -1)),
        (Adjacency::NORTH, IVec2::new(0, -1)),
        (Adjacency::NORTH_EAST, IVec2::new(-1, -1)),
        (Adjacency::EAST, IVec2::new(-1, 0)),
        (Adjacency::SOUTH_EAST, IVec2::new(-1, 1)),
        (Adjacency::SOUTH, IVec2::new(0, 1)),
        (Adjacency::SOUTH_WEST, IVec2::new(1, 1)),
        (Adjacency::WEST, IVec2::new(1, 0)),
    ];

    pub fn opposite(&self) -> Self {
        Adjacency::from_bits_retain(self.bits().rotate_right(4))
    }

    pub fn values() -> impl Iterator<Item = Self> {
        (0..=255u8).map(Self::from_bits_retain)
    }
}
