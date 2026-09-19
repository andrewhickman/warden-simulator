use std::fmt;

use crate::{BaseSpriteCenter::WallInverseCorner, Model::DoorVerticalNorth};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TileVariant {
    Empty,
    Wall,
    Door,
    StairN,
    StairS,
    StairE,
    StairW,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Model {
    Empty,
    Wall { north: WallModel, south: WallModel },
    DoorHorizontal,
    DoorHorizontalFull,
    DoorVertical,
    DoorVerticalNorth,
    DoorVerticalSouth,
    DoorVerticalNorthSouth,
    StairN,
    StairNFull,
    StairS,
    StairSFull,
    StairE,
    StairENorth,
    StairESouth,
    StairENorthSouth,
    StairW,
    StairWNorth,
    StairWSouth,
    StairWNorthSouth,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum WallModel {
    Corner,
    Vertical,
    Horizontal,
    InverseCorner,
    Full,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct BaseSprite {
    pub center: BaseSpriteCenter,
    pub south: BaseSpriteSouth,
    pub east: BaseSpriteEast,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum BaseSpriteCenter {
    None,
    WallCorner,
    WallVertical,
    WallHorizontal,
    WallInverseCorner,
    WallFull,
    StairN,
    StairNFull,
    StairS,
    StairSFull,
    StairE,
    StairEFull,
    StairW,
    StairWFull,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum BaseSpriteSouth {
    None,
    Door,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum BaseSpriteEast {
    None,
    Door,
    StairN,
    StairS,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct TopSprite {
    pub center: TopSpriteCenter,
    pub south: TopSpriteSouth,
    pub south_east: TopSpriteSouthEast,
    pub east: TopSpriteEast,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum TopSpriteCenter {
    None,
    WallCorner,
    WallVertical,
    WallHorizontal,
    WallInverseCorner,
    WallFull,
    Door,
    StairS,
    StairSFull,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum TopSpriteSouth {
    None,
    Door,
    WallCorner,
    WallHorizontal,
    StairN,
    StairNFull,
    StairE,
    StairEFull,
    StairW,
    StairWFull,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum TopSpriteEast {
    None,
    Door,
    StairS,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum TopSpriteSouthEast {
    None,
    StairN,
}

impl TileVariant {
    pub fn values() -> [TileVariant; 7] {
        [
            TileVariant::Empty,
            TileVariant::Wall,
            TileVariant::Door,
            TileVariant::StairN,
            TileVariant::StairS,
            TileVariant::StairE,
            TileVariant::StairW,
        ]
    }

    pub fn flip(&self) -> Self {
        use TileVariant::*;

        match self {
            StairE => StairW,
            StairW => StairE,
            _ => *self,
        }
    }
}

impl Model {
    pub fn resolve(
        center: TileVariant,
        north: TileVariant,
        north_east: TileVariant,
        east: TileVariant,
        south_east: TileVariant,
        south: TileVariant,
    ) -> Model {
        match center {
            TileVariant::Empty => Model::Empty,
            TileVariant::Wall => Model::Wall {
                north: WallModel::resolve(
                    matches!(north, TileVariant::Wall | TileVariant::StairS),
                    matches!(east, TileVariant::Wall | TileVariant::StairW),
                    matches!(
                        north_east,
                        TileVariant::Wall | TileVariant::StairW | TileVariant::StairS
                    ),
                ),
                south: WallModel::resolve(
                    matches!(south, TileVariant::Wall | TileVariant::StairN),
                    matches!(east, TileVariant::Wall | TileVariant::StairW),
                    matches!(
                        south_east,
                        TileVariant::Wall | TileVariant::StairW | TileVariant::StairN
                    ),
                ),
            },
            TileVariant::Door => {
                if east == TileVariant::Wall {
                    Model::DoorHorizontalFull
                } else {
                    match (north, south) {
                        (TileVariant::Wall, TileVariant::Wall) => Model::DoorVerticalNorthSouth,
                        (TileVariant::Wall, _) => Model::DoorVerticalNorth,
                        (_, TileVariant::Wall) => Model::DoorVerticalSouth,
                        (_, _) => Model::DoorVertical,
                    }
                }
            }
            TileVariant::StairN => match east {
                TileVariant::Wall | TileVariant::StairN => Model::StairNFull,
                _ => Model::StairN,
            },
            TileVariant::StairS => match east {
                TileVariant::Wall | TileVariant::StairS => Model::StairSFull,
                _ => Model::StairS,
            },
            TileVariant::StairE => match (north, south) {
                (
                    TileVariant::Wall | TileVariant::StairE,
                    TileVariant::Wall | TileVariant::StairE,
                ) => Model::StairENorthSouth,
                (TileVariant::Wall | TileVariant::StairE, _) => Model::StairENorth,
                (_, TileVariant::Wall | TileVariant::StairE) => Model::StairESouth,
                (_, _) => Model::StairE,
            },
            TileVariant::StairW => match (north, south) {
                (
                    TileVariant::Wall | TileVariant::StairW,
                    TileVariant::Wall | TileVariant::StairW,
                ) => Model::StairWNorthSouth,
                (TileVariant::Wall | TileVariant::StairW, _) => Model::StairWNorth,
                (_, TileVariant::Wall | TileVariant::StairW) => Model::StairWSouth,
                (_, _) => Model::StairW,
            },
        }
    }

    pub fn center_base_sprite(&self) -> BaseSpriteCenter {
        match self {
            Model::Empty
            | Model::DoorHorizontal
            | Model::DoorHorizontalFull
            | Model::DoorVertical
            | Model::DoorVerticalNorth
            | Model::DoorVerticalSouth
            | Model::DoorVerticalNorthSouth => BaseSpriteCenter::None,
            Model::Wall { north: _, south } => match south {
                WallModel::Corner => BaseSpriteCenter::WallCorner,
                WallModel::Vertical => BaseSpriteCenter::WallVertical,
                WallModel::Horizontal => BaseSpriteCenter::WallHorizontal,
                WallModel::InverseCorner => BaseSpriteCenter::WallInverseCorner,
                WallModel::Full => BaseSpriteCenter::WallFull,
            },
            Model::StairN => BaseSpriteCenter::StairN,
            Model::StairNFull => BaseSpriteCenter::StairNFull,
            Model::StairS => BaseSpriteCenter::StairS,
            Model::StairSFull => BaseSpriteCenter::StairSFull,
            Model::StairE | Model::StairENorth => BaseSpriteCenter::StairE,
            Model::StairESouth | Model::StairENorthSouth => BaseSpriteCenter::StairEFull,
            Model::StairW | Model::StairWNorth => BaseSpriteCenter::StairW,
            Model::StairWSouth | Model::StairWNorthSouth => BaseSpriteCenter::StairWFull,
        }
    }

    pub fn center_top_sprite(&self) -> TopSpriteCenter {
        match self {
            Model::Empty => TopSpriteCenter::None,
            Model::Wall { north: _, south } => match south {
                WallModel::Corner => TopSpriteCenter::WallCorner,
                WallModel::Vertical => TopSpriteCenter::WallVertical,
                WallModel::Horizontal => TopSpriteCenter::WallHorizontal,
                WallModel::InverseCorner => TopSpriteCenter::WallInverseCorner,
                WallModel::Full => TopSpriteCenter::WallFull,
            },
            Model::DoorHorizontal
            | Model::DoorHorizontalFull
            | Model::DoorVertical
            | DoorVerticalNorth => TopSpriteCenter::None,
            Model::DoorVerticalSouth | Model::DoorVerticalNorthSouth => TopSpriteCenter::Door,
            Model::StairN
            | Model::StairNFull
            | Model::StairE
            | Model::StairENorth
            | Model::StairESouth
            | Model::StairENorthSouth
            | Model::StairW
            | Model::StairWNorth
            | Model::StairWSouth
            | Model::StairWNorthSouth => TopSpriteCenter::None,
            Model::StairS => TopSpriteCenter::StairS,
            Model::StairSFull => TopSpriteCenter::StairSFull,
        }
    }

    pub fn north_base_sprite(&self) -> BaseSpriteSouth {
        match self {
            Model::DoorVerticalNorth | Model::DoorVerticalNorthSouth => BaseSpriteSouth::Door,
            _ => BaseSpriteSouth::None,
        }
    }

    pub fn north_top_sprite(&self) -> TopSpriteSouth {
        match self {
            Model::Empty => TopSpriteSouth::None,
            Model::Wall {
                north: WallModel::Corner | WallModel::Vertical,
                south: _,
            } => TopSpriteSouth::WallCorner,
            Model::Wall {
                north: WallModel::Horizontal | WallModel::InverseCorner | WallModel::Full,
                south: _,
            } => TopSpriteSouth::WallHorizontal,
            Model::DoorHorizontal
            | Model::DoorHorizontalFull
            | Model::DoorVertical
            | Model::DoorVerticalSouth => TopSpriteSouth::None,
            Model::DoorVerticalNorth | Model::DoorVerticalNorthSouth => TopSpriteSouth::Door,
            Model::StairN => TopSpriteSouth::StairN,
            Model::StairNFull => TopSpriteSouth::StairNFull,
            Model::StairS | Model::StairSFull => TopSpriteSouth::None,
            Model::StairE | Model::StairESouth => TopSpriteSouth::StairE,
            Model::StairENorth | Model::StairENorthSouth => TopSpriteSouth::StairEFull,
            Model::StairW | Model::StairWSouth => TopSpriteSouth::StairW,
            Model::StairWNorth | Model::StairWNorthSouth => TopSpriteSouth::StairWFull,
        }
    }

    pub fn east_base_sprite(&self) -> BaseSpriteEast {
        match self {
            Model::Empty => BaseSpriteEast::None,
            Model::Wall { .. } => BaseSpriteEast::None,
            Model::DoorHorizontal
            | Model::DoorVertical
            | Model::DoorVerticalNorth
            | Model::DoorVerticalSouth
            | Model::DoorVerticalNorthSouth => BaseSpriteEast::None,
            Model::DoorHorizontalFull => BaseSpriteEast::Door,
            Model::StairN | Model::StairS => BaseSpriteEast::None,
            Model::StairNFull => BaseSpriteEast::StairN,
            Model::StairSFull => BaseSpriteEast::StairS,
            Model::StairE
            | Model::StairENorth
            | Model::StairESouth
            | Model::StairENorthSouth
            | Model::StairW
            | Model::StairWNorth
            | Model::StairWSouth
            | Model::StairWNorthSouth => BaseSpriteEast::None,
        }
    }

    pub fn east_top_sprite(&self) -> TopSpriteEast {
        match self {
            Model::Empty => TopSpriteEast::None,
            Model::Wall { .. } => TopSpriteEast::None,
            Model::DoorHorizontal
            | Model::DoorVertical
            | Model::DoorVerticalNorth
            | Model::DoorVerticalSouth
            | Model::DoorVerticalNorthSouth => TopSpriteEast::None,
            Model::DoorHorizontalFull => TopSpriteEast::Door,
            Model::StairN
            | Model::StairS
            | Model::StairNFull
            | Model::StairE
            | Model::StairENorth
            | Model::StairESouth
            | Model::StairENorthSouth
            | Model::StairW
            | Model::StairWNorth
            | Model::StairWSouth
            | Model::StairWNorthSouth => TopSpriteEast::None,
            Model::StairSFull => TopSpriteEast::StairS,
        }
    }

    pub fn north_east_top_sprite(&self) -> TopSpriteSouthEast {
        match self {
            Model::Empty => TopSpriteSouthEast::None,
            Model::Wall { .. } => TopSpriteSouthEast::None,
            Model::DoorHorizontal
            | Model::DoorVertical
            | Model::DoorVerticalNorth
            | Model::DoorVerticalSouth
            | Model::DoorVerticalNorthSouth
            | Model::DoorHorizontalFull => TopSpriteSouthEast::None,
            Model::StairN | Model::StairS | Model::StairSFull => TopSpriteSouthEast::None,
            Model::StairNFull => TopSpriteSouthEast::StairN,
            Model::StairE | Model::StairENorth | Model::StairESouth | Model::StairENorthSouth => {
                TopSpriteSouthEast::None
            }
            Model::StairW | Model::StairWSouth | Model::StairWNorth | Model::StairWNorthSouth => {
                TopSpriteSouthEast::None
            }
        }
    }

    pub fn flip(&self) -> Model {
        match self {
            Model::StairE => Model::StairW,
            Model::StairENorth => Model::StairWNorth,
            Model::StairESouth => Model::StairWSouth,
            Model::StairENorthSouth => Model::StairWNorthSouth,
            Model::StairW => Model::StairE,
            Model::StairWNorth => Model::StairENorth,
            Model::StairWSouth => Model::StairESouth,
            Model::StairWNorthSouth => Model::StairENorthSouth,
            _ => *self,
        }
    }
}

impl WallModel {
    fn resolve(south: bool, east: bool, south_east: bool) -> WallModel {
        match (south, east, south_east) {
            (false, false, _) => WallModel::Corner,
            (false, true, _) => WallModel::Horizontal,
            (true, false, _) => WallModel::Vertical,
            (true, true, false) => WallModel::InverseCorner,
            (true, true, true) => WallModel::Full,
        }
    }
}

pub fn resolve_sprites(
    center: TileVariant,
    north: TileVariant,
    north_east: TileVariant,
    east: TileVariant,
    south_east: TileVariant,
    south: TileVariant,
) -> (BaseSprite, TopSprite) {
    let center_model = Model::resolve(center, north, north_east, east, south_east, south);

    let east_model = Model::resolve(
        east.flip(),
        north_east.flip(),
        north.flip(),
        center.flip(),
        south.flip(),
        south_east.flip(),
    )
    .flip();
    let south_east_model = Model::resolve(
        south_east.flip(),
        east.flip(),
        center.flip(),
        south.flip(),
        TileVariant::Empty,
        TileVariant::Empty,
    )
    .flip();
    let south_model = Model::resolve(
        south,
        center,
        east,
        south_east,
        TileVariant::Empty,
        TileVariant::Empty,
    );

    (
        BaseSprite {
            center: center_model.center_base_sprite(),
            south: south_model.north_base_sprite(),
            east: east_model.east_base_sprite(),
        }
        .normalize(),
        TopSprite {
            center: center_model.center_top_sprite(),
            south: south_model.north_top_sprite(),
            south_east: south_east_model.north_east_top_sprite(),
            east: east_model.east_top_sprite(),
        }
        .normalize(),
    )
}

impl fmt::Debug for BaseSprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{:?}{:?}", self.center, self.east, self.south)
    }
}

impl fmt::Debug for BaseSpriteCenter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseSpriteCenter::None => write!(f, "Empty"),
            BaseSpriteCenter::WallCorner => write!(f, "WallCorner"),
            BaseSpriteCenter::WallVertical => write!(f, "WallVertical"),
            BaseSpriteCenter::WallHorizontal => write!(f, "WallHorizontal"),
            BaseSpriteCenter::WallInverseCorner => write!(f, "WallInverseCorner"),
            BaseSpriteCenter::WallFull => write!(f, "WallFull"),
            BaseSpriteCenter::StairN => write!(f, "StairN"),
            BaseSpriteCenter::StairNFull => write!(f, "StairNFull"),
            BaseSpriteCenter::StairS => write!(f, "StairS"),
            BaseSpriteCenter::StairSFull => write!(f, "StairSFull"),
            BaseSpriteCenter::StairE => write!(f, "StairE"),
            BaseSpriteCenter::StairEFull => write!(f, "StairEFull"),
            BaseSpriteCenter::StairW => write!(f, "StairW"),
            BaseSpriteCenter::StairWFull => write!(f, "StairWFull"),
        }
    }
}

impl fmt::Debug for BaseSpriteEast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Door => write!(f, "_EastDoor"),
            Self::StairN => write!(f, "_EastStairN"),
            Self::StairS => write!(f, "_EastStairS"),
        }
    }
}

impl fmt::Debug for BaseSpriteSouth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Door => write!(f, "_SouthDoor"),
        }
    }
}

