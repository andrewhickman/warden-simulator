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
pub enum BaseSprite {
    Empty,
    Wall {
        adjacency: Adjacency,
        east: BaseSpriteEast,
        south: BaseSpriteSouth,
    },
    StairN {
        adjacency: Adjacency,
    },
    StairS {
        adjacency: Adjacency,
        south: BaseSpriteSouth,
    },
    StairE {
        adjacency: Adjacency,
        east: BaseSpriteEast,
    },
    StairW {
        adjacency: Adjacency,
    },
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum BaseSpriteEast {
    None,
    Door,
    StairN,
    StairS,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum BaseSpriteSouth {
    None,
    Door,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Adjacency {
    Corner,
    Horizontal,
    Vertical,
    InverseCorner,
    Full,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TopSprite {
    Empty {
        east: TopSpriteEast,
    },
    Wall {
        adjacency: Adjacency,
        east: TopSpriteEast,
        north: TopSpriteNorth,
    },
    StairN {
        adjacency: Adjacency,
        north: TopSpriteNorth,
    },
    StairE {
        adjacency: Adjacency,
        east: TopSpriteEast,
    },
    StairW {
        adjacency: Adjacency,
    },
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TopSpriteNorth {
    None,
    Door,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TopSpriteEast {
    None,
    StairN,
}

impl TileVariant {
    pub fn values() -> [TileVariant; 8] {
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

impl BaseSprite {
    pub fn resolve(
        center: TileVariant,
        east: TileVariant,
        south: TileVariant,
        south_east: TileVariant,
    ) -> BaseSprite {
        match center {
            TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV => BaseSprite::Empty,
            TileVariant::Wall => BaseSprite::Wall {
                adjacency: Adjacency::resolve(
                    matches!(south, TileVariant::Wall | TileVariant::StairN),
                    matches!(east, TileVariant::Wall | TileVariant::StairW),
                    matches!(
                        south_east,
                        TileVariant::Wall | TileVariant::StairN | TileVariant::StairW
                    ),
                ),
                east: BaseSpriteEast::resolve(east),
                south: BaseSpriteSouth::resolve(south),
            },
            TileVariant::StairN => BaseSprite::StairN {
                adjacency: Adjacency::resolve(
                    false,
                    matches!(
                        east,
                        TileVariant::Wall | TileVariant::StairW | TileVariant::StairE
                    ),
                    false,
                ),
            },
            TileVariant::StairS => BaseSprite::StairS {
                adjacency: Adjacency::resolve(
                    matches!(south, TileVariant::Wall | TileVariant::StairN),
                    matches!(
                        east,
                        TileVariant::Wall | TileVariant::StairW | TileVariant::StairS
                    ),
                    matches!(
                        south_east,
                        TileVariant::Wall | TileVariant::StairN | TileVariant::StairW
                    ),
                ),
                south: BaseSpriteSouth::resolve(south),
            },
            TileVariant::StairE => BaseSprite::StairE {
                adjacency: Adjacency::resolve(
                    matches!(
                        south,
                        TileVariant::Wall | TileVariant::StairN | TileVariant::StairE
                    ),
                    matches!(east, TileVariant::Wall | TileVariant::StairW),
                    matches!(
                        south_east,
                        TileVariant::Wall | TileVariant::StairN | TileVariant::StairW
                    ),
                ),
                east: BaseSpriteEast::resolve(east),
            },
            TileVariant::StairW => BaseSprite::StairW {
                adjacency: Adjacency::resolve(
                    matches!(
                        south,
                        TileVariant::Wall | TileVariant::StairN | TileVariant::StairW
                    ),
                    false,
                    false,
                ),
            },
        }
    }
}

impl BaseSpriteEast {
    pub fn resolve(east: TileVariant) -> BaseSpriteEast {
        match east {
            TileVariant::Empty
            | TileVariant::Wall
            | TileVariant::DoorV
            | TileVariant::StairW
            | TileVariant::StairE => BaseSpriteEast::None,
            TileVariant::DoorH => BaseSpriteEast::Door,
            TileVariant::StairN => BaseSpriteEast::StairN,
            TileVariant::StairS => BaseSpriteEast::StairS,
        }
    }
}

impl BaseSpriteSouth {
    pub fn resolve(south: TileVariant) -> BaseSpriteSouth {
        match south {
            TileVariant::Empty
            | TileVariant::Wall
            | TileVariant::DoorH
            | TileVariant::StairN
            | TileVariant::StairS
            | TileVariant::StairE
            | TileVariant::StairW => BaseSpriteSouth::None,
            TileVariant::DoorV => BaseSpriteSouth::Door,
        }
    }
}

impl Adjacency {
    pub fn resolve(south: bool, east: bool, south_east: bool) -> Adjacency {
        match (south, east, south_east) {
            (false, false, _) => Adjacency::Corner,
            (false, true, _) => Adjacency::Horizontal,
            (true, false, _) => Adjacency::Vertical,
            (true, true, false) => Adjacency::InverseCorner,
            (true, true, true) => Adjacency::Full,
        }
    }
}

impl TopSprite {
    pub const EMPTY: TopSprite = TopSprite::Empty {
        east: TopSpriteEast::None,
    };

    pub fn resolve(
        north: TileVariant,
        north_east: TileVariant,
        center: TileVariant,
        east: TileVariant,
    ) -> TopSprite {
        match center {
            TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairS => {
                TopSprite::Empty {
                    east: TopSpriteEast::resolve(center, east),
                }
            }
            TileVariant::StairE | TileVariant::StairW | TileVariant::Wall if center == north => {
                TopSprite::Empty {
                    east: TopSpriteEast::resolve(center, east),
                }
            }
            TileVariant::Wall => TopSprite::Wall {
                adjacency: Adjacency::resolve(
                    false,
                    matches!(east, TileVariant::Wall | TileVariant::StairW),
                    false,
                ),
                north: TopSpriteNorth::resolve(north),
                east: TopSpriteEast::resolve(center, east),
            },
            TileVariant::StairN => TopSprite::StairN {
                adjacency: Adjacency::resolve(
                    matches!(north, TileVariant::Wall | TileVariant::StairS),
                    matches!(
                        east,
                        TileVariant::Wall | TileVariant::StairW | TileVariant::StairN
                    ),
                    matches!(
                        north_east,
                        TileVariant::Wall | TileVariant::StairW | TileVariant::StairS
                    ),
                ),
                north: TopSpriteNorth::resolve(north),
            },
            TileVariant::StairE => TopSprite::StairE {
                adjacency: Adjacency::resolve(
                    matches!(north, TileVariant::Wall | TileVariant::StairS),
                    matches!(east, TileVariant::Wall | TileVariant::StairW),
                    matches!(
                        north_east,
                        TileVariant::Wall | TileVariant::StairW | TileVariant::StairS
                    ),
                ),
                east: TopSpriteEast::resolve(center, east),
            },
            TileVariant::StairW => TopSprite::StairW {
                adjacency: Adjacency::resolve(
                    matches!(north, TileVariant::Wall | TileVariant::StairS),
                    false,
                    false,
                ),
            },
        }
    }
}

impl TopSpriteEast {
    pub fn resolve(center: TileVariant, east: TileVariant) -> TopSpriteEast {
        match east {
            TileVariant::StairN if matches!(center, TileVariant::Wall | TileVariant::StairE) => {
                TopSpriteEast::StairN
            }
            _ => TopSpriteEast::None,
        }
    }
}

impl TopSpriteNorth {
    pub fn resolve(north: TileVariant) -> TopSpriteNorth {
        match north {
            TileVariant::Empty
            | TileVariant::Wall
            | TileVariant::DoorH
            | TileVariant::StairN
            | TileVariant::StairS
            | TileVariant::StairW
            | TileVariant::StairE => TopSpriteNorth::None,
            TileVariant::DoorV => TopSpriteNorth::Door,
        }
    }
}

impl fmt::Display for BaseSprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseSprite::Empty => write!(f, "EMPTY"),
            BaseSprite::Wall {
                adjacency,
                east,
                south,
            } => write!(f, "BASE_WALL{adjacency}{east}{south}"),
            BaseSprite::StairN { adjacency } => write!(f, "BASE_STAIRNORTH{adjacency}"),
            BaseSprite::StairS { adjacency, south } => {
                write!(f, "BASE_STAIRSOUTH{adjacency}{south}")
            }
            BaseSprite::StairE { adjacency, east } => write!(f, "BASE_STAIREAST{adjacency}{east}"),
            BaseSprite::StairW { adjacency } => write!(f, "BASE_STAIRWEST{adjacency}"),
        }
    }
}

impl fmt::Display for Adjacency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Adjacency::Corner => write!(f, "CORNER"),
            Adjacency::Horizontal => write!(f, "HORIZONTAL"),
            Adjacency::Vertical => write!(f, "VERTICAL"),
            Adjacency::InverseCorner => write!(f, "INVERSECORNER"),
            Adjacency::Full => write!(f, "FULL"),
        }
    }
}

impl fmt::Display for BaseSpriteEast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseSpriteEast::None => write!(f, ""),
            BaseSpriteEast::Door => write!(f, "_EASTDOOR"),
            BaseSpriteEast::StairN => write!(f, "_EASTSTAIRNORTH"),
            BaseSpriteEast::StairS => write!(f, "_EASTSTAIRSOUTH"),
        }
    }
}

