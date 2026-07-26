#![warn(clippy::match_same_arms)]

use std::{
    collections::HashSet,
    fmt::{self, Display},
};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum TileKind {
    Empty,
    Wall,
    Door,
    StairN,
    StairE,
    StairS,
    StairW,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Sprite {
    Wall(WallSprite),
    Empty(SouthDecoration),
    DoorV(SouthDoorDecoration),
    StairN {
        east: EastStairDecoration,
        south: SouthDecoration,
    },
    StairE {
        south: HorizontalStairSouthDecoration,
    },
    StairW {
        north: NorthStairDecoration,
        south: HorizontalStairSouthDecoration,
    },
    StairS {
        east: EastStairDecoration,
        south: SouthDecoration,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WallSprite {
    Corner {
        east: EastWallDecoration,
        south: SouthWallDecoration,
    },
    Horizontal {
        south: SouthWallDecoration,
    },
    Vertical {
        east: EastWallDecoration,
        south_east: SouthEastWallDecoration,
    },
    InverseCorner,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SouthDoorDecoration {
    WallCorner,
    WallHorizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EastStairDecoration {
    Empty,
    Stair,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NorthStairDecoration {
    Empty,
    Stair,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SouthDecoration {
    Empty,
    WallCorner,
    WallHorizontal,
    StairE,
    StairW,
    StairN,
    StairNDouble,
    StairNWall,
    StairSWall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HorizontalStairSouthDecoration {
    Empty,
    WallCorner,
    WallHorizontal,
    WallFull,
    StairE,
    StairW,
    StairN,
    StairNDouble,
    StairNWall,
    StairNWallFull,
    StairSWall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EastWallDecoration {
    Empty,
    Door,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SouthEastWallDecoration {
    Empty,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SouthWallDecoration {
    Empty,
    Door,
    StairE,
    StairW,
    StairN,
    StairNDouble,
    StairNWall,
    StairSWall,
}

macro_rules! stairs {
    () => {
        TileKind::StairN | TileKind::StairE | TileKind::StairS | TileKind::StairW
    };
}

macro_rules! esw {
    () => {
        TileKind::Empty | TileKind::Door | TileKind::StairE | TileKind::StairS | TileKind::StairW
    };
}

macro_rules! ewn {
    () => {
        TileKind::Empty | TileKind::Door | TileKind::StairN | TileKind::StairE | TileKind::StairW
    };
}

macro_rules! esn {
    () => {
        TileKind::Empty | TileKind::Door | TileKind::StairE | TileKind::StairS | TileKind::StairN
    };
}

fn main() {
    let mut variants = HashSet::new();
    let mut ordered = Vec::new();

    for center in TileKind::values() {
        for south in TileKind::values() {
            for south_east in TileKind::values() {
                for east in TileKind::values() {
                    // for south_west in TileKind::values() {
                    //     for west in TileKind::values() {
                    for north in TileKind::values() {
                        let right = tile_sprite(center, south, south_east, east, north);
                        // let left = tile_sprite(center, south, south_west, west, north);

                        if variants.insert(right) {
                            ordered.push(right);
                        }
                    }
                    //     }
                    // }
                }
            }
        }
    }

    // println!("Ordered tiles: {:?}", ordered);

    println!("Found {} unique wall sprites", variants.len());

    // let all_sprites: HashSet<Sprite> = {
    //     let mut set = HashSet::new();
    //     for east in EastWallDecoration::values() {
    //         for south in SouthWallDecoration::values() {
    //             set.insert(Sprite::Wall(WallSprite::Corner { east, south }));
    //         }
    //     }
    //     for south in SouthWallDecoration::values() {
    //         set.insert(Sprite::Wall(WallSprite::Horizontal { south }));
    //     }
    //     for east in EastWallDecoration::values() {
    //         for south_east in SouthEastWallDecoration::values() {
    //             set.insert(Sprite::Wall(WallSprite::Vertical { east, south_east }));
    //         }
    //     }
    //     set.insert(Sprite::Wall(WallSprite::InverseCorner));
    //     set.insert(Sprite::Wall(WallSprite::Full));
    //     for south in SouthDecoration::values() {
    //         set.insert(Sprite::Empty(south));
    //     }
    //     for south in SouthDoorDecoration::values() {
    //         set.insert(Sprite::DoorV(south));
    //     }
    //     for east in EastStairDecoration::values() {
    //         for south in SouthDecoration::values() {
    //             set.insert(Sprite::StairN { east, south });
    //         }
    //     }
    //     for south in HorizontalStairSouthDecoration::values() {
    //         set.insert(Sprite::StairE { south });
    //         for north in NorthStairDecoration::values() {
    //             set.insert(Sprite::StairW { south, north });
    //         }
    //     }
    //     for east in EastStairDecoration::values() {
    //         for south in SouthDecoration::values() {
    //             set.insert(Sprite::StairS { east, south });
    //         }
    //     }
    //     set
    // };

    // let mut unreached: Vec<Sprite> = all_sprites.difference(&variants).cloned().collect();
    // unreached.sort_by_key(|s| format!("{s:?}"));
    // println!("Unreached variants ({}):", unreached.len());
    // for sprite in &unreached {
    //     println!("  {sprite:?}");
    // }

    let mut count = 1;
    let mut row = 1;

    println!("Row {row}");
    for tile in ordered {
        println!("{tile}");

        if count == 16 {
            println!();
            row += 1;
            println!("Row {row}");
            count = 0;
        } else {
        }
        count += 1;
    }
}

impl TileKind {
    fn values() -> [TileKind; 7] {
        [
            TileKind::Empty,
            TileKind::Wall,
            TileKind::Door,
            TileKind::StairN,
            TileKind::StairS,
            TileKind::StairE,
            TileKind::StairW,
        ]
    }
}

impl EastWallDecoration {
    fn values() -> [EastWallDecoration; 2] {
        [EastWallDecoration::Empty, EastWallDecoration::Door]
    }
}

impl SouthWallDecoration {
    fn values() -> [SouthWallDecoration; 8] {
        [
            SouthWallDecoration::Empty,
            SouthWallDecoration::Door,
            SouthWallDecoration::StairE,
            SouthWallDecoration::StairW,
            SouthWallDecoration::StairN,
            SouthWallDecoration::StairNDouble,
            SouthWallDecoration::StairNWall,
            SouthWallDecoration::StairSWall,
        ]
    }
}

impl SouthEastWallDecoration {
    fn values() -> [SouthEastWallDecoration; 2] {
        [
            SouthEastWallDecoration::Empty,
            SouthEastWallDecoration::Wall,
        ]
    }
}

impl SouthDecoration {
    fn values() -> [SouthDecoration; 9] {
        [
            SouthDecoration::Empty,
            SouthDecoration::WallCorner,
            SouthDecoration::WallHorizontal,
            SouthDecoration::StairE,
            SouthDecoration::StairW,
            SouthDecoration::StairN,
            SouthDecoration::StairNDouble,
            SouthDecoration::StairNWall,
            SouthDecoration::StairSWall,
        ]
    }
}

impl HorizontalStairSouthDecoration {
    fn values() -> [HorizontalStairSouthDecoration; 11] {
        [
            HorizontalStairSouthDecoration::Empty,
            HorizontalStairSouthDecoration::WallCorner,
            HorizontalStairSouthDecoration::WallHorizontal,
            HorizontalStairSouthDecoration::WallFull,
            HorizontalStairSouthDecoration::StairE,
            HorizontalStairSouthDecoration::StairW,
            HorizontalStairSouthDecoration::StairN,
            HorizontalStairSouthDecoration::StairNDouble,
            HorizontalStairSouthDecoration::StairNWall,
            HorizontalStairSouthDecoration::StairNWallFull,
            HorizontalStairSouthDecoration::StairSWall,
        ]
    }
}

impl SouthDoorDecoration {
    fn values() -> [SouthDoorDecoration; 2] {
        [
            SouthDoorDecoration::WallCorner,
            SouthDoorDecoration::WallHorizontal,
        ]
    }
}

impl EastStairDecoration {
    fn values() -> [EastStairDecoration; 3] {
        [
            EastStairDecoration::Empty,
            EastStairDecoration::Stair,
            EastStairDecoration::Wall,
        ]
    }
}

impl NorthStairDecoration {
    fn values() -> [NorthStairDecoration; 3] {
        [
            NorthStairDecoration::Empty,
            NorthStairDecoration::Stair,
            NorthStairDecoration::Wall,
        ]
    }
}

fn tile_sprite(
    center: TileKind,
    south: TileKind,
    south_east: TileKind,
    east: TileKind,
    north: TileKind,
) -> Sprite {
    use TileKind::*;

    match (center, south, south_east, east, north) {
        (Empty | Door, Empty | Door, _, _, _)
        | (Empty | Door, StairS, Empty | Door | stairs!(), _, _) => {
            Sprite::Empty(SouthDecoration::Empty)
        }
        (Empty, Wall, Wall, _, _) | (Empty, Wall, stairs!(), Wall, _) => {
            Sprite::Empty(SouthDecoration::WallHorizontal)
        }
        (Empty, Wall, stairs!(), Empty | Door | stairs!(), _)
        | (Empty, Wall, Empty | Door, _, _) => Sprite::Empty(SouthDecoration::WallCorner),
        (Empty | Door, StairN, esw!(), _, _) => Sprite::Empty(SouthDecoration::StairN),
        (Empty | Door, StairN, Wall, _, _) => Sprite::Empty(SouthDecoration::StairNWall),
        (Empty | Door, StairN, StairN, _, _) => Sprite::Empty(SouthDecoration::StairNDouble),
        (Empty | Door, StairE, _, _, _) => Sprite::Empty(SouthDecoration::StairE),
        (Empty | Door, StairW, _, _, _) => Sprite::Empty(SouthDecoration::StairW),
        (Empty | Door, StairS, Wall, _, _) => Sprite::Empty(SouthDecoration::StairSWall),
        (Door, Wall, Wall, _, _) | (Door, Wall, stairs!(), Wall, _) => {
            Sprite::DoorV(SouthDoorDecoration::WallHorizontal)
        }
        (Door, Wall, stairs!(), Empty | Door | stairs!(), _) | (Door, Wall, Empty | Door, _, _) => {
            Sprite::DoorV(SouthDoorDecoration::WallCorner)
        }
        (Wall, Empty, _, Empty, _) | (Wall, StairS, Empty | Door | stairs!(), Empty, _) => {
            Sprite::Wall(WallSprite::Corner {
                east: EastWallDecoration::Empty,
                south: SouthWallDecoration::Empty,
            })
        }
        (Wall, Empty, _, Door, _) | (Wall, StairS, Empty | Door | stairs!(), Door, _) => {
            Sprite::Wall(WallSprite::Corner {
                east: EastWallDecoration::Door,
                south: SouthWallDecoration::Empty,
            })
        }
        (Wall, Door, _, Empty, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::Door,
        }),
        (Wall, Door, _, Door, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::Door,
        }),
        (Wall, Empty, _, Wall | stairs!(), _)
        | (Wall, StairS, Empty | Door | stairs!(), Wall | stairs!(), _) => {
            Sprite::Wall(WallSprite::Horizontal {
                south: SouthWallDecoration::Empty,
            })
        }
        (Wall, Door, _, Wall | stairs!(), _) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::Door,
        }),
        (Wall, Wall, Empty | Door, Empty, _) => Sprite::Wall(WallSprite::Vertical {
            east: EastWallDecoration::Empty,
            south_east: SouthEastWallDecoration::Empty,
        }),
        (Wall, Wall, Empty | Door, Wall | stairs!(), _) => Sprite::Wall(WallSprite::InverseCorner),
        (Wall, Wall, Empty | Door, Door, _) => Sprite::Wall(WallSprite::Vertical {
            east: EastWallDecoration::Door,
            south_east: SouthEastWallDecoration::Empty,
        }),
        (Wall, Wall, Wall | stairs!(), Empty, _) => Sprite::Wall(WallSprite::Vertical {
            east: EastWallDecoration::Empty,
            south_east: SouthEastWallDecoration::Wall,
        }),
        (Wall, Wall, Wall | stairs!(), Door, _) => Sprite::Wall(WallSprite::Vertical {
            east: EastWallDecoration::Door,
            south_east: SouthEastWallDecoration::Wall,
        }),
        (Wall, Wall, Wall | stairs!(), Wall | stairs!(), _) => Sprite::Wall(WallSprite::Full),
        (Wall, StairN, esw!(), Empty, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairN,
        }),
        (Wall, StairN, esw!(), Door, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairN,
        }),
        (Wall, StairN, esw!(), Wall | stairs!(), _) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::StairN,
        }),
        (Wall, StairN, Wall, Empty, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairNWall,
        }),
        (Wall, StairN, Wall, Door, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairNWall,
        }),
        (Wall, StairN, Wall, Wall | stairs!(), _) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::StairNWall,
        }),
        (Wall, StairN, StairN, Empty, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairNDouble,
        }),
        (Wall, StairN, StairN, Door, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairNDouble,
        }),
        (Wall, StairN, StairN, Wall | stairs!(), _) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::StairNDouble,
        }),
        (Wall, StairS, Wall, Empty, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairSWall,
        }),
        (Wall, StairS, Wall, Door, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairSWall,
        }),
        (Wall, StairS, Wall, Wall | stairs!(), _) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::StairSWall,
        }),
        (Wall, StairE, _, Empty, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairE,
        }),
        (Wall, StairE, _, Door, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairE,
        }),
        (Wall, StairE, Empty | Door | stairs!(), Wall | stairs!(), _) => {
            Sprite::Wall(WallSprite::Horizontal {
                south: SouthWallDecoration::StairE,
            })
        }
        (Wall, StairE, Wall, Wall | stairs!(), _) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::StairE,
        }),
        (Wall, StairW, _, Empty, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairW,
        }),
        (Wall, StairW, _, Door, _) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairW,
        }),
        (Wall, StairW, Empty | Door | stairs!(), Wall | stairs!(), _) => {
            Sprite::Wall(WallSprite::Horizontal {
                south: SouthWallDecoration::StairW,
            })
        }
        (Wall, StairW, Wall, Wall | stairs!(), _) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::StairW,
        }),
        (StairN, Empty | Door, _, esw!(), _)
        | (StairN, StairS, Empty | Door | stairs!(), esw!(), _) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::Empty,
        },
        (StairN, Empty | Door, _, Wall, _)
        | (StairN, StairS, Empty | Door | stairs!(), Wall, _) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::Empty,
        },
        (StairN, Empty | Door, _, StairN, _)
        | (StairN, StairS, Empty | Door | stairs!(), StairN, _) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::Empty,
        },
        (StairN, Wall, Empty | Door, esw!(), _) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::WallCorner,
        },
        (StairN, Wall, Empty | Door, Wall, _) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::WallCorner,
        },
        (StairN, Wall, Empty | Door, StairN, _) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::WallCorner,
        },
        (StairN, Wall, Wall | stairs!(), esw!(), _) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::WallHorizontal,
        },
        (StairN, Wall, Wall | stairs!(), Wall, _) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::WallHorizontal,
        },
        (StairN, Wall, Wall | stairs!(), StairN, _) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::WallHorizontal,
        },
        (StairN, StairN, esw!(), esw!(), _) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairN,
        },
        (StairN, StairN, esw!(), Wall, _) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairN,
        },
        (StairN, StairN, esw!(), StairN, _) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairN,
        },
        (StairN, StairN, Wall, esw!(), _) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairNWall,
        },
        (StairN, StairN, Wall, Wall, _) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairNWall,
        },
        (StairN, StairN, Wall, StairN, _) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairNWall,
        },
        (StairN, StairN, StairN, esw!(), _) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairNDouble,
        },
        (StairN, StairN, StairN, Wall, _) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairNDouble,
        },
        (StairN, StairN, StairN, StairN, _) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairNDouble,
        },
        (StairN, StairS, Wall, esw!(), _) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairSWall,
        },
        (StairN, StairS, Wall, Wall, _) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairSWall,
        },
        (StairN, StairS, Wall, StairN, _) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairSWall,
        },
        (StairN, StairE, _, esw!(), _) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairE,
        },
        (StairN, StairW, _, esw!(), _) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairW,
        },
        (StairN, StairE, _, Wall, _) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairE,
        },
        (StairN, StairW, _, Wall, _) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairW,
        },
        (StairN, StairE, _, StairN, _) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairE,
        },
        (StairN, StairW, _, StairN, _) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairW,
        },
        (StairS, Empty | Door, _, ewn!(), _)
        | (StairS, StairS, Empty | Door | stairs!(), ewn!(), _) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::Empty,
        },
        (StairS, Empty | Door, _, Wall, _)
        | (StairS, StairS, Empty | Door | stairs!(), Wall, _) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::Empty,
        },
        (StairS, Empty | Door, _, StairS, _)
        | (StairS, StairS, Empty | Door | stairs!(), StairS, _) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::Empty,
        },
        (StairS, Wall, Empty | Door, ewn!(), _) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::WallCorner,
        },
        (StairS, Wall, Empty | Door, Wall, _) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::WallCorner,
        },
        (StairS, Wall, Empty | Door, StairS, _) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::WallCorner,
        },
        (StairS, Wall, Wall | stairs!(), ewn!(), _) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::WallHorizontal,
        },
        (StairS, Wall, Wall | stairs!(), Wall, _) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::WallHorizontal,
        },
        (StairS, Wall, Wall | stairs!(), StairS, _) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::WallHorizontal,
        },
        (StairS, StairN, esw!(), ewn!(), _) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairN,
        },
        (StairS, StairN, esw!(), Wall, _) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairN,
        },
        (StairS, StairN, esw!(), StairS, _) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairN,
        },
        (StairS, StairN, Wall, ewn!(), _) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairNWall,
        },
        (StairS, StairN, Wall, Wall, _) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairNWall,
        },
        (StairS, StairN, Wall, StairS, _) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairNWall,
        },
        (StairS, StairN, StairN, ewn!(), _) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairNDouble,
        },
        (StairS, StairN, StairN, Wall, _) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairNDouble,
        },
        (StairS, StairN, StairN, StairS, _) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairNDouble,
        },
        (StairS, StairS, Wall, ewn!(), _) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairSWall,
        },
        (StairS, StairS, Wall, Wall, _) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairSWall,
        },
        (StairS, StairS, Wall, StairS, _) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairSWall,
        },
        (StairS, StairE, _, ewn!(), _) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairE,
        },
        (StairS, StairW, _, ewn!(), _) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairW,
        },
        (StairS, StairE, _, Wall, _) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairE,
        },
        (StairS, StairW, _, Wall, _) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairW,
        },
        (StairS, StairE, _, StairS, _) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairE,
        },
        (StairS, StairW, _, StairS, _) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairW,
        },
        (StairE, Empty | Door, _, _, _) | (StairE, StairS, Empty | Door | stairs!(), _, _) => {
            Sprite::StairE {
                south: HorizontalStairSouthDecoration::Empty,
            }
        }
        (StairE, Wall, Empty | Door, _, _) => Sprite::StairE {
            south: HorizontalStairSouthDecoration::WallCorner,
        },
        (StairE, Wall, Wall | stairs!(), Empty | Door | stairs!(), _) => Sprite::StairE {
            south: HorizontalStairSouthDecoration::WallHorizontal,
        },
        (StairE, Wall, Wall | stairs!(), Wall, _) => Sprite::StairE {
            south: HorizontalStairSouthDecoration::WallFull,
        },
        (StairE, StairN, esw!(), _, _) => Sprite::StairE {
            south: HorizontalStairSouthDecoration::StairN,
        },
        (StairE, StairN, Wall, Empty | Door | stairs!(), _) => Sprite::StairE {
            south: HorizontalStairSouthDecoration::StairNWall,
        },
        (StairE, StairN, Wall, Wall, _) => Sprite::StairE {
            south: HorizontalStairSouthDecoration::StairNWallFull,
        },
        (StairE, StairN, StairN, _, _) => Sprite::StairE {
            south: HorizontalStairSouthDecoration::StairNDouble,
        },
        (StairE, StairS, Wall, _, _) => Sprite::StairE {
            south: HorizontalStairSouthDecoration::StairSWall,
        },
        (StairE, StairE, _, _, _) => Sprite::StairE {
            south: HorizontalStairSouthDecoration::StairE,
        },
        (StairE, StairW, _, _, _) => Sprite::StairE {
            south: HorizontalStairSouthDecoration::StairW,
        },
        (StairW, Empty | Door, _, _, esn!())
        | (StairW, StairS, Empty | Door | stairs!(), _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: HorizontalStairSouthDecoration::Empty,
        },
        (StairW, Empty | Door, _, _, Wall)
        | (StairW, StairS, Empty | Door | stairs!(), _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: HorizontalStairSouthDecoration::Empty,
        },
        (StairW, Empty | Door, _, _, StairW)
        | (StairW, StairS, Empty | Door | stairs!(), _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: HorizontalStairSouthDecoration::Empty,
        },
        (StairW, Wall, Empty | Door, _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: HorizontalStairSouthDecoration::WallCorner,
        },
        (StairW, Wall, Empty | Door, _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: HorizontalStairSouthDecoration::WallCorner,
        },
        (StairW, Wall, Empty | Door, _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: HorizontalStairSouthDecoration::WallCorner,
        },
        (StairW, Wall, Wall | stairs!(), Empty | Door | stairs!(), esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: HorizontalStairSouthDecoration::WallHorizontal,
        },
        (StairW, Wall, Wall | stairs!(), Wall, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: HorizontalStairSouthDecoration::WallFull,
        },
        (StairW, Wall, Wall | stairs!(), Empty | Door | stairs!(), Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: HorizontalStairSouthDecoration::WallHorizontal,
        },
        (StairW, Wall, Wall | stairs!(), Wall, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: HorizontalStairSouthDecoration::WallFull,
        },
        (StairW, Wall, Wall | stairs!(), Empty | Door | stairs!(), StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: HorizontalStairSouthDecoration::WallHorizontal,
        },
        (StairW, Wall, Wall | stairs!(), Wall, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: HorizontalStairSouthDecoration::WallFull,
        },
        (StairW, StairN, esw!(), _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: HorizontalStairSouthDecoration::StairN,
        },
        (StairW, StairN, esw!(), _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: HorizontalStairSouthDecoration::StairN,
        },
        (StairW, StairN, esw!(), _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: HorizontalStairSouthDecoration::StairN,
        },
        (StairW, StairN, Wall, Empty | Door | stairs!(), esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: HorizontalStairSouthDecoration::StairNWall,
        },
        (StairW, StairN, Wall, Wall, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: HorizontalStairSouthDecoration::StairNWallFull,
        },
        (StairW, StairN, Wall, Empty | Door | stairs!(), Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: HorizontalStairSouthDecoration::StairNWall,
        },
        (StairW, StairN, Wall, Wall, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: HorizontalStairSouthDecoration::StairNWallFull,
        },
        (StairW, StairN, Wall, Empty | Door | stairs!(), StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: HorizontalStairSouthDecoration::StairNWall,
        },
        (StairW, StairN, Wall, Wall, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: HorizontalStairSouthDecoration::StairNWallFull,
        },
        (StairW, StairN, StairN, _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: HorizontalStairSouthDecoration::StairNDouble,
        },
        (StairW, StairN, StairN, _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: HorizontalStairSouthDecoration::StairNDouble,
        },
        (StairW, StairN, StairN, _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: HorizontalStairSouthDecoration::StairNDouble,
        },
        (StairW, StairS, Wall, _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: HorizontalStairSouthDecoration::StairSWall,
        },
        (StairW, StairS, Wall, _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: HorizontalStairSouthDecoration::StairSWall,
        },
        (StairW, StairS, Wall, _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: HorizontalStairSouthDecoration::StairSWall,
        },
        (StairW, StairE, _, _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: HorizontalStairSouthDecoration::StairE,
        },
        (StairW, StairE, _, _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: HorizontalStairSouthDecoration::StairE,
        },
        (StairW, StairE, _, _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: HorizontalStairSouthDecoration::StairE,
        },
        (StairW, StairW, _, _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: HorizontalStairSouthDecoration::StairW,
        },
        (StairW, StairW, _, _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: HorizontalStairSouthDecoration::StairW,
        },
        (StairW, StairW, _, _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: HorizontalStairSouthDecoration::StairW,
        },
    }
}

