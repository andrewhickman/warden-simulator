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

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
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

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum BaseSpriteEast {
    None,
    Door,
    StairN,
    StairS,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum BaseSpriteSouth {
    None,
    Door,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum Adjacency {
    Corner,
    Horizontal,
    Vertical,
    InverseCorner,
    Full,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum TopSprite {
    Empty {
        east: TopSpriteEast,
        overlap: OverlapBase,
    },
    Wall {
        adjacency: Adjacency,
        east: TopSpriteEast,
        north: TopSpriteNorth,
        overlap: OverlapBase,
    },
    StairN {
        adjacency: Adjacency,
        north: TopSpriteNorth,
        overlap: OverlapBase,
    },
    StairE {
        adjacency: Adjacency,
        east: TopSpriteEast,
        overlap: OverlapBase,
    },
    StairW {
        adjacency: Adjacency,
        overlap: OverlapBase,
    },
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum TopSpriteNorth {
    None,
    Door,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum TopSpriteEast {
    None,
    StairN,
}

/// The base sprite a top sprite visually overlaps with, as found by `wdn-enumerate`'s mask analysis.
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum OverlapBase {
    None,
    WallCorner,
    WallHorizontal,
    WallVertical,
    WallFull,
    WallCornerEastDoor,
    WallCornerEastStairN,
    WallCornerEastStairS,
    WallVerticalEastDoor,
    WallVerticalEastStairN,
    WallVerticalEastStairS,
    StairSouthCorner,
    StairSouthHorizontal,
    StairSouthVertical,
    StairSouthFull,
    StairSouthInverseCorner,
    StairEastVertical,
    StairEastInverseCorner,
    StairEastFull,
    StairEastVerticalEastDoor,
    StairEastVerticalEastStairN,
    StairEastVerticalEastStairS,
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
                        TileVariant::Wall | TileVariant::StairW | TileVariant::StairN
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
        overlap: OverlapBase::None,
    };

    pub fn resolve(
        north: TileVariant,
        _: TileVariant,
        center: TileVariant,
        east: TileVariant,
        base: BaseSprite,
    ) -> TopSprite {
        let overlap = OverlapBase::resolve(base);
        let sprite = match center {
            TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV | TileVariant::StairS => {
                let east = TopSpriteEast::resolve(center, east);
                TopSprite::Empty {
                    east,
                    overlap: if east == TopSpriteEast::None {
                        OverlapBase::None
                    } else {
                        overlap
                    },
                }
            }
            TileVariant::StairE | TileVariant::StairW if center == north => {
                let east = TopSpriteEast::resolve(center, east);
                TopSprite::Empty {
                    east,
                    overlap: if east == TopSpriteEast::None {
                        OverlapBase::None
                    } else {
                        overlap
                    },
                }
            }
            TileVariant::Wall
                if center == north && !matches!(east, TileVariant::Wall | TileVariant::StairW) =>
            {
                let east = TopSpriteEast::resolve(center, east);
                TopSprite::Empty {
                    east,
                    overlap: if east == TopSpriteEast::None {
                        OverlapBase::None
                    } else {
                        overlap
                    },
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
                overlap,
            },
            TileVariant::StairN => {
                let adjacency = Adjacency::resolve(
                    matches!(north, TileVariant::Wall | TileVariant::StairS),
                    matches!(
                        east,
                        TileVariant::Wall | TileVariant::StairW | TileVariant::StairN
                    ),
                    true,
                );
                TopSprite::StairN {
                    adjacency,
                    north: TopSpriteNorth::resolve(north),
                    overlap: if adjacency == Adjacency::Vertical {
                        OverlapBase::None
                    } else {
                        overlap
                    },
                }
            }
            TileVariant::StairE => TopSprite::StairE {
                adjacency: Adjacency::resolve(
                    matches!(north, TileVariant::Wall | TileVariant::StairS),
                    matches!(east, TileVariant::Wall | TileVariant::StairW),
                    true,
                ),
                east: TopSpriteEast::resolve(center, east),
                overlap,
            },
            TileVariant::StairW => TopSprite::StairW {
                adjacency: Adjacency::resolve(
                    matches!(north, TileVariant::Wall | TileVariant::StairS),
                    false,
                    true,
                ),
                overlap: OverlapBase::None,
            },
        };

        sprite.normalize()
    }

    /// Reduces overlap variants that `wdn-enumerate`'s patch analysis found to be
    /// pixel-identical down to a single canonical representative.
    pub fn normalize(self) -> TopSprite {
        match self {
            TopSprite::Empty {
                east: TopSpriteEast::StairN,
                overlap,
            } => TopSprite::Empty {
                east: TopSpriteEast::StairN,
                overlap: match overlap {
                    OverlapBase::StairEastFull => OverlapBase::WallFull,
                    OverlapBase::WallVerticalEastDoor
                    | OverlapBase::WallVerticalEastStairN
                    | OverlapBase::StairEastVertical
                    | OverlapBase::StairEastVerticalEastDoor
                    | OverlapBase::StairEastVerticalEastStairN
                    | OverlapBase::StairEastVerticalEastStairS => OverlapBase::WallVertical,
                    other => other,
                },
            },
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::None,
                north,
                overlap,
            } => TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::None,
                north,
                overlap: match overlap {
                    OverlapBase::StairEastInverseCorner
                    | OverlapBase::StairEastVerticalEastDoor
                    | OverlapBase::StairEastVerticalEastStairN
                    | OverlapBase::StairEastVerticalEastStairS => OverlapBase::StairEastVertical,
                    OverlapBase::StairSouthInverseCorner => OverlapBase::StairSouthVertical,
                    other => other,
                },
            },
            TopSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
                north,
                overlap,
            } => TopSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
                north,
                overlap: match overlap {
                    OverlapBase::StairEastVerticalEastDoor
                    | OverlapBase::StairEastVerticalEastStairN
                    | OverlapBase::StairEastVerticalEastStairS => OverlapBase::StairEastVertical,
                    other => other,
                },
            },
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
                north,
                overlap,
            } => TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
                north,
                overlap: match overlap {
                    OverlapBase::StairEastVerticalEastDoor
                    | OverlapBase::StairEastVerticalEastStairN
                    | OverlapBase::StairEastVerticalEastStairS => OverlapBase::StairEastVertical,
                    other => other,
                },
            },
            TopSprite::StairN {
                adjacency: Adjacency::Full,
                north,
                overlap,
            } => TopSprite::StairN {
                adjacency: Adjacency::Full,
                north,
                overlap: match overlap {
                    OverlapBase::WallVerticalEastDoor
                    | OverlapBase::WallVerticalEastStairN
                    | OverlapBase::StairSouthVertical => OverlapBase::WallVertical,
                    other => other,
                },
            },
            TopSprite::StairN {
                adjacency: Adjacency::Horizontal,
                north,
                overlap,
            } => TopSprite::StairN {
                adjacency: Adjacency::Horizontal,
                north,
                overlap: match overlap {
                    OverlapBase::StairEastVerticalEastDoor
                    | OverlapBase::StairEastVerticalEastStairN
                    | OverlapBase::StairEastVerticalEastStairS => OverlapBase::StairEastVertical,
                    other => other,
                },
            },
            TopSprite::StairN {
                adjacency: Adjacency::Corner,
                north,
                overlap,
            } => TopSprite::StairN {
                adjacency: Adjacency::Corner,
                north,
                overlap: match overlap {
                    OverlapBase::StairEastInverseCorner
                    | OverlapBase::StairEastVerticalEastDoor
                    | OverlapBase::StairEastVerticalEastStairN
                    | OverlapBase::StairEastVerticalEastStairS => OverlapBase::StairEastVertical,
                    other => other,
                },
            },
            TopSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: TopSpriteEast::None,
                overlap,
            } => TopSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: TopSpriteEast::None,
                overlap: match overlap {
                    OverlapBase::None => OverlapBase::None,
                    _ => OverlapBase::WallCorner,
                },
            },
            TopSprite::StairE {
                adjacency: Adjacency::Full,
                east: TopSpriteEast::None,
                overlap,
            } => TopSprite::StairE {
                adjacency: Adjacency::Full,
                east: TopSpriteEast::None,
                overlap: match overlap {
                    OverlapBase::WallCornerEastDoor
                    | OverlapBase::WallCornerEastStairN
                    | OverlapBase::StairSouthCorner => OverlapBase::WallCorner,
                    OverlapBase::WallCornerEastStairS | OverlapBase::StairSouthHorizontal => {
                        OverlapBase::WallHorizontal
                    }
                    other => other,
                },
            },
            TopSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: TopSpriteEast::StairN,
                overlap,
            } => TopSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: TopSpriteEast::StairN,
                overlap: match overlap {
                    OverlapBase::WallCornerEastStairS | OverlapBase::StairSouthHorizontal => {
                        OverlapBase::WallHorizontal
                    }
                    OverlapBase::WallCornerEastDoor
                    | OverlapBase::WallCornerEastStairN
                    | OverlapBase::StairSouthCorner => OverlapBase::WallCorner,
                    other => other,
                },
            },
            other => other,
        }
    }

    /// The overlap category stored on this sprite, regardless of its variant.
    pub fn overlap(self) -> OverlapBase {
        match self {
            TopSprite::Empty { overlap, .. }
            | TopSprite::Wall { overlap, .. }
            | TopSprite::StairN { overlap, .. }
            | TopSprite::StairE { overlap, .. }
            | TopSprite::StairW { overlap, .. } => overlap,
        }
    }

    /// This sprite with its overlap category cleared, i.e. the original pre-`OverlapBase` shape.
    pub fn without_overlap(self) -> TopSprite {
        match self {
            TopSprite::Empty { east, .. } => TopSprite::Empty {
                east,
                overlap: OverlapBase::None,
            },
            TopSprite::Wall {
                adjacency,
                east,
                north,
                ..
            } => TopSprite::Wall {
                adjacency,
                east,
                north,
                overlap: OverlapBase::None,
            },
            TopSprite::StairN {
                adjacency, north, ..
            } => TopSprite::StairN {
                adjacency,
                north,
                overlap: OverlapBase::None,
            },
            TopSprite::StairE {
                adjacency, east, ..
            } => TopSprite::StairE {
                adjacency,
                east,
                overlap: OverlapBase::None,
            },
            TopSprite::StairW { adjacency, .. } => TopSprite::StairW {
                adjacency,
                overlap: OverlapBase::None,
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

impl OverlapBase {
    /// Maps a base sprite to the overlap category it belongs to, if any.
    pub fn resolve(base: BaseSprite) -> OverlapBase {
        match base {
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            } => OverlapBase::WallCorner,
            BaseSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            } => OverlapBase::WallHorizontal,
            BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            } => OverlapBase::WallVertical,
            BaseSprite::Wall {
                adjacency: Adjacency::Full,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            } => OverlapBase::WallFull,
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::Door,
                south: BaseSpriteSouth::None,
            } => OverlapBase::WallCornerEastDoor,
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::StairN,
                south: BaseSpriteSouth::None,
            } => OverlapBase::WallCornerEastStairN,
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::StairS,
                south: BaseSpriteSouth::None,
            } => OverlapBase::WallCornerEastStairS,
            BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::Door,
                south: BaseSpriteSouth::None,
            } => OverlapBase::WallVerticalEastDoor,
            BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairN,
                south: BaseSpriteSouth::None,
            } => OverlapBase::WallVerticalEastStairN,
            BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairS,
                south: BaseSpriteSouth::None,
            } => OverlapBase::WallVerticalEastStairS,
            BaseSprite::StairS {
                adjacency: Adjacency::Corner,
                south: BaseSpriteSouth::None,
            } => OverlapBase::StairSouthCorner,
            BaseSprite::StairS {
                adjacency: Adjacency::Horizontal,
                south: BaseSpriteSouth::None,
            } => OverlapBase::StairSouthHorizontal,
            BaseSprite::StairS {
                adjacency: Adjacency::Vertical,
                south: BaseSpriteSouth::None,
            } => OverlapBase::StairSouthVertical,
            BaseSprite::StairS {
                adjacency: Adjacency::Full,
                south: BaseSpriteSouth::None,
            } => OverlapBase::StairSouthFull,
            BaseSprite::StairS {
                adjacency: Adjacency::InverseCorner,
                south: BaseSpriteSouth::None,
            } => OverlapBase::StairSouthInverseCorner,
            BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::None,
            } => OverlapBase::StairEastVertical,
            BaseSprite::StairE {
                adjacency: Adjacency::InverseCorner,
                east: BaseSpriteEast::None,
            } => OverlapBase::StairEastInverseCorner,
            BaseSprite::StairE {
                adjacency: Adjacency::Full,
                east: BaseSpriteEast::None,
            } => OverlapBase::StairEastFull,
            BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::Door,
            } => OverlapBase::StairEastVerticalEastDoor,
            BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairN,
            } => OverlapBase::StairEastVerticalEastStairN,
            BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairS,
            } => OverlapBase::StairEastVerticalEastStairS,
            _ => OverlapBase::None,
        }
    }

    /// The base sprite this overlap category was derived from.
    pub fn base_sprite(self) -> BaseSprite {
        match self {
            OverlapBase::None => BaseSprite::Empty,
            OverlapBase::WallCorner => BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::WallHorizontal => BaseSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::WallVertical => BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::WallFull => BaseSprite::Wall {
                adjacency: Adjacency::Full,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::WallCornerEastDoor => BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::Door,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::WallCornerEastStairN => BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::StairN,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::WallCornerEastStairS => BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::StairS,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::WallVerticalEastDoor => BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::Door,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::WallVerticalEastStairN => BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairN,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::WallVerticalEastStairS => BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairS,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::StairSouthCorner => BaseSprite::StairS {
                adjacency: Adjacency::Corner,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::StairSouthHorizontal => BaseSprite::StairS {
                adjacency: Adjacency::Horizontal,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::StairSouthVertical => BaseSprite::StairS {
                adjacency: Adjacency::Vertical,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::StairSouthFull => BaseSprite::StairS {
                adjacency: Adjacency::Full,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::StairSouthInverseCorner => BaseSprite::StairS {
                adjacency: Adjacency::InverseCorner,
                south: BaseSpriteSouth::None,
            },
            OverlapBase::StairEastVertical => BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::None,
            },
            OverlapBase::StairEastInverseCorner => BaseSprite::StairE {
                adjacency: Adjacency::InverseCorner,
                east: BaseSpriteEast::None,
            },
            OverlapBase::StairEastFull => BaseSprite::StairE {
                adjacency: Adjacency::Full,
                east: BaseSpriteEast::None,
            },
            OverlapBase::StairEastVerticalEastDoor => BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::Door,
            },
            OverlapBase::StairEastVerticalEastStairN => BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairN,
            },
            OverlapBase::StairEastVerticalEastStairS => BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairS,
            },
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

impl fmt::Debug for BaseSprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseSprite::Empty => write!(f, "BaseSprite::Empty"),
            BaseSprite::Wall {
                adjacency,
                east,
                south,
            } => f
                .debug_struct("BaseSprite::Wall")
                .field("adjacency", adjacency)
                .field("east", east)
                .field("south", south)
                .finish(),
            BaseSprite::StairN { adjacency } => f
                .debug_struct("BaseSprite::StairN")
                .field("adjacency", adjacency)
                .finish(),
            BaseSprite::StairS { adjacency, south } => f
                .debug_struct("BaseSprite::StairS")
                .field("adjacency", adjacency)
                .field("south", south)
                .finish(),
            BaseSprite::StairE { adjacency, east } => f
                .debug_struct("BaseSprite::StairE")
                .field("adjacency", adjacency)
                .field("east", east)
                .finish(),
            BaseSprite::StairW { adjacency } => f
                .debug_struct("BaseSprite::StairW")
                .field("adjacency", adjacency)
                .finish(),
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

impl fmt::Debug for Adjacency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Adjacency::Corner => write!(f, "Adjacency::Corner"),
            Adjacency::Horizontal => write!(f, "Adjacency::Horizontal"),
            Adjacency::Vertical => write!(f, "Adjacency::Vertical"),
            Adjacency::InverseCorner => write!(f, "Adjacency::InverseCorner"),
            Adjacency::Full => write!(f, "Adjacency::Full"),
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

impl fmt::Debug for BaseSpriteEast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseSpriteEast::None => write!(f, "BaseSpriteEast::None"),
            BaseSpriteEast::Door => write!(f, "BaseSpriteEast::Door"),
            BaseSpriteEast::StairN => write!(f, "BaseSpriteEast::StairN"),
            BaseSpriteEast::StairS => write!(f, "BaseSpriteEast::StairS"),
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

impl fmt::Debug for BaseSpriteSouth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseSpriteSouth::None => write!(f, "BaseSpriteSouth::None"),
            BaseSpriteSouth::Door => write!(f, "BaseSpriteSouth::Door"),
        }
    }
}

