#![warn(clippy::match_same_arms)]

use std::collections::HashSet;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum TileVariant {
    Empty,
    Wall,
    DoorH,
    DoorV,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
struct Sprite {
    mid: MidSprite,
    top: TopSprite,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum MidSprite {
    Empty,
    Corner {
        east: EastDecoration,
        south: SouthDecoration,
    },
    Horizontal {
        south: SouthDecoration,
    },
    Vertical {
        east: EastDecoration,
    },
    InverseCorner,
    Full,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum TopSprite {
    Empty,
    Horizontal { south: TopSouthDecoration },
    Corner { south: TopSouthDecoration },
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum EastDecoration {
    None,
    Door,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum SouthDecoration {
    None,
    Door,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum TopSouthDecoration {
    None,
    Door,
}

fn main() {
    let mut tops = HashSet::<TopSprite>::new();
    let mut mids = HashSet::<MidSprite>::new();
    let mut sprites = HashSet::<Sprite>::new();

    for center in TileVariant::values() {
        for north in TileVariant::values() {
            for north_east in TileVariant::values() {
                for east in TileVariant::values() {
                    for south_east in TileVariant::values() {
                        for south in TileVariant::values() {
                            let mid = MidSprite::resolve(
                                center, north, north_east, east, south_east, south,
                            );
                            let top = TopSprite::resolve(
                                center, north, north_east, east, south_east, south,
                            );

                            tops.insert(top);
                            mids.insert(mid);
                            sprites.insert(Sprite { mid, top });
                        }
                    }
                }
            }
        }
    }

    println!("Unique tops: {}", tops.len());
    println!("Unique mids: {}", mids.len());
    println!("Unique sprites: {}", sprites.len());
}

impl TileVariant {
    fn values() -> impl IntoIterator<Item = TileVariant> {
        [
            TileVariant::Empty,
            TileVariant::Wall,
            TileVariant::DoorH,
            TileVariant::DoorV,
        ]
    }
}

impl MidSprite {
    fn resolve(
        center: TileVariant,
        north: TileVariant,
        north_east: TileVariant,
        east: TileVariant,
        south_east: TileVariant,
        south: TileVariant,
    ) -> Self {
        match (center, north, north_east, east, south_east, south) {
            (TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV, _, _, _, _, _) => {
                MidSprite::Empty
            }
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorV,
                _,
                TileVariant::Empty | TileVariant::DoorH,
            ) => MidSprite::Corner {
                east: EastDecoration::None,
                south: SouthDecoration::None,
            },
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorV,
                _,
                TileVariant::DoorV,
            ) => MidSprite::Corner {
                east: EastDecoration::None,
                south: SouthDecoration::Door,
            },
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::DoorH,
                _,
                TileVariant::Empty | TileVariant::DoorH,
            ) => MidSprite::Corner {
                east: EastDecoration::Door,
                south: SouthDecoration::None,
            },
            (TileVariant::Wall, _, _, TileVariant::DoorH, _, TileVariant::DoorV) => {
                MidSprite::Corner {
                    east: EastDecoration::Door,
                    south: SouthDecoration::Door,
                }
            }
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Wall,
                _,
                TileVariant::Empty | TileVariant::DoorH,
            ) => MidSprite::Horizontal {
                south: SouthDecoration::None,
            },
            (TileVariant::Wall, _, _, TileVariant::Wall, _, TileVariant::DoorV) => {
                MidSprite::Horizontal {
                    south: SouthDecoration::Door,
                }
            }
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorV,
                _,
                TileVariant::Wall,
            ) => MidSprite::Vertical {
                east: EastDecoration::None,
            },
            (TileVariant::Wall, _, _, TileVariant::DoorH, _, TileVariant::Wall) => {
                MidSprite::Vertical {
                    east: EastDecoration::Door,
                }
            }
            (
                TileVariant::Wall,
                _,
                _,
                TileVariant::Wall,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV,
                TileVariant::Wall,
            ) => MidSprite::Full,
            (TileVariant::Wall, _, _, TileVariant::Wall, TileVariant::Wall, TileVariant::Wall) => {
                MidSprite::InverseCorner
            }
        }
    }
}

impl TopSprite {
    fn resolve(
        center: TileVariant,
        north: TileVariant,
        north_east: TileVariant,
        east: TileVariant,
        south_east: TileVariant,
        south: TileVariant,
    ) -> Self {
        match (center, north, north_east, east, south_east, south) {
            (TileVariant::Wall, _, _, _, _, _)
            | (_, _, _, _, _, TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV) => {
                TopSprite::Empty
            }
            (
                TileVariant::Empty | TileVariant::DoorH,
                _,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV,
                TileVariant::Wall,
            ) => TopSprite::Corner {
                south: TopSouthDecoration::None,
            },
            (
                TileVariant::DoorV,
                _,
                _,
                _,
                TileVariant::Empty | TileVariant::DoorH | TileVariant::DoorV,
                TileVariant::Wall,
            ) => TopSprite::Corner {
                south: TopSouthDecoration::Door,
            },
            (
                TileVariant::Empty | TileVariant::DoorH,
                _,
                _,
                _,
                TileVariant::Wall,
                TileVariant::Wall,
            ) => TopSprite::Horizontal {
                south: TopSouthDecoration::None,
            },
            (TileVariant::DoorV, _, _, _, TileVariant::Wall, TileVariant::Wall) => {
                TopSprite::Horizontal {
                    south: TopSouthDecoration::Door,
                }
            }
        }
    }
}
