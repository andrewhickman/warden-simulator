use super::*;

impl TileVariant {
    pub fn flip(&self) -> Self {
        use TileVariant::*;

        match self {
            StairE => StairW,
            StairW => StairE,
            _ => *self,
        }
    }
}

impl MidSprite {
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
                east: EastDecoration::None,
                south: SouthDecoration::StairN,
            } => 3,
            Corner {
                east: EastDecoration::Door,
                south: SouthDecoration::None,
            } => 4,
            Corner {
                east: EastDecoration::Door,
                south: SouthDecoration::Door,
            } => 5,
            Corner {
                east: EastDecoration::Door,
                south: SouthDecoration::StairN,
            } => 6,
            Corner {
                east: EastDecoration::StairN,
                south: SouthDecoration::None,
            } => 7,
            Corner {
                east: EastDecoration::StairN,
                south: SouthDecoration::Door,
            } => 8,
            Corner {
                east: EastDecoration::StairN,
                south: SouthDecoration::StairN,
            } => 9,
            Corner {
                east: EastDecoration::StairS,
                south: SouthDecoration::None,
            } => 10,
            Corner {
                east: EastDecoration::StairS,
                south: SouthDecoration::Door,
            } => 11,
            Corner {
                east: EastDecoration::StairS,
                south: SouthDecoration::StairN,
            } => 12,
            Horizontal {
                south: SouthDecoration::None,
            } => 13,
            Horizontal {
                south: SouthDecoration::Door,
            } => 14,
            Horizontal {
                south: SouthDecoration::StairN,
            } => 15,
            Vertical {
                east: EastDecoration::None,
            } => 16,
            Vertical {
                east: EastDecoration::Door,
            } => 17,
            Vertical {
                east: EastDecoration::StairN,
            } => 18,
            Vertical {
                east: EastDecoration::StairS,
            } => 19,
            InverseCorner => 20,
            Full => 21,
            StairN => 22,
            StairNFull => 23,
            StairS => 24,
            StairSFull => 25,
            StairE => 26,
            StairEFull => 27,
            StairW => 28,
            StairWFull => 29,
        }
    }
}

impl TopSprite {
    pub fn flip(&self) -> Self {
        use TopSprite::*;

        match self {
            StairE => StairW,
            StairW => StairE,
            _ => *self,
        }
    }

    pub fn id(&self) -> u16 {
        use TopSprite::*;

        match self {
            Empty {
                east: EastTopDecoration::None,
            } => 0,
            Empty {
                east: EastTopDecoration::EastStairW,
            } => 30,
            Empty {
                east: EastTopDecoration::EastStairWFull,
            } => 31,
            Empty {
                east: EastTopDecoration::SouthEastStairN,
            } => 32,
            Empty {
                east: EastTopDecoration::SouthEastStairW,
            } => 33,
            Empty {
                east: EastTopDecoration::SouthEastStairWFull,
            } => 34,
            Corner {
                east: EastTopDecoration::None,
                deco: TopDecoration::None,
            } => 35,
            Corner {
                east: EastTopDecoration::None,
                deco: TopDecoration::Door,
            } => 36,
            Corner {
                east: EastTopDecoration::EastStairW,
                deco: TopDecoration::None,
            } => 37,
            Corner {
                east: EastTopDecoration::EastStairW,
                deco: TopDecoration::Door,
            } => 38,
            Corner {
                east: EastTopDecoration::EastStairWFull,
                deco: TopDecoration::None,
            } => 39,
            Corner {
                east: EastTopDecoration::EastStairWFull,
                deco: TopDecoration::Door,
            } => 40,
            Corner {
                east: EastTopDecoration::SouthEastStairN,
                deco: TopDecoration::None,
            } => 41,
            Corner {
                east: EastTopDecoration::SouthEastStairN,
                deco: TopDecoration::Door,
            } => 42,
            Corner {
                east: EastTopDecoration::SouthEastStairW,
                deco: TopDecoration::None,
            } => 43,
            Corner {
                east: EastTopDecoration::SouthEastStairW,
                deco: TopDecoration::Door,
            } => 44,
            Corner {
                east: EastTopDecoration::SouthEastStairWFull,
                deco: TopDecoration::None,
            } => 45,
            Corner {
                east: EastTopDecoration::SouthEastStairWFull,
                deco: TopDecoration::Door,
            } => 46,
            Horizontal {
                deco: TopDecoration::None,
            } => 47,
            Horizontal {
                deco: TopDecoration::Door,
            } => 48,
            StairN => 49,
            StairNFull => 50,
            StairE => 51,
            StairEFull => 52,
            StairW => 53,
            StairWFull => 54,
        }
    }
}