impl fmt::Display for TopSprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::None,
            } => write!(f, "EMPTY"),
            TopSprite::Empty { east, overlap } => write!(f, "TOP_EMPTY{east}{overlap}"),
            TopSprite::Wall {
                adjacency,
                east,
                north,
                overlap,
            } => write!(f, "TOP_WALL{adjacency}{east}{north}{overlap}"),
            TopSprite::StairN {
                adjacency,
                north,
                overlap,
            } => write!(f, "TOP_STAIRNORTH{adjacency}{north}{overlap}"),
            TopSprite::StairE {
                adjacency,
                east,
                overlap,
            } => write!(f, "TOP_STAIREAST{adjacency}{east}{overlap}"),
            TopSprite::StairW { adjacency, overlap } => {
                write!(f, "TOP_STAIRWEST{adjacency}{overlap}")
            }
        }
    }
}

impl fmt::Debug for TopSprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopSprite::Empty { east, overlap } => f
                .debug_struct("TopSprite::Empty")
                .field("east", east)
                .field("overlap", overlap)
                .finish(),
            TopSprite::Wall {
                adjacency,
                east,
                north,
                overlap,
            } => f
                .debug_struct("TopSprite::Wall")
                .field("adjacency", adjacency)
                .field("east", east)
                .field("north", north)
                .field("overlap", overlap)
                .finish(),
            TopSprite::StairN {
                adjacency,
                north,
                overlap,
            } => f
                .debug_struct("TopSprite::StairN")
                .field("adjacency", adjacency)
                .field("north", north)
                .field("overlap", overlap)
                .finish(),
            TopSprite::StairE {
                adjacency,
                east,
                overlap,
            } => f
                .debug_struct("TopSprite::StairE")
                .field("adjacency", adjacency)
                .field("east", east)
                .field("overlap", overlap)
                .finish(),
            TopSprite::StairW { adjacency, overlap } => f
                .debug_struct("TopSprite::StairW")
                .field("adjacency", adjacency)
                .field("overlap", overlap)
                .finish(),
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

