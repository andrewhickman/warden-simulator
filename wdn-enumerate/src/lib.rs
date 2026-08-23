mod ids;

use std::fmt;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TileVariant {
    Empty,
    Wall,
    DoorH,
    DoorV,
    StairN,
    StairS,
    StairE,
    StairW,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct Sprite {
    pub mid: MidSprite,
    pub top: TopSprite,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, PartialOrd, Ord)]
pub enum MidSprite {
    Empty,
    Corner {
        east: EastDecoration,
        south: SouthDecoration,
    },
    Horizontal {
        south: SouthDecoration,
    },
    Vertical {
        east: EastDecoration,
    },
    InverseCorner,
    Full,
    StairN,
    StairNFull,
    StairS,
    StairSFull,
    StairE,
    StairEFull,
    StairW,
    StairWFull,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, PartialOrd, Ord)]
pub enum TopSprite {
    Empty {
        south_east: SouthEastTopDecoration,
    },
    Corner {
        south_east: SouthEastTopDecoration,
        deco: TopDecoration,
    },
    Horizontal {
        deco: TopDecoration,
    },
    StairN,
    StairNFull,
    StairE,
    StairEFull,
    StairW,
    StairWFull,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, PartialOrd, Ord)]
pub enum EastDecoration {
    None,
    Door,
    StairN,
    StairS,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, PartialOrd, Ord)]
pub enum SouthDecoration {
    None,
    Door,
    StairN,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, PartialOrd, Ord)]
pub enum TopDecoration {
    None,
    Door,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, PartialOrd, Ord)]
pub enum SouthEastTopDecoration {
    None,
    StairN,
    StairW,
    StairWFull,
}

impl TileVariant {
    pub fn values() -> impl IntoIterator<Item = TileVariant> {
        [
            TileVariant::Empty,
            TileVariant::Wall,
            TileVariant::DoorH,
            TileVariant::DoorV,
            TileVariant::StairN,
            TileVariant::StairS,
            TileVariant::StairE,
            TileVariant::StairW,
        ]
    }
}

impl Sprite {
    pub fn bleeds(&self) -> bool {
        matches!(
            self.mid,
            MidSprite::Corner { .. }
                | MidSprite::Horizontal { .. }
                | MidSprite::Vertical { .. }
                | MidSprite::InverseCorner
                | MidSprite::Full
        ) && !matches!(
            self.top,
            TopSprite::Empty {
                south_east: SouthEastTopDecoration::None
            }
        )
    }
}

impl MidSprite {
    pub fn resolve(
        center: TileVariant,
        _north: TileVariant,
        _north_east: TileVariant,
        east: TileVariant,
        south_east: TileVariant,
        south: TileVariant,
    ) -> Self {
        match center {
            TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV => MidSprite::Empty,
            TileVariant::Wall => {
                let east_open = !matches!(east, TileVariant::Wall | TileVariant::StairW);
                let south_open = south != TileVariant::Wall;
                match (east_open, south_open) {
                    (true, true) => MidSprite::Corner {
                        east: EastDecoration::from_open_tile(east),
                        south: SouthDecoration::from_open_tile(south),
                    },
                    (false, true) => MidSprite::Horizontal {
                        south: SouthDecoration::from_open_tile(south),
                    },
                    (true, false) => MidSprite::Vertical {
                        east: EastDecoration::from_open_tile(east),
                    },
                    (false, false) if south_east == TileVariant::Wall => MidSprite::Full,
                    (false, false) => MidSprite::InverseCorner,
                }
            }
            TileVariant::StairN => match east {
                TileVariant::Wall | TileVariant::StairN => MidSprite::StairNFull,
                _ => MidSprite::StairN,
            },
            TileVariant::StairS => match east {
                TileVariant::Wall | TileVariant::StairS => MidSprite::StairSFull,
                _ => MidSprite::StairS,
            },
            TileVariant::StairE => match south {
                TileVariant::Wall | TileVariant::StairS => MidSprite::StairEFull,
                _ => MidSprite::StairE,
            },
            TileVariant::StairW => match south {
                TileVariant::Wall | TileVariant::StairS => MidSprite::StairWFull,
                _ => MidSprite::StairW,
            },
        }
    }
}

impl EastDecoration {
    /// Maps a non-`Wall` tile to the decoration drawn on the east edge.
    fn from_open_tile(tile: TileVariant) -> Self {
        match tile {
            TileVariant::Empty | TileVariant::DoorV | TileVariant::StairE => EastDecoration::None,
            TileVariant::DoorH => EastDecoration::Door,
            TileVariant::StairN => EastDecoration::StairN,
            TileVariant::StairS => EastDecoration::StairS,
            TileVariant::Wall | TileVariant::StairW => {
                unreachable!("east tile is known to be non-Wall")
            }
        }
    }
}

impl SouthDecoration {
    /// Maps a non-`Wall` tile to the decoration drawn on the south edge.
    fn from_open_tile(tile: TileVariant) -> Self {
        match tile {
            TileVariant::Empty
            | TileVariant::DoorH
            | TileVariant::StairS
            | TileVariant::StairE
            | TileVariant::StairW => SouthDecoration::None,
            TileVariant::DoorV => SouthDecoration::Door,
            TileVariant::StairN => SouthDecoration::StairN,
            TileVariant::Wall => unreachable!("south tile is known to be non-Wall"),
        }
    }
}

