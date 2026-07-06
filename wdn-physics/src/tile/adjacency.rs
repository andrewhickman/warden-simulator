use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bitflags::bitflags;

use crate::tile::material::TileKind;

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

    pub fn doors(&self) -> Adjacency {
        self.doors
    }

    pub fn insert_walls(&mut self, adjacency: Adjacency) {
        self.walls.insert(adjacency);
    }

    pub fn insert_doors(&mut self, adjacency: Adjacency) {
        self.doors.insert(adjacency);
    }

    pub fn remove_walls(&mut self, adjacency: Adjacency) {
        self.walls.remove(adjacency);
    }

    pub fn remove_doors(&mut self, adjacency: Adjacency) {
        self.doors.remove(adjacency);
    }

    pub fn update(
        &mut self,
        adjacency: Adjacency,
        prev_material: TileKind,
        current_material: TileKind,
    ) {
        match prev_material {
            TileKind::Empty => {}
            TileKind::Wall => self.remove_walls(adjacency),
            TileKind::Door => self.remove_doors(adjacency),
        }

        match current_material {
            TileKind::Empty => {}
            TileKind::Wall => self.insert_walls(adjacency),
            TileKind::Door => self.insert_doors(adjacency),
        }
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
}

#[test]
fn test_insert_walls() {
    let mut adj = TileAdjacency::NONE;

    adj.insert_walls(Adjacency::NORTH);
    assert_eq!(adj.walls(), Adjacency::NORTH);
    assert_eq!(adj.doors(), Adjacency::NONE);

    adj.insert_walls(Adjacency::SOUTH | Adjacency::EAST);
    assert_eq!(
        adj.walls(),
        Adjacency::NORTH | Adjacency::SOUTH | Adjacency::EAST
    );
    assert_eq!(adj.doors(), Adjacency::NONE);
}

#[test]
fn test_insert_doors() {
    let mut adj = TileAdjacency::NONE;

    adj.insert_doors(Adjacency::WEST);
    assert_eq!(adj.doors(), Adjacency::WEST);
    assert_eq!(adj.walls(), Adjacency::NONE);

    adj.insert_doors(Adjacency::WEST | Adjacency::NORTH_EAST);
    assert_eq!(adj.doors(), Adjacency::WEST | Adjacency::NORTH_EAST);
    assert_eq!(adj.walls(), Adjacency::NONE);
}

#[test]
fn test_remove_walls() {
    let mut adj = TileAdjacency::new(
        Adjacency::NORTH | Adjacency::EAST | Adjacency::SOUTH,
        Adjacency::WEST,
    );

    adj.remove_walls(Adjacency::EAST);
    assert_eq!(adj.walls(), Adjacency::NORTH | Adjacency::SOUTH);
    assert_eq!(adj.doors(), Adjacency::WEST);

    adj.remove_walls(Adjacency::NORTH | Adjacency::SOUTH);
    assert_eq!(adj.walls(), Adjacency::NONE);
}

#[test]
fn test_remove_doors() {
    let mut adj = TileAdjacency::new(Adjacency::NORTH, Adjacency::SOUTH | Adjacency::WEST);

    adj.remove_doors(Adjacency::WEST);
    assert_eq!(adj.doors(), Adjacency::SOUTH);
    assert_eq!(adj.walls(), Adjacency::NORTH);

    adj.remove_doors(Adjacency::SOUTH);
    assert_eq!(adj.doors(), Adjacency::NONE);
}
