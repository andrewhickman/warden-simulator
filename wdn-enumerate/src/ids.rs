use super::*;

impl BaseSprite {
    pub fn id(&self) -> u16 {
        match self {
            BaseSprite::Empty => 0,
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            } => 1,
            BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            } => 2,
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::Door,
            } => 3,
            BaseSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            } => 4,
            BaseSprite::Wall {
                adjacency: Adjacency::InverseCorner,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            } => 5,
            BaseSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::Door,
            } => 6,
            BaseSprite::Wall {
                adjacency: Adjacency::Full,
                east: BaseSpriteEast::None,
                south: BaseSpriteSouth::None,
            } => 7,
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::Door,
                south: BaseSpriteSouth::None,
            } => 8,
            BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::Door,
                south: BaseSpriteSouth::None,
            } => 9,
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::Door,
                south: BaseSpriteSouth::Door,
            } => 10,
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::StairN,
                south: BaseSpriteSouth::None,
            } => 11,
            BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairN,
                south: BaseSpriteSouth::None,
            } => 12,
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::StairN,
                south: BaseSpriteSouth::Door,
            } => 13,
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::StairS,
                south: BaseSpriteSouth::None,
            } => 14,
            BaseSprite::Wall {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairS,
                south: BaseSpriteSouth::None,
            } => 15,
            BaseSprite::Wall {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::StairS,
                south: BaseSpriteSouth::Door,
            } => 16,
            BaseSprite::StairN {
                adjacency: Adjacency::Corner,
            } => 17,
            BaseSprite::StairN {
                adjacency: Adjacency::Horizontal,
            } => 18,
            BaseSprite::StairS {
                adjacency: Adjacency::Corner,
                south: BaseSpriteSouth::None,
            } => 19,
            BaseSprite::StairS {
                adjacency: Adjacency::Vertical,
                south: BaseSpriteSouth::None,
            } => 20,
            BaseSprite::StairS {
                adjacency: Adjacency::Corner,
                south: BaseSpriteSouth::Door,
            } => 21,
            BaseSprite::StairS {
                adjacency: Adjacency::Horizontal,
                south: BaseSpriteSouth::None,
            } => 22,
            BaseSprite::StairS {
                adjacency: Adjacency::InverseCorner,
                south: BaseSpriteSouth::None,
            } => 23,
            BaseSprite::StairS {
                adjacency: Adjacency::Horizontal,
                south: BaseSpriteSouth::Door,
            } => 24,
            BaseSprite::StairS {
                adjacency: Adjacency::Full,
                south: BaseSpriteSouth::None,
            } => 25,
            BaseSprite::StairE {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::None,
            } => 26,
            BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::None,
            } => 27,
            BaseSprite::StairE {
                adjacency: Adjacency::Horizontal,
                east: BaseSpriteEast::None,
            } => 28,
            BaseSprite::StairE {
                adjacency: Adjacency::InverseCorner,
                east: BaseSpriteEast::None,
            } => 29,
            BaseSprite::StairE {
                adjacency: Adjacency::Full,
                east: BaseSpriteEast::None,
            } => 30,
            BaseSprite::StairE {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::Door,
            } => 31,
            BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::Door,
            } => 32,
            BaseSprite::StairE {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::StairN,
            } => 33,
            BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairN,
            } => 34,
            BaseSprite::StairE {
                adjacency: Adjacency::Corner,
                east: BaseSpriteEast::StairS,
            } => 35,
            BaseSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: BaseSpriteEast::StairS,
            } => 36,
            BaseSprite::StairW {
                adjacency: Adjacency::Corner,
            } => 37,
            BaseSprite::StairW {
                adjacency: Adjacency::Vertical,
            } => 38,
            _ => unimplemented!("unimplemented BaseSprite {:?}", self),
        }
    }
}

impl TopSprite {
    pub fn id(&self) -> usize {
        match self {
            TopSprite::Empty {
                east: TopSpriteEast::None,
            } => 0,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::None,
            } => 39,
            TopSprite::StairN {
                adjacency: Adjacency::Corner,
                north: TopSpriteNorth::None,
            } => 40,
            TopSprite::StairE {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::None,
            } => 41,
            TopSprite::StairW {
                adjacency: Adjacency::Corner,
            } => 42,
            TopSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::None,
            } => 43,
            TopSprite::StairN {
                adjacency: Adjacency::Horizontal,
                north: TopSpriteNorth::None,
            } => 44,
            TopSprite::StairE {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
            } => 45,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
                north: TopSpriteNorth::None,
            } => 46,
            TopSprite::StairE {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
            } => 47,
            TopSprite::StairN {
                adjacency: Adjacency::Vertical,
                north: TopSpriteNorth::None,
            } => 48,
            TopSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: TopSpriteEast::None,
            } => 49,
            TopSprite::StairW {
                adjacency: Adjacency::Vertical,
            } => 50,
            TopSprite::StairN {
                adjacency: Adjacency::Full,
                north: TopSpriteNorth::None,
            } => 51,
            TopSprite::StairE {
                adjacency: Adjacency::Full,
                east: TopSpriteEast::None,
            } => 52,
            TopSprite::Empty {
                east: TopSpriteEast::StairN,
            } => 53,
            TopSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: TopSpriteEast::StairN,
            } => 54,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::Door,
            } => 55,
            TopSprite::StairN {
                adjacency: Adjacency::Corner,
                north: TopSpriteNorth::Door,
            } => 56,
            TopSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::Door,
            } => 57,
            TopSprite::StairN {
                adjacency: Adjacency::Horizontal,
                north: TopSpriteNorth::Door,
            } => 58,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
                north: TopSpriteNorth::Door,
            } => 59,
            _ => unimplemented!("unimplemented TopSprite {:?}", self),
        }
    }
}
