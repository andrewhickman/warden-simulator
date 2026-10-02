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
            _ => unreachable!("id() called on a sprite not produced by the enumerator: {self:?}"),
        }
    }
}

impl TopSprite {
    pub fn id(&self) -> usize {
        match self {
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::None,
            } => 0,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::None,
            } => 39,
            TopSprite::StairN {
                adjacency: Adjacency::Corner,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::None,
            } => 40,
            TopSprite::StairE {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::None,
                overlap: OverlapBase::None,
            } => 41,
            TopSprite::StairW {
                adjacency: Adjacency::Corner,
                overlap: OverlapBase::None,
            } => 42,
            TopSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::None,
            } => 43,
            TopSprite::StairN {
                adjacency: Adjacency::Horizontal,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::None,
            } => 44,
            TopSprite::StairE {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
                overlap: OverlapBase::None,
            } => 45,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::None,
            } => 46,
            TopSprite::StairE {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
                overlap: OverlapBase::None,
            } => 47,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallCorner,
            } => 48,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallVertical,
            } => 49,
            TopSprite::StairN {
                adjacency: Adjacency::Vertical,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::WallVertical,
            } => 50,
            TopSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallCorner,
            } => 51,
            TopSprite::StairW {
                adjacency: Adjacency::Vertical,
                overlap: OverlapBase::WallCorner,
            } => 52,
            TopSprite::StairN {
                adjacency: Adjacency::Full,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::WallVertical,
            } => 53,
            TopSprite::StairE {
                adjacency: Adjacency::Full,
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallCorner,
            } => 54,
            TopSprite::Empty {
                east: TopSpriteEast::StairN,
                overlap: OverlapBase::WallVertical,
            } => 55,
            TopSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: TopSpriteEast::StairN,
                overlap: OverlapBase::WallCorner,
            } => 56,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallHorizontal,
            } => 57,
            TopSprite::StairN {
                adjacency: Adjacency::Vertical,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::None,
            } => 58,
            TopSprite::StairW {
                adjacency: Adjacency::Vertical,
                overlap: OverlapBase::WallHorizontal,
            } => 59,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallFull,
            } => 60,
            TopSprite::StairN {
                adjacency: Adjacency::Full,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::WallFull,
            } => 61,
            TopSprite::StairE {
                adjacency: Adjacency::Full,
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallHorizontal,
            } => 62,
            TopSprite::Empty {
                east: TopSpriteEast::StairN,
                overlap: OverlapBase::WallFull,
            } => 63,
            TopSprite::StairE {
                adjacency: Adjacency::Vertical,
                east: TopSpriteEast::StairN,
                overlap: OverlapBase::WallHorizontal,
            } => 64,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallCornerEastDoor,
            } => 65,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallVerticalEastDoor,
            } => 66,
            TopSprite::StairN {
                adjacency: Adjacency::Vertical,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::WallVerticalEastDoor,
            } => 67,
            TopSprite::StairW {
                adjacency: Adjacency::Vertical,
                overlap: OverlapBase::WallCornerEastDoor,
            } => 68,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallCornerEastStairN,
            } => 69,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallVerticalEastStairN,
            } => 70,
            TopSprite::StairN {
                adjacency: Adjacency::Vertical,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::WallVerticalEastStairN,
            } => 71,
            TopSprite::StairW {
                adjacency: Adjacency::Vertical,
                overlap: OverlapBase::WallCornerEastStairN,
            } => 72,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallCornerEastStairS,
            } => 73,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::WallVerticalEastStairS,
            } => 74,
            TopSprite::StairN {
                adjacency: Adjacency::Vertical,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::WallVerticalEastStairS,
            } => 75,
            TopSprite::StairW {
                adjacency: Adjacency::Vertical,
                overlap: OverlapBase::WallCornerEastStairS,
            } => 76,
            TopSprite::StairN {
                adjacency: Adjacency::Full,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::WallVerticalEastStairS,
            } => 77,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::Door,
                overlap: OverlapBase::None,
            } => 78,
            TopSprite::StairN {
                adjacency: Adjacency::Corner,
                north: TopSpriteNorth::Door,
                overlap: OverlapBase::None,
            } => 79,
            TopSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::Door,
                overlap: OverlapBase::None,
            } => 80,
            TopSprite::StairN {
                adjacency: Adjacency::Horizontal,
                north: TopSpriteNorth::Door,
                overlap: OverlapBase::None,
            } => 81,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
                north: TopSpriteNorth::Door,
                overlap: OverlapBase::None,
            } => 82,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::StairSouthCorner,
            } => 83,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairSouthVertical,
            } => 84,
            TopSprite::StairN {
                adjacency: Adjacency::Vertical,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairSouthVertical,
            } => 85,
            TopSprite::StairW {
                adjacency: Adjacency::Vertical,
                overlap: OverlapBase::StairSouthCorner,
            } => 86,
            TopSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairSouthVertical,
            } => 87,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairSouthVertical,
            } => 88,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::StairSouthHorizontal,
            } => 89,
            TopSprite::StairN {
                adjacency: Adjacency::Vertical,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairSouthInverseCorner,
            } => 90,
            TopSprite::StairW {
                adjacency: Adjacency::Vertical,
                overlap: OverlapBase::StairSouthHorizontal,
            } => 91,
            TopSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairSouthFull,
            } => 92,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairSouthFull,
            } => 93,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairEastVertical,
            } => 94,
            TopSprite::StairN {
                adjacency: Adjacency::Corner,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairEastVertical,
            } => 95,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::StairEastVertical,
            } => 96,
            TopSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairEastVertical,
            } => 97,
            TopSprite::StairN {
                adjacency: Adjacency::Horizontal,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairEastVertical,
            } => 98,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairEastVertical,
            } => 99,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::StairEastInverseCorner,
            } => 100,
            TopSprite::Wall {
                adjacency: Adjacency::Horizontal,
                east: TopSpriteEast::None,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairEastFull,
            } => 101,
            TopSprite::StairN {
                adjacency: Adjacency::Horizontal,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairEastFull,
            } => 102,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::StairEastFull,
            } => 103,
            TopSprite::Wall {
                adjacency: Adjacency::Corner,
                east: TopSpriteEast::StairN,
                north: TopSpriteNorth::None,
                overlap: OverlapBase::StairEastFull,
            } => 104,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::StairEastVerticalEastDoor,
            } => 105,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::StairEastVerticalEastStairN,
            } => 106,
            TopSprite::Empty {
                east: TopSpriteEast::None,
                overlap: OverlapBase::StairEastVerticalEastStairS,
            } => 107,
            _ => unreachable!("id() called on a sprite not produced by the enumerator: {self:?}"),
        }
    }
}