impl fmt::Debug for TopSpriteEast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopSpriteEast::None => write!(f, "TopSpriteEast::None"),
            TopSpriteEast::StairN => write!(f, "TopSpriteEast::StairN"),
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

impl fmt::Debug for TopSpriteNorth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopSpriteNorth::None => write!(f, "TopSpriteNorth::None"),
            TopSpriteNorth::Door => write!(f, "TopSpriteNorth::Door"),
        }
    }
}

impl fmt::Display for OverlapBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OverlapBase::None => write!(f, ""),
            OverlapBase::WallCorner => write!(f, "_OVERWALLCORNER"),
            OverlapBase::WallHorizontal => write!(f, "_OVERWALLHORIZONTAL"),
            OverlapBase::WallVertical => write!(f, "_OVERWALLVERTICAL"),
            OverlapBase::WallFull => write!(f, "_OVERWALLFULL"),
            OverlapBase::WallCornerEastDoor => write!(f, "_OVERWALLCORNEREASTDOOR"),
            OverlapBase::WallCornerEastStairN => write!(f, "_OVERWALLCORNEREASTSTAIRNORTH"),
            OverlapBase::WallCornerEastStairS => write!(f, "_OVERWALLCORNEREASTSTAIRSOUTH"),
            OverlapBase::WallVerticalEastDoor => write!(f, "_OVERWALLVERTICALEASTDOOR"),
            OverlapBase::WallVerticalEastStairN => write!(f, "_OVERWALLVERTICALEASTSTAIRNORTH"),
            OverlapBase::WallVerticalEastStairS => write!(f, "_OVERWALLVERTICALEASTSTAIRSOUTH"),
            OverlapBase::StairSouthCorner => write!(f, "_OVERSTAIRSOUTHCORNER"),
            OverlapBase::StairSouthHorizontal => write!(f, "_OVERSTAIRSOUTHHORIZONTAL"),
            OverlapBase::StairSouthVertical => write!(f, "_OVERSTAIRSOUTHVERTICAL"),
            OverlapBase::StairSouthFull => write!(f, "_OVERSTAIRSOUTHFULL"),
            OverlapBase::StairSouthInverseCorner => write!(f, "_OVERSTAIRSOUTHINVERSECORNER"),
            OverlapBase::StairEastVertical => write!(f, "_OVERSTAIREASTVERTICAL"),
            OverlapBase::StairEastInverseCorner => write!(f, "_OVERSTAIREASTINVERSECORNER"),
            OverlapBase::StairEastFull => write!(f, "_OVERSTAIREASTFULL"),
            OverlapBase::StairEastVerticalEastDoor => write!(f, "_OVERSTAIREASTVERTICALEASTDOOR"),
            OverlapBase::StairEastVerticalEastStairN => {
                write!(f, "_OVERSTAIREASTVERTICALEASTSTAIRNORTH")
            }
            OverlapBase::StairEastVerticalEastStairS => {
                write!(f, "_OVERSTAIREASTVERTICALEASTSTAIRSOUTH")
            }
        }
    }
}

