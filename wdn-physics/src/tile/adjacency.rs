use std::fmt;

use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bitflags::bitflags;

use crate::tile::material::TileKind;

#[derive(Default, Clone, Copy, Component, PartialEq, Eq)]
pub struct TileAdjacency(Adjacency, Adjacency);

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
    pub const NONE: Self = Self(Adjacency::NONE, Adjacency::NONE);

    pub fn solid(&self) -> Adjacency {
        self.0 ^ self.1
    }

    pub fn empty(&self) -> Adjacency {
        (self.0 | self.1).complement()
    }

    pub fn walls(&self) -> Adjacency {
        self.0 & !self.1
    }

    pub fn doors(&self) -> Adjacency {
        self.1 & !self.0
    }

    pub fn stairs(&self) -> Adjacency {
        self.0 & self.1
    }

    pub fn get(&self, kind: TileKind) -> Adjacency {
        match kind {
            TileKind::Empty => self.empty(),
            TileKind::Wall => self.walls(),
            TileKind::Door => self.doors(),
            TileKind::Stairs => self.stairs(),
        }
    }

    pub fn update(
        &mut self,
        adjacency: Adjacency,
        prev_material: TileKind,
        current_material: TileKind,
    ) {
        debug_assert!(self.get(prev_material).contains(adjacency));

        match (prev_material, current_material) {
            (TileKind::Empty, TileKind::Empty)
            | (TileKind::Wall, TileKind::Wall)
            | (TileKind::Door, TileKind::Door)
            | (TileKind::Stairs, TileKind::Stairs) => {}
            (TileKind::Empty, TileKind::Wall) | (TileKind::Door, TileKind::Stairs) => {
                self.0.insert(adjacency)
            }
            (TileKind::Empty, TileKind::Door) | (TileKind::Wall, TileKind::Stairs) => {
                self.1.insert(adjacency)
            }
            (TileKind::Wall, TileKind::Empty) | (TileKind::Stairs, TileKind::Door) => {
                self.0.remove(adjacency);
            }
            (TileKind::Door, TileKind::Empty) | (TileKind::Stairs, TileKind::Wall) => {
                self.1.remove(adjacency);
            }
            (TileKind::Empty, TileKind::Stairs) => {
                self.0.insert(adjacency);
                self.1.insert(adjacency);
            }
            (TileKind::Door, TileKind::Wall) => {
                self.1.remove(adjacency);
                self.0.insert(adjacency);
            }
            (TileKind::Wall, TileKind::Door) => {
                self.0.remove(adjacency);
                self.1.insert(adjacency);
            }
            (TileKind::Stairs, TileKind::Empty) => {
                self.0.remove(adjacency);
                self.1.remove(adjacency);
            }
        }
    }
}

impl fmt::Debug for TileAdjacency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TileAdjacency")
            .field("walls", &self.walls())
            .field("doors", &self.doors())
            .field("stairs", &self.stairs())
            .finish()
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

#[test]
fn test_update() {
    let mut adj = TileAdjacency::NONE;

    adj.update(Adjacency::NORTH, TileKind::Empty, TileKind::Wall);
    assert_eq!(adj.walls(), Adjacency::NORTH);
    assert_eq!(adj.doors(), Adjacency::NONE);

    adj.update(Adjacency::NORTH, TileKind::Wall, TileKind::Door);
    assert_eq!(adj.walls(), Adjacency::NONE);
    assert_eq!(adj.doors(), Adjacency::NORTH);

    adj.update(Adjacency::NORTH, TileKind::Door, TileKind::Empty);
    assert_eq!(adj.walls(), Adjacency::NONE);
    assert_eq!(adj.doors(), Adjacency::NONE);

    adj.update(Adjacency::EAST, TileKind::Empty, TileKind::Wall);
    adj.update(Adjacency::SOUTH, TileKind::Empty, TileKind::Door);
    assert_eq!(adj.walls(), Adjacency::EAST);
    assert_eq!(adj.doors(), Adjacency::SOUTH);
    assert_eq!(adj.solid(), Adjacency::EAST | Adjacency::SOUTH);

    adj.update(Adjacency::EAST, TileKind::Wall, TileKind::Empty);
    assert_eq!(adj.walls(), Adjacency::NONE);
    assert_eq!(adj.doors(), Adjacency::SOUTH);

    adj.update(Adjacency::WEST, TileKind::Empty, TileKind::Stairs);
    assert_eq!(adj.stairs(), Adjacency::WEST);
    assert_eq!(adj.walls(), Adjacency::NONE);
    assert_eq!(adj.doors(), Adjacency::SOUTH);
    assert_eq!(adj.solid(), Adjacency::SOUTH);

    adj.update(Adjacency::WEST, TileKind::Stairs, TileKind::Wall);
    assert_eq!(adj.stairs(), Adjacency::NONE);
    assert_eq!(adj.walls(), Adjacency::WEST);

    adj.update(Adjacency::WEST, TileKind::Wall, TileKind::Stairs);
    assert_eq!(adj.walls(), Adjacency::NONE);
    assert_eq!(adj.stairs(), Adjacency::WEST);

    adj.update(Adjacency::WEST, TileKind::Stairs, TileKind::Door);
    assert_eq!(adj.stairs(), Adjacency::NONE);
    assert_eq!(adj.doors(), Adjacency::SOUTH | Adjacency::WEST);

    adj.update(Adjacency::WEST, TileKind::Door, TileKind::Stairs);
    assert_eq!(adj.doors(), Adjacency::SOUTH);
    assert_eq!(adj.stairs(), Adjacency::WEST);

    adj.update(Adjacency::WEST, TileKind::Stairs, TileKind::Empty);
    assert_eq!(adj.stairs(), Adjacency::NONE);
    assert_eq!(adj.doors(), Adjacency::SOUTH);
}
