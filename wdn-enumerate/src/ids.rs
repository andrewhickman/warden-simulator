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
                south_east: SouthEastTopDecoration::None,
            } => 0,
            Empty {
                south_east: SouthEastTopDecoration::StairN,
            } => 30,
            Empty {
                south_east: SouthEastTopDecoration::StairW,
            } => 31,
            Empty {
                south_east: SouthEastTopDecoration::StairWFull,
            } => 32,
            Corner {
                south_east: SouthEastTopDecoration::None,
                deco: TopDecoration::None,
            } => 33,
            Corner {
                south_east: SouthEastTopDecoration::None,
                deco: TopDecoration::Door,
            } => 34,
            Corner {
                south_east: SouthEastTopDecoration::StairN,
                deco: TopDecoration::None,
            } => 35,
            Corner {
                south_east: SouthEastTopDecoration::StairN,
                deco: TopDecoration::Door,
            } => 36,
            Corner {
                south_east: SouthEastTopDecoration::StairW,
                deco: TopDecoration::None,
            } => 37,
            Corner {
                south_east: SouthEastTopDecoration::StairW,
                deco: TopDecoration::Door,
            } => 38,
            Corner {
                south_east: SouthEastTopDecoration::StairWFull,
                deco: TopDecoration::None,
            } => 39,
            Corner {
                south_east: SouthEastTopDecoration::StairWFull,
                deco: TopDecoration::Door,
            } => 40,
            Horizontal {
                deco: TopDecoration::None,
            } => 41,
            Horizontal {
                deco: TopDecoration::Door,
            } => 42,
            StairN => 43,
            StairNFull => 44,
            StairE => 45,
            StairEFull => 46,
            StairW => 47,
            StairWFull => 48,
        }
    }
}