impl fmt::Debug for TopSprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}{:?}{:?}{:?}",
            self.center, self.south, self.south_east, self.east
        )
    }
}

impl fmt::Debug for TopSpriteCenter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "Empty"),
            Self::WallCorner => write!(f, "WallCorner"),
            Self::WallVertical => write!(f, "WallVertical"),
            Self::WallHorizontal => write!(f, "WallHorizontal"),
            Self::WallInverseCorner => write!(f, "WallInverseCorner"),
            Self::WallFull => write!(f, "WallFull"),
            Self::Door => write!(f, "Door"),
            Self::StairS => write!(f, "StairS"),
            Self::StairSFull => write!(f, "StairSFull"),
        }
    }
}

impl fmt::Debug for TopSpriteSouth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Door => write!(f, "_SouthDoor"),
            Self::WallCorner => write!(f, "_SouthWallCorner"),
            Self::WallHorizontal => write!(f, "_SouthWallHorizontal"),
            Self::StairN => write!(f, "_SouthStairN"),
            Self::StairNFull => write!(f, "_SouthStairNFull"),
            Self::StairE => write!(f, "_SouthStairE"),
            Self::StairEFull => write!(f, "_SouthStairEFull"),
            Self::StairW => write!(f, "_SouthStairW"),
            Self::StairWFull => write!(f, "_SouthStairWFull"),
        }
    }
}