impl fmt::Display for BaseSpriteSouth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseSpriteSouth::None => write!(f, ""),
            BaseSpriteSouth::Door => write!(f, "_SOUTHDOOR"),
        }
    }
}

impl fmt::Display for TopSprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopSprite::Empty {
                east: TopSpriteEast::None,
            } => write!(f, "EMPTY"),
            TopSprite::Empty { east } => write!(f, "TOP_EMPTY{east}"),
            TopSprite::Wall {
                adjacency,
                east,
                north,
            } => write!(f, "TOP_WALL{adjacency}{east}{north}"),
            TopSprite::StairN { adjacency, north } => write!(f, "TOP_STAIRNORTH{adjacency}{north}"),
            TopSprite::StairE { adjacency, east } => write!(f, "TOP_STAIREAST{adjacency}{east}"),
            TopSprite::StairW { adjacency } => write!(f, "TOP_STAIRWEST{adjacency}"),
        }
    }
}

impl fmt::Display for TopSpriteEast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopSpriteEast::None => write!(f, ""),
            TopSpriteEast::StairN => write!(f, "_EASTSTAIRNORTH"),
        }
    }
}

impl fmt::Display for TopSpriteNorth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopSpriteNorth::None => write!(f, ""),
            TopSpriteNorth::Door => write!(f, "_NORTHDOOR"),
        }
    }
}