impl fmt::Debug for OverlapBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OverlapBase::None => write!(f, "OverlapBase::None"),
            OverlapBase::WallCorner => write!(f, "OverlapBase::WallCorner"),
            OverlapBase::WallHorizontal => write!(f, "OverlapBase::WallHorizontal"),
            OverlapBase::WallVertical => write!(f, "OverlapBase::WallVertical"),
            OverlapBase::WallFull => write!(f, "OverlapBase::WallFull"),
            OverlapBase::WallCornerEastDoor => write!(f, "OverlapBase::WallCornerEastDoor"),
            OverlapBase::WallCornerEastStairN => write!(f, "OverlapBase::WallCornerEastStairN"),
            OverlapBase::WallCornerEastStairS => write!(f, "OverlapBase::WallCornerEastStairS"),
            OverlapBase::WallVerticalEastDoor => write!(f, "OverlapBase::WallVerticalEastDoor"),
            OverlapBase::WallVerticalEastStairN => {
                write!(f, "OverlapBase::WallVerticalEastStairN")
            }
            OverlapBase::WallVerticalEastStairS => {
                write!(f, "OverlapBase::WallVerticalEastStairS")
            }
            OverlapBase::StairSouthCorner => write!(f, "OverlapBase::StairSouthCorner"),
            OverlapBase::StairSouthHorizontal => write!(f, "OverlapBase::StairSouthHorizontal"),
            OverlapBase::StairSouthVertical => write!(f, "OverlapBase::StairSouthVertical"),
            OverlapBase::StairSouthFull => write!(f, "OverlapBase::StairSouthFull"),
            OverlapBase::StairSouthInverseCorner => {
                write!(f, "OverlapBase::StairSouthInverseCorner")
            }
            OverlapBase::StairEastVertical => write!(f, "OverlapBase::StairEastVertical"),
            OverlapBase::StairEastInverseCorner => {
                write!(f, "OverlapBase::StairEastInverseCorner")
            }
            OverlapBase::StairEastFull => write!(f, "OverlapBase::StairEastFull"),
            OverlapBase::StairEastVerticalEastDoor => {
                write!(f, "OverlapBase::StairEastVerticalEastDoor")
            }
            OverlapBase::StairEastVerticalEastStairN => {
                write!(f, "OverlapBase::StairEastVerticalEastStairN")
            }
            OverlapBase::StairEastVerticalEastStairS => {
                write!(f, "OverlapBase::StairEastVerticalEastStairS")
            }
        }
    }
}
