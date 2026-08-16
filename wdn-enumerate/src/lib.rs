use std::fmt;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TileVariant {
    Empty,
    Wall,
    DoorH,
    DoorV,
    StairN,
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
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, PartialOrd, Ord)]
pub enum EastDecoration {
    None,
    Door,
    StairN,
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
        ]
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
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN,
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
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN,
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
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN,
            ) => MidSprite::Corner {
                east: EastDecoration::StairN,
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
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Wall,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN,
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
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Wall,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairN,
                TileVariant::Wall,
            ) => MidSprite::InverseCorner,
            (TileVariant::Wall, _, _, TileVariant::Wall, TileVariant::Wall, TileVariant::Wall) => {
                MidSprite::Full
            }
            (
                TileVariant::StairN,
                _,
                _,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV,
            ) => MidSprite::StairN,
            (TileVariant::StairN, _, _, _, _, TileVariant::Wall | TileVariant::StairN) => {
                MidSprite::StairNFull
            }
        }
    }

    pub fn id(&self) -> u16 {
        use MidSprite::*;

        match self {
            Empty => 0,
            Corner {
                east: EastDecoration::None,
                south: SouthDecoration::None,
            } => 1,
            Corner {
                east: EastDecoration::None,
                south: SouthDecoration::Door,
            } => 2,
            Corner {
                east: EastDecoration::Door,
                south: SouthDecoration::None,
            } => 3,
            Corner {
                east: EastDecoration::Door,
                south: SouthDecoration::Door,
            } => 4,
            Corner {
                east: EastDecoration::StairN,
                south: SouthDecoration::None,
            } => 5,
            Corner {
                east: EastDecoration::StairN,
                south: SouthDecoration::Door,
            } => 6,
            Horizontal {
                south: SouthDecoration::None,
            } => 7,
            Horizontal {
                south: SouthDecoration::Door,
            } => 8,
            Vertical {
                east: EastDecoration::None,
            } => 9,
            Vertical {
                east: EastDecoration::Door,
            } => 10,
            Vertical {
                east: EastDecoration::StairN,
            } => 11,
            InverseCorner => 12,
            Full => 13,
            StairN => 14,
            StairNFull => 15,
        }
    }
}

impl TopSprite {
    pub fn resolve(
        center: TileVariant,
        north: TileVariant,
        north_east: TileVariant,
        east: TileVariant,
        south_east: TileVariant,
        south: TileVariant,
    ) -> Self {
        match (center, north, north_east, east, south_east, south) {
            (
                TileVariant::Wall,
                _,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairN,
                _,
            )
            | (_, _, _, _, _, TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV) => {
                TopSprite::Empty {
                    south_east: SouthEastTopDecoration::None,
                }
            }
            (_, _, _, _, _, TileVariant::StairN) => TopSprite::Empty {
                south_east: SouthEastTopDecoration::StairN,
            },
            (
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN,
                _,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV,
                TileVariant::Wall,
            ) => TopSprite::Corner {
                deco: TopDecoration::None,
                south_east: SouthEastTopDecoration::None,
            },
            (
                TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN,
                _,
                _,
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
                _,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV,
                TileVariant::Wall,
            ) => TopSprite::Corner {
                deco: TopDecoration::Door,
                south_east: SouthEastTopDecoration::None,
            },
            (TileVariant::DoorV, _, _, _, TileVariant::StairN, TileVariant::Wall) => {
                TopSprite::Corner {
                    deco: TopDecoration::Door,
                    south_east: SouthEastTopDecoration::StairN,
                }
            }
            (
                TileVariant::Wall | TileVariant::Empty | TileVariant::DoorH | TileVariant::StairN,
                _,
                _,
                _,
                TileVariant::Wall,
                TileVariant::Wall,
            ) => TopSprite::Horizontal {
                deco: TopDecoration::None,
            },
            (TileVariant::DoorV, _, _, _, TileVariant::Wall, TileVariant::Wall) => {
                TopSprite::Horizontal {
                    deco: TopDecoration::Door,
                }
            }
        }
    }

    pub fn id(&self) -> u16 {
        use TopSprite::*;

        match self {
            Empty {
                south_east: SouthEastTopDecoration::None,
            } => 0,
            Empty {
                south_east: SouthEastTopDecoration::StairN,
            } => 16,
            Corner {
                south_east: SouthEastTopDecoration::None,
                deco: TopDecoration::None,
            } => 17,
            Corner {
                south_east: SouthEastTopDecoration::None,
                deco: TopDecoration::Door,
            } => 18,
            Corner {
                south_east: SouthEastTopDecoration::StairN,
                deco: TopDecoration::None,
            } => 19,
            Corner {
                south_east: SouthEastTopDecoration::StairN,
                deco: TopDecoration::Door,
            } => 20,
            Horizontal {
                deco: TopDecoration::None,
            } => 21,
            Horizontal {
                deco: TopDecoration::Door,
            } => 22,
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
        }
    }
}

impl fmt::Display for EastDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EastDecoration::None => write!(f, ""),
            EastDecoration::Door => write!(f, "_EastDoor"),
            EastDecoration::StairN => write!(f, "_EastStairN"),
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
