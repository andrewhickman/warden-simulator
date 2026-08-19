use super::*;

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
            Corner {
                east: EastDecoration::StairS,
                south: SouthDecoration::None,
            } => 7,
            Corner {
                east: EastDecoration::StairS,
                south: SouthDecoration::Door,
            } => 8,
            Horizontal {
                south: SouthDecoration::None,
            } => 9,
            Horizontal {
                south: SouthDecoration::Door,
            } => 10,
            Vertical {
                east: EastDecoration::None,
            } => 11,
            Vertical {
                east: EastDecoration::Door,
            } => 12,
            Vertical {
                east: EastDecoration::StairN,
            } => 13,
            Vertical {
                east: EastDecoration::StairS,
            } => 14,
            InverseCorner => 15,
            Full => 16,
            StairN => 17,
            StairNFull => 18,
            StairS => 19,
            StairSFull => 20,
        }
    }
}

impl TopSprite {
    pub fn id(&self) -> u16 {
        use TopSprite::*;

        match self {
            Empty {
                south_east: SouthEastTopDecoration::None,
            } => 0,
            Empty {
                south_east: SouthEastTopDecoration::StairN,
            } => 21,
            Corner {
                south_east: SouthEastTopDecoration::None,
                deco: TopDecoration::None,
            } => 22,
            Corner {
                south_east: SouthEastTopDecoration::None,
                deco: TopDecoration::Door,
            } => 23,
            Corner {
                south_east: SouthEastTopDecoration::StairN,
                deco: TopDecoration::None,
            } => 24,
            Corner {
                south_east: SouthEastTopDecoration::StairN,
                deco: TopDecoration::Door,
            } => 25,
            Horizontal {
                deco: TopDecoration::None,
            } => 26,
            Horizontal {
                deco: TopDecoration::Door,
            } => 27,
            StairN => 28,
            StairNFull => 29,
        }
    }
}