impl TopSprite {
    pub fn resolve(
        center: TileVariant,
        _north: TileVariant,
        _north_east: TileVariant,
        east: TileVariant,
        south_east: TileVariant,
        south: TileVariant,
    ) -> Self {
        match south {
            TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairS => {
                TopSprite::Empty {
                    south_east: SouthEastTopDecoration::None,
                }
            }
            TileVariant::StairN => match south_east {
                TileVariant::StairN | TileVariant::Wall => TopSprite::StairNFull,
                _ => TopSprite::StairN,
            },
            TileVariant::Wall if south_east == TileVariant::Wall => TopSprite::Horizontal {
                deco: TopDecoration::from_center(center),
            },
            TileVariant::Wall if center == TileVariant::Wall => TopSprite::Empty {
                south_east: SouthEastTopDecoration::from_open_tile(south_east, east),
            },
            TileVariant::Wall => TopSprite::Corner {
                deco: TopDecoration::from_center(center),
                south_east: SouthEastTopDecoration::from_open_tile(south_east, east),
            },
            TileVariant::StairE => match center {
                TileVariant::StairE | TileVariant::Wall => TopSprite::StairEFull,
                _ => TopSprite::StairE,
            },
            TileVariant::StairW => match center {
                TileVariant::StairW | TileVariant::Wall => TopSprite::StairWFull,
                _ => TopSprite::StairW,
            },
        }
    }
}

impl TopDecoration {
    /// Maps the tile at the sprite's center to the decoration drawn on the top edge.
    fn from_center(center: TileVariant) -> Self {
        match center {
            TileVariant::DoorV => TopDecoration::Door,
            _ => TopDecoration::None,
        }
    }
}

impl SouthEastTopDecoration {
    /// Maps a non-`Wall` south-east tile to the corner decoration drawn there.
    fn from_open_tile(tile: TileVariant, north: TileVariant) -> Self {
        match tile {
            TileVariant::StairN => SouthEastTopDecoration::StairN,
            TileVariant::StairW => match north {
                TileVariant::StairW | TileVariant::Wall => SouthEastTopDecoration::StairWFull,
                _ => SouthEastTopDecoration::StairW,
            },
            _ => SouthEastTopDecoration::None,
        }
    }
}

impl fmt::Display for MidSprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MidSprite::Empty => write!(f, "Empty"),
            MidSprite::Corner { east, south } => {
                write!(f, "Corner{east}{south}")
            }
            MidSprite::Horizontal { south } => write!(f, "Horizontal{south}"),
            MidSprite::Vertical { east } => write!(f, "Vertical{east}"),
            MidSprite::InverseCorner => write!(f, "InverseCorner"),
            MidSprite::Full => write!(f, "Full"),
            MidSprite::StairN => write!(f, "StairN"),
            MidSprite::StairNFull => write!(f, "StairNFull"),
            MidSprite::StairS => write!(f, "StairS"),
            MidSprite::StairSFull => write!(f, "StairSFull"),
            MidSprite::StairE => write!(f, "StairE"),
            MidSprite::StairEFull => write!(f, "StairEFull"),
            MidSprite::StairW => write!(f, "StairW"),
            MidSprite::StairWFull => write!(f, "StairWFull"),
        }
    }
}

impl fmt::Display for EastDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EastDecoration::None => write!(f, ""),
            EastDecoration::Door => write!(f, "_EastDoor"),
            EastDecoration::StairN => write!(f, "_EastStairN"),
            EastDecoration::StairS => write!(f, "_EastStairS"),
        }
    }
}

impl fmt::Display for SouthDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SouthDecoration::None => write!(f, ""),
            SouthDecoration::Door => write!(f, "_SouthDoor"),
            SouthDecoration::StairN => write!(f, "_SouthStairN"),
        }
    }
}

impl fmt::Display for TopSprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopSprite::Empty { south_east } => write!(f, "Empty{south_east}"),
            TopSprite::Horizontal { deco } => write!(f, "Horizontal{deco}"),
            TopSprite::Corner { deco, south_east } => {
                write!(f, "Corner{deco}{south_east}")
            }
            TopSprite::StairN => write!(f, "StairN"),
            TopSprite::StairNFull => write!(f, "StairNFull"),
            TopSprite::StairE => write!(f, "StairE"),
            TopSprite::StairEFull => write!(f, "StairEFull"),
            TopSprite::StairW => write!(f, "StairW"),
            TopSprite::StairWFull => write!(f, "StairWFull"),
        }
    }
}

impl fmt::Display for TopDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopDecoration::None => write!(f, ""),
            TopDecoration::Door => write!(f, "_Door"),
        }
    }
}

impl fmt::Display for SouthEastTopDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SouthEastTopDecoration::None => write!(f, ""),
            SouthEastTopDecoration::StairN => write!(f, "_SouthEastStairN"),
            SouthEastTopDecoration::StairW => write!(f, "_SouthEastStairW"),
            SouthEastTopDecoration::StairWFull => write!(f, "_SouthEastStairWFull"),
        }
    }
}