impl Display for Sprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sprite::Wall(WallSprite::Corner { east, south }) => {
                write!(f, "WallCorner{south}{east}")
            }
            Sprite::Wall(WallSprite::Horizontal { south }) => {
                write!(f, "WallHorizontal{south}")
            }
            Sprite::Wall(WallSprite::Vertical { east, south_east }) => {
                write!(f, "WallVertical{south_east}{east}")
            }
            Sprite::Wall(WallSprite::InverseCorner) => write!(f, "WallInverseCorner"),
            Sprite::Wall(WallSprite::Full) => write!(f, "WallFull"),
            Sprite::Empty(south) => write!(f, "Empty{south}"),
            Sprite::DoorV(south) => write!(f, "Door{south}"),
            Sprite::StairN { east, south } => write!(f, "StairN{south}{east}"),
            Sprite::StairS { east, south } => {
                write!(f, "StairS{south}{east}")
            }
            Sprite::StairE { south } => write!(f, "StairE{south}"),
            Sprite::StairW { south, north } => write!(f, "StairW{south}{north}"),
        }
    }
}

impl Display for EastWallDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EastWallDecoration::Empty => Ok(()),
            EastWallDecoration::Door => write!(f, "_EastDoor"),
        }
    }
}

impl Display for SouthWallDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SouthWallDecoration::Empty => Ok(()),
            SouthWallDecoration::Door => write!(f, "_SouthDoor"),
            SouthWallDecoration::StairE => write!(f, "_SouthStairE"),
            SouthWallDecoration::StairW => write!(f, "_SouthStairW"),
            SouthWallDecoration::StairN => write!(f, "_SouthStairN"),
            SouthWallDecoration::StairNDouble => write!(f, "_SouthStairNDouble"),
            SouthWallDecoration::StairNWall => write!(f, "_SouthStairNWall"),
            SouthWallDecoration::StairSWall => write!(f, "_SouthStairSWall"),
        }
    }
}

