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
        north: TileVariant,
        north_east: TileVariant,
        east: TileVariant,
        south_east: TileVariant,
        south: TileVariant,
    ) -> Self {
        match (center, north, north_east, east, south_east, south) {
            (TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV, _, _, _, _, _) => {
                MidSprite::Empty
            }
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorV,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN | TileVariant::StairS,
            ) => MidSprite::Corner {
                east: EastDecoration::None,
                south: SouthDecoration::None,
            },
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorV,
                _,
                TileVariant::DoorV,
            ) => MidSprite::Corner {
                east: EastDecoration::None,
                south: SouthDecoration::Door,
            },
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::DoorH,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN | TileVariant::StairS,
            ) => MidSprite::Corner {
                east: EastDecoration::Door,
                south: SouthDecoration::None,
            },
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::StairN,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN | TileVariant::StairS,
            ) => MidSprite::Corner {
                east: EastDecoration::StairN,
                south: SouthDecoration::None,
            },
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::StairS,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN | TileVariant::StairS,
            ) => MidSprite::Corner {
                east: EastDecoration::StairS,
                south: SouthDecoration::None,
            },
            (TileVariant::Wall, _, _, TileVariant::DoorH, _, TileVariant::DoorV) => {
                MidSprite::Corner {
                    east: EastDecoration::Door,
                    south: SouthDecoration::Door,
                }
            }
            (TileVariant::Wall, _, _, TileVariant::StairN, _, TileVariant::DoorV) => {
                MidSprite::Corner {
                    east: EastDecoration::StairN,
                    south: SouthDecoration::Door,
                }
            }
            (TileVariant::Wall, _, _, TileVariant::StairS, _, TileVariant::DoorV) => {
                MidSprite::Corner {
                    east: EastDecoration::StairS,
                    south: SouthDecoration::Door,
                }
            }
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Wall,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN | TileVariant::StairS,
            ) => MidSprite::Horizontal {
                south: SouthDecoration::None,
            },
            (TileVariant::Wall, _, _, TileVariant::Wall, _, TileVariant::DoorV) => {
                MidSprite::Horizontal {
                    south: SouthDecoration::Door,
                }
            }
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorV,
                _,
                TileVariant::Wall,
            ) => MidSprite::Vertical {
                east: EastDecoration::None,
            },
            (TileVariant::Wall, _, _, TileVariant::DoorH, _, TileVariant::Wall) => {
                MidSprite::Vertical {
                    east: EastDecoration::Door,
                }
            }
            (TileVariant::Wall, _, _, TileVariant::StairN, _, TileVariant::Wall) => {
                MidSprite::Vertical {
                    east: EastDecoration::StairN,
                }
            }
            (TileVariant::Wall, _, _, TileVariant::StairS, _, TileVariant::Wall) => {
                MidSprite::Vertical {
                    east: EastDecoration::StairS,
                }
            }
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Wall,
                TileVariant::Empty
                | TileVariant::DoorH
                | TileVariant::DoorV
                | TileVariant::StairN
                | TileVariant::StairS,
                TileVariant::Wall,
            ) => MidSprite::InverseCorner,
            (TileVariant::Wall, _, _, TileVariant::Wall, TileVariant::Wall, TileVariant::Wall) => {
                MidSprite::Full
            }
            (
                TileVariant::StairN,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairS,
                _,
                _,
            ) => MidSprite::StairN,
            (TileVariant::StairN, _, _, TileVariant::Wall | TileVariant::StairN, _, _) => {
                MidSprite::StairNFull
            }
            (
                TileVariant::StairS,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairN,
                _,
                _,
            ) => MidSprite::StairS,
            (TileVariant::StairS, _, _, TileVariant::Wall | TileVariant::StairS, _, _) => {
                MidSprite::StairSFull
            }
        }
    }
}

impl TopSprite {
    pub fn resolve(
        center: TileVariant,
        _: TileVariant,
        _: TileVariant,
        east: TileVariant,
        south_east: TileVariant,
        south: TileVariant,
    ) -> Self {
        match (center, east, south_east, south) {
            (
                TileVariant::Wall,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairS,
                TileVariant::Empty
                | TileVariant::Wall
                | TileVariant::DoorH
                | TileVariant::DoorV
                | TileVariant::StairS,
            )
            | (
                _,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairS,
            ) => TopSprite::Empty {
                south_east: SouthEastTopDecoration::None,
            },
            (TileVariant::Wall, _, TileVariant::StairN, TileVariant::Wall) => TopSprite::Empty {
                south_east: SouthEastTopDecoration::StairN,
            },
            (
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN | TileVariant::StairS,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairS,
                TileVariant::Wall,
            ) => TopSprite::Corner {
                deco: TopDecoration::None,
                south_east: SouthEastTopDecoration::None,
            },
            (
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN | TileVariant::StairS,
                _,
                TileVariant::StairN,
                TileVariant::Wall,
            ) => TopSprite::Corner {
                deco: TopDecoration::None,
                south_east: SouthEastTopDecoration::StairN,
            },
            (
                TileVariant::DoorV,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairS,
                TileVariant::Wall,
            ) => TopSprite::Corner {
                deco: TopDecoration::Door,
                south_east: SouthEastTopDecoration::None,
            },
            (TileVariant::DoorV, _, TileVariant::StairN, TileVariant::Wall) => TopSprite::Corner {
                deco: TopDecoration::Door,
                south_east: SouthEastTopDecoration::StairN,
            },
            (
                TileVariant::Wall
                | TileVariant::Empty
                | TileVariant::DoorH
                | TileVariant::StairN
                | TileVariant::StairS,
                _,
                TileVariant::Wall,
                TileVariant::Wall,
            ) => TopSprite::Horizontal {
                deco: TopDecoration::None,
            },
            (TileVariant::DoorV, _, TileVariant::Wall, TileVariant::Wall) => {
                TopSprite::Horizontal {
                    deco: TopDecoration::Door,
                }
            }
            (
                _,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairS,
                TileVariant::StairN,
            ) => TopSprite::StairN,
            (_, _, TileVariant::StairN | TileVariant::Wall, TileVariant::StairN) => {
                TopSprite::StairNFull
            }
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
        }
    }
}
