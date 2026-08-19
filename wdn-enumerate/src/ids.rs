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
    pub fn flip(&self) -> Self {
        use MidSprite::*;

        match self {
            StairE => StairW,
            StairEFull => StairWFull,
            StairW => StairE,
            StairWFull => StairEFull,
            _ => *self,
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
            Corner {
                east: EastDecoration::StairW,
                south: SouthDecoration::None,
            } => 13,
            Corner {
                east: EastDecoration::StairW,
                south: SouthDecoration::Door,
            } => 14,
            Corner {
                east: EastDecoration::StairW,
                south: SouthDecoration::StairN,
            } => 15,
            Horizontal {
                south: SouthDecoration::None,
            } => 16,
            Horizontal {
                south: SouthDecoration::Door,
            } => 17,
            Horizontal {
                south: SouthDecoration::StairN,
            } => 18,
            Vertical {
                east: EastDecoration::None,
            } => 19,
            Vertical {
                east: EastDecoration::Door,
            } => 20,
            Vertical {
                east: EastDecoration::StairN,
            } => 21,
            Vertical {
                east: EastDecoration::StairS,
            } => 22,
            Vertical {
                east: EastDecoration::StairW,
            } => 23,
            InverseCorner => 24,
            Full => 25,
            StairN => 26,
            StairNFull => 27,
            StairS => 28,
            StairSFull => 29,
            StairE => 30,
            StairEFull => 31,
            StairW => 32,
            StairWFull => 33,
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
            } => 34,
            Corner {
                south_east: SouthEastTopDecoration::None,
                deco: TopDecoration::None,
            } => 35,
            Corner {
                south_east: SouthEastTopDecoration::None,
                deco: TopDecoration::Door,
            } => 36,
            Corner {
                south_east: SouthEastTopDecoration::StairN,
                deco: TopDecoration::None,
            } => 37,
            Corner {
                south_east: SouthEastTopDecoration::StairN,
                deco: TopDecoration::Door,
            } => 38,
            Horizontal {
                deco: TopDecoration::None,
            } => 39,
            Horizontal {
                deco: TopDecoration::Door,
            } => 40,
            StairN => 41,
            StairNFull => 42,
            StairE => 43,
            StairW => 44,
        }
    }
}