impl Display for SouthEastWallDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SouthEastWallDecoration::Empty => Ok(()),
            SouthEastWallDecoration::Wall => write!(f, "_SouthEastWall"),
        }
    }
}

impl Display for SouthDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SouthDecoration::Empty => Ok(()),
            SouthDecoration::WallCorner => write!(f, "_SouthWallCorner"),
            SouthDecoration::WallHorizontal => write!(f, "_SouthWallHorizontal"),
            SouthDecoration::StairE => write!(f, "_SouthStairE"),
            SouthDecoration::StairW => write!(f, "_SouthStairW"),
            SouthDecoration::StairN => write!(f, "_SouthStairN"),
            SouthDecoration::StairNDouble => write!(f, "_SouthStairNDouble"),
            SouthDecoration::StairNWall => write!(f, "_SouthStairNWall"),
            SouthDecoration::StairSWall => write!(f, "_SouthStairSWall"),
        }
    }
}

impl Display for HorizontalStairSouthDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HorizontalStairSouthDecoration::Empty => Ok(()),
            HorizontalStairSouthDecoration::WallCorner => write!(f, "_SouthWallCorner"),
            HorizontalStairSouthDecoration::WallHorizontal => write!(f, "_SouthWallHorizontal"),
            HorizontalStairSouthDecoration::WallFull => write!(f, "_SouthWallFull"),
            HorizontalStairSouthDecoration::StairE => write!(f, "_SouthStairE"),
            HorizontalStairSouthDecoration::StairW => write!(f, "_SouthStairW"),
            HorizontalStairSouthDecoration::StairN => write!(f, "_SouthStairN"),
            HorizontalStairSouthDecoration::StairNDouble => write!(f, "_SouthStairNDouble"),
            HorizontalStairSouthDecoration::StairNWall => write!(f, "_SouthStairNWall"),
            HorizontalStairSouthDecoration::StairNWallFull => write!(f, "_SouthStairNWallFull"),
            HorizontalStairSouthDecoration::StairSWall => write!(f, "_SouthStairSWall"),
        }
    }
}

impl Display for SouthDoorDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SouthDoorDecoration::WallCorner => write!(f, "_SouthWallCorner"),
            SouthDoorDecoration::WallHorizontal => write!(f, "_SouthWallHorizontal"),
        }
    }
}

impl Display for EastStairDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EastStairDecoration::Empty => Ok(()),
            EastStairDecoration::Stair => write!(f, "_EastStair"),
            EastStairDecoration::Wall => write!(f, "_EastWall"),
        }
    }
}

impl Display for NorthStairDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NorthStairDecoration::Empty => Ok(()),
            NorthStairDecoration::Stair => write!(f, "_NorthStair"),
            NorthStairDecoration::Wall => write!(f, "_NorthWall"),
        }
    }
}
