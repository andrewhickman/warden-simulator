use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bitflags::bitflags;

#[derive(Debug, Default, Clone, Copy, Component, PartialEq, Eq)]
pub struct TileAdjacency {
    walls: Adjacency,
    doors: Adjacency,
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

    pub fn walls_mut(&mut self) -> &mut Adjacency {
        &mut self.walls
    }

    pub fn doors(&self) -> Adjacency {
        self.doors
    }

    pub fn doors_mut(&mut self) -> &mut Adjacency {
        &mut self.doors
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

    pub fn flip_y(&self) -> Self {
        Adjacency::from_bits_retain(self.bits().reverse_bits().rotate_right(1))
    }

    pub fn flip_x(&self) -> Self {
        Adjacency::from_bits_retain(self.bits().reverse_bits().rotate_right(5))
    }

    pub fn values() -> impl Iterator<Item = Self> {
        (0..=255u8).map(Self::from_bits_retain)
    }
}

#[test]
fn test_flip_x() {
    assert_eq!(Adjacency::NORTH.flip_x(), Adjacency::NORTH);
    assert_eq!(Adjacency::SOUTH.flip_x(), Adjacency::SOUTH);
    assert_eq!(Adjacency::EAST.flip_x(), Adjacency::WEST);
    assert_eq!(Adjacency::WEST.flip_x(), Adjacency::EAST);
    assert_eq!(Adjacency::NORTH_EAST.flip_x(), Adjacency::NORTH_WEST);
    assert_eq!(Adjacency::NORTH_WEST.flip_x(), Adjacency::NORTH_EAST);
    assert_eq!(Adjacency::SOUTH_EAST.flip_x(), Adjacency::SOUTH_WEST);
    assert_eq!(Adjacency::SOUTH_WEST.flip_x(), Adjacency::SOUTH_EAST);
}

#[test]
fn test_flip_y() {
    assert_eq!(Adjacency::EAST.flip_y(), Adjacency::EAST);
    assert_eq!(Adjacency::WEST.flip_y(), Adjacency::WEST);
    assert_eq!(Adjacency::NORTH.flip_y(), Adjacency::SOUTH);
    assert_eq!(Adjacency::SOUTH.flip_y(), Adjacency::NORTH);
    assert_eq!(Adjacency::NORTH_EAST.flip_y(), Adjacency::SOUTH_EAST);
    assert_eq!(Adjacency::SOUTH_EAST.flip_y(), Adjacency::NORTH_EAST);
    assert_eq!(Adjacency::NORTH_WEST.flip_y(), Adjacency::SOUTH_WEST);
    assert_eq!(Adjacency::SOUTH_WEST.flip_y(), Adjacency::NORTH_WEST);
}
