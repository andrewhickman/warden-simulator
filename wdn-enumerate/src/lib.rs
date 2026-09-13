use crate::Model::DoorVerticalNorth;

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

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct BaseSprite {
    center: BaseSpriteCenter,
    south: BaseSpriteSouth,
    east: BaseSpriteEast,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
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

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum BaseSpriteSouth {
    None,
    Door,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum BaseSpriteEast {
    None,
    Door,
    StairN,
    StairS,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct TopSprite {
    center: TopSpriteCenter,
    south: TopSpriteSouth,
    south_east: TopSpriteSouthEast,
    east: TopSpriteEast,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TopSpriteCenter {
    None,
    WallCorner,
    WallVertical,
    WallHorizontal,
    WallInverseCorner,
    WallFull,
    Door,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
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

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TopSpriteEast {
    None,
    Door,
    StairW,
    StairWFull,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TopSpriteSouthEast {
    None,
    StairN,
    StairW,
    StairWFull,
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
                    matches!(north_east, TileVariant::Wall),
                ),
                south: WallModel::resolve(
                    matches!(south, TileVariant::Wall | TileVariant::StairN),
                    matches!(east, TileVariant::Wall | TileVariant::StairW),
                    matches!(south_east, TileVariant::Wall),
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
            Model::StairW | Model::StairWSouth => BaseSpriteCenter::StairW,
            Model::StairWNorth | Model::StairWNorthSouth => BaseSpriteCenter::StairWFull,
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
            | Model::StairS
            | Model::StairSFull
            | Model::StairE
            | Model::StairENorth
            | Model::StairESouth
            | Model::StairENorthSouth
            | Model::StairW
            | Model::StairWNorth
            | Model::StairWSouth
            | Model::StairWNorthSouth => TopSpriteCenter::None,
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
            Model::StairW | Model::StairWSouth => TopSpriteSouth::StairWFull,
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
            Model::StairN | Model::StairS => TopSpriteEast::None,
            Model::StairNFull
            | Model::StairSFull
            | Model::StairE
            | Model::StairENorth
            | Model::StairESouth
            | Model::StairENorthSouth => TopSpriteEast::None,
            Model::StairW | Model::StairWNorth => TopSpriteEast::StairW,
            Model::StairWSouth | Model::StairWNorthSouth => TopSpriteEast::StairWFull,
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
            Model::StairW | Model::StairWSouth => TopSpriteSouthEast::StairW,
            Model::StairWNorth | Model::StairWNorthSouth => TopSpriteSouthEast::StairWFull,
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
        },
        TopSprite {
            center: center_model.center_top_sprite(),
            south: south_model.north_top_sprite(),
            south_east: south_east_model.north_east_top_sprite(),
            east: east_model.east_top_sprite(),
        },
    )
}