impl fmt::Debug for TopSpriteSouthEast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::StairN => write!(f, "_SouthEastStairN"),
        }
    }
}

impl fmt::Debug for TopSpriteEast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Door => write!(f, "_EastDoor"),
            Self::StairS => write!(f, "_EastStairS"),
        }
    }
}

impl BaseSprite {
    /// The sprite with no walls, doors, or stairs.
    pub const EMPTY: Self = Self {
        center: BaseSpriteCenter::None,
        south: BaseSpriteSouth::None,
        east: BaseSpriteEast::None,
    };

    pub fn resolve(
        center: TileVariant,
        north: TileVariant,
        north_east: TileVariant,
        east: TileVariant,
        south_east: TileVariant,
        south: TileVariant,
    ) -> Self {
        resolve_sprites(center, north, north_east, east, south_east, south).0
    }

    /// Maps a sprite to its canonical, visually-deduplicated form.
    pub fn normalize(self) -> Self {
        match self {
            // The full stair-top already spans the tile, covering where the matching east stair edge would be.
            Self {
                center: BaseSpriteCenter::StairNFull,
                east: BaseSpriteEast::StairN,
                ..
            }
            | Self {
                center: BaseSpriteCenter::StairSFull,
                east: BaseSpriteEast::StairS,
                ..
            } => Self {
                east: BaseSpriteEast::None,
                ..self
            },
            _ => self,
        }
    }
}

