use bevy_math::prelude::*;
use bitflags::bitflags;

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WallAdjacency : u8 {
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

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DoorAdjacency : u8 {
        const NONE = 0b0000_0000;
        const NORTH = 0b0000_0001;
        const EAST = 0b0000_0010;
        const SOUTH = 0b0000_0100;
        const WEST = 0b0000_1000;
    }
}

impl WallAdjacency {
    pub(crate) const OFFSETS: [(WallAdjacency, IVec2); 8] = [
        (WallAdjacency::NORTH_WEST, IVec2::new(1, -1)),
        (WallAdjacency::NORTH, IVec2::new(0, -1)),
        (WallAdjacency::NORTH_EAST, IVec2::new(-1, -1)),
        (WallAdjacency::EAST, IVec2::new(-1, 0)),
        (WallAdjacency::SOUTH_EAST, IVec2::new(-1, 1)),
        (WallAdjacency::SOUTH, IVec2::new(0, 1)),
        (WallAdjacency::SOUTH_WEST, IVec2::new(1, 1)),
        (WallAdjacency::WEST, IVec2::new(1, 0)),
    ];

    pub fn values() -> impl Iterator<Item = Self> {
        (0..=255u8).map(Self::from_bits_retain)
    }
}

impl DoorAdjacency {
    pub(crate) const OFFSETS: [(DoorAdjacency, IVec2); 4] = [
        (DoorAdjacency::NORTH, IVec2::new(0, -1)),
        (DoorAdjacency::EAST, IVec2::new(-1, 0)),
        (DoorAdjacency::SOUTH, IVec2::new(0, 1)),
        (DoorAdjacency::WEST, IVec2::new(1, 0)),
    ];

    pub fn values() -> impl Iterator<Item = Self> {
        (0..=15u8).map(Self::from_bits_retain)
    }
}