impl TopSprite {
    /// The sprite with no walls, doors, or stairs.
    pub const EMPTY: Self = Self {
        center: TopSpriteCenter::None,
        south: TopSpriteSouth::None,
        south_east: TopSpriteSouthEast::None,
        east: TopSpriteEast::None,
    };

    pub fn resolve(
        center: TileVariant,
        north: TileVariant,
        north_east: TileVariant,
        east: TileVariant,
        south_east: TileVariant,
        south: TileVariant,
    ) -> Self {
        resolve_sprites(center, north, north_east, east, south_east, south).1
    }

    /// Maps a sprite to its canonical, visually-deduplicated form.
    pub fn normalize(self) -> Self {
        // These two reductions key off disjoint fields and can both apply to the same sprite,
        // so both are applied here in sequence to keep `normalize()` a fixed point in one call.
        let this = match self {
            // A full south-facing stair top already spans the tile, covering the north stair
            // silhouette that would otherwise share its south-east corner.
            Self {
                south: TopSpriteSouth::StairNFull,
                ..
            } => Self {
                south_east: TopSpriteSouthEast::None,
                ..self
            },
            _ => self,
        };

        let this = match this {
            // The full south-facing stair top already spans the tile, covering where the
            // matching east stair edge would be.
            Self {
                center: TopSpriteCenter::StairSFull,
                east: TopSpriteEast::StairS,
                ..
            } => Self {
                east: TopSpriteEast::None,
                ..this
            },
            _ => this,
        };

        let this = match this {
            Self {
                center: TopSpriteCenter::WallCorner | TopSpriteCenter::WallHorizontal,
                east: TopSpriteEast::None | TopSpriteEast::Door | TopSpriteEast::StairS,
                south: TopSpriteSouth::None | TopSpriteSouth::Door,
                south_east: TopSpriteSouthEast::None,
            } => Self::EMPTY,
            Self {
                center: TopSpriteCenter::WallVertical | TopSpriteCenter::WallInverseCorner,
                east: TopSpriteEast::None | TopSpriteEast::Door | TopSpriteEast::StairS,
                south: TopSpriteSouth::WallCorner,
                south_east: TopSpriteSouthEast::None,
            } => Self::EMPTY,
            Self {
                center: TopSpriteCenter::WallCorner,
                east: TopSpriteEast::None | TopSpriteEast::Door,
                south: TopSpriteSouth::StairEFull | TopSpriteSouth::StairWFull,
                south_east: TopSpriteSouthEast::None,
            } => Self {
                south: this.south,
                ..Self::EMPTY
            },
            Self {
                center: TopSpriteCenter::WallCorner,
                east: TopSpriteEast::StairS,
                south: TopSpriteSouth::StairEFull,
                south_east: TopSpriteSouthEast::None,
            } => Self {
                south: TopSpriteSouth::StairEFull,
                ..Self::EMPTY
            },
            Self {
                center: TopSpriteCenter::WallHorizontal | TopSpriteCenter::WallCorner,
                east: TopSpriteEast::None | TopSpriteEast::Door | TopSpriteEast::StairS,
                south: TopSpriteSouth::StairWFull,
                south_east: TopSpriteSouthEast::None,
            } => Self {
                south: this.south,
                ..Self::EMPTY
            },
            Self {
                center: TopSpriteCenter::WallFull,
                east: TopSpriteEast::None | TopSpriteEast::Door,
                south: TopSpriteSouth::WallHorizontal,
                south_east: TopSpriteSouthEast::None,
            } => Self::EMPTY,
            Self {
                center: TopSpriteCenter::StairS | TopSpriteCenter::StairSFull,
                east: TopSpriteEast::None,
                south: TopSpriteSouth::None | TopSpriteSouth::StairE | TopSpriteSouth::StairW,
                south_east: TopSpriteSouthEast::None,
            } => Self {
                south: this.south,
                ..Self::EMPTY
            },
            _ => this,
        };

        this
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_CENTERS: [BaseSpriteCenter; 14] = [
        BaseSpriteCenter::None,
        BaseSpriteCenter::WallCorner,
        BaseSpriteCenter::WallVertical,
        BaseSpriteCenter::WallHorizontal,
        BaseSpriteCenter::WallInverseCorner,
        BaseSpriteCenter::WallFull,
        BaseSpriteCenter::StairN,
        BaseSpriteCenter::StairNFull,
        BaseSpriteCenter::StairS,
        BaseSpriteCenter::StairSFull,
        BaseSpriteCenter::StairE,
        BaseSpriteCenter::StairEFull,
        BaseSpriteCenter::StairW,
        BaseSpriteCenter::StairWFull,
    ];
    const BASE_SOUTHS: [BaseSpriteSouth; 2] = [BaseSpriteSouth::None, BaseSpriteSouth::Door];
    const BASE_EASTS: [BaseSpriteEast; 4] = [
        BaseSpriteEast::None,
        BaseSpriteEast::Door,
        BaseSpriteEast::StairN,
        BaseSpriteEast::StairS,
    ];

    const TOP_CENTERS: [TopSpriteCenter; 9] = [
        TopSpriteCenter::None,
        TopSpriteCenter::WallCorner,
        TopSpriteCenter::WallVertical,
        TopSpriteCenter::WallHorizontal,
        TopSpriteCenter::WallInverseCorner,
        TopSpriteCenter::WallFull,
        TopSpriteCenter::Door,
        TopSpriteCenter::StairS,
        TopSpriteCenter::StairSFull,
    ];
    const TOP_SOUTHS: [TopSpriteSouth; 10] = [
        TopSpriteSouth::None,
        TopSpriteSouth::Door,
        TopSpriteSouth::WallCorner,
        TopSpriteSouth::WallHorizontal,
        TopSpriteSouth::StairN,
        TopSpriteSouth::StairNFull,
        TopSpriteSouth::StairE,
        TopSpriteSouth::StairEFull,
        TopSpriteSouth::StairW,
        TopSpriteSouth::StairWFull,
    ];
    const TOP_SOUTH_EASTS: [TopSpriteSouthEast; 2] =
        [TopSpriteSouthEast::None, TopSpriteSouthEast::StairN];
    const TOP_EASTS: [TopSpriteEast; 3] = [
        TopSpriteEast::None,
        TopSpriteEast::Door,
        TopSpriteEast::StairS,
    ];

    /// `normalize()` must reach its fixed point in a single match, i.e. re-normalizing an
    /// already-normalized sprite must be a no-op, for every possible sprite.
    #[test]
    fn base_sprite_normalize_is_a_fixed_point_in_one_step() {
        for &center in &BASE_CENTERS {
            for &south in &BASE_SOUTHS {
                for &east in &BASE_EASTS {
                    let sprite = BaseSprite {
                        center,
                        south,
                        east,
                    };
                    let normalized = sprite.normalize();
                    assert_eq!(
                        normalized.normalize(),
                        normalized,
                        "normalize() is not a fixed point for {sprite:?} -> {normalized:?}"
                    );
                }
            }
        }
    }

    /// `normalize()` must reach its fixed point in a single match, i.e. re-normalizing an
    /// already-normalized sprite must be a no-op, for every possible sprite.
    #[test]
    fn top_sprite_normalize_is_a_fixed_point_in_one_step() {
        for &center in &TOP_CENTERS {
            for &south in &TOP_SOUTHS {
                for &south_east in &TOP_SOUTH_EASTS {
                    for &east in &TOP_EASTS {
                        let sprite = TopSprite {
                            center,
                            south,
                            south_east,
                            east,
                        };
                        let normalized = sprite.normalize();
                        assert_eq!(
                            normalized.normalize(),
                            normalized,
                            "normalize() is not a fixed point for {sprite:?} -> {normalized:?}"
                        );
                    }
                }
            }
        }
    }
}
