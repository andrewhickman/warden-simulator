#![warn(clippy::match_same_arms)]

use std::collections::HashSet;

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
        south: SouthDecoration,
    },
    StairW {
        south: SouthDecoration,
    },
    StairWDouble,
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
enum SouthDecoration {
    Empty,
    WallCorner,
    WallHorizontal,
    StairE,
    StairN,
    StairNDouble,
    StairNWall,
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
        TileKind::StairE | TileKind::StairS | TileKind::StairW
    };
}

macro_rules! ewn {
    () => {
        TileKind::StairN | TileKind::StairE | TileKind::StairW
    };
}

fn main() {
    let mut variants = HashSet::new();
    let mut ordered = Vec::new();

    for center in TileKind::values() {
        for south in TileKind::values() {
            for south_east in TileKind::values() {
                for east in TileKind::values() {
                    let tile = tile_sprite(center, south, south_east, east);

                    if variants.insert(tile) {
                        ordered.push(tile);
                    }
                }
            }
        }
    }

    // println!("Ordered tiles: {:?}", ordered);

    // let mut count = 0;
    // for tile in ordered {
    //     println!("    {},", tile);

    //     if count == 14 {
    //         println!();
    //         count = 0;
    //     } else {
    //         count += 1;
    //     }
    // }

    println!("Found {} unique wall sprites", variants.len());

    let all_sprites: HashSet<Sprite> = {
        let mut set = HashSet::new();
        for east in EastWallDecoration::values() {
            for south in SouthWallDecoration::values() {
                set.insert(Sprite::Wall(WallSprite::Corner { east, south }));
            }
        }
        for south in SouthWallDecoration::values() {
            set.insert(Sprite::Wall(WallSprite::Horizontal { south }));
        }
        for east in EastWallDecoration::values() {
            for south_east in SouthEastWallDecoration::values() {
                set.insert(Sprite::Wall(WallSprite::Vertical { east, south_east }));
            }
        }
        set.insert(Sprite::Wall(WallSprite::InverseCorner));
        set.insert(Sprite::Wall(WallSprite::Full));
        for south in SouthDecoration::values() {
            set.insert(Sprite::Empty(south));
        }
        for south in SouthDoorDecoration::values() {
            set.insert(Sprite::DoorV(south));
        }
        for east in EastStairDecoration::values() {
            for south in SouthDecoration::values() {
                set.insert(Sprite::StairN { east, south });
            }
        }
        for south in SouthDecoration::values() {
            set.insert(Sprite::StairE { south });
        }
        for south in SouthDecoration::values() {
            set.insert(Sprite::StairW { south });
        }
        set.insert(Sprite::StairWDouble);
        for east in EastStairDecoration::values() {
            for south in SouthDecoration::values() {
                set.insert(Sprite::StairS { east, south });
            }
        }
        set
    };

    let mut unreached: Vec<Sprite> = all_sprites.difference(&variants).cloned().collect();
    unreached.sort_by_key(|s| format!("{s:?}"));
    println!("Unreached variants ({}):", unreached.len());
    for sprite in &unreached {
        println!("  {sprite:?}");
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
    fn values() -> [SouthDecoration; 8] {
        [
            SouthDecoration::Empty,
            SouthDecoration::WallCorner,
            SouthDecoration::WallHorizontal,
            SouthDecoration::StairE,
            SouthDecoration::StairN,
            SouthDecoration::StairNDouble,
            SouthDecoration::StairNWall,
            SouthDecoration::StairSWall,
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

fn tile_sprite(center: TileKind, south: TileKind, south_east: TileKind, east: TileKind) -> Sprite {
    use TileKind::*;

    match (center, south, south_east, east) {
        (Empty | Door, Empty | Door | StairW, _, _)
        | (Empty | Door, StairS, Empty | Door | stairs!(), _) => {
            Sprite::Empty(SouthDecoration::Empty)
        }
        (Empty, Wall, Wall, _) | (Empty, Wall, stairs!(), Wall) => {
            Sprite::Empty(SouthDecoration::WallHorizontal)
        }
        (Empty, Wall, stairs!(), Empty | Door | stairs!()) | (Empty, Wall, Empty | Door, _) => {
            Sprite::Empty(SouthDecoration::WallCorner)
        }
        (Empty | Door, StairN, Empty | Door | esw!(), _) => Sprite::Empty(SouthDecoration::StairN),
        (Empty | Door, StairN, Wall, _) => Sprite::Empty(SouthDecoration::StairNWall),
        (Empty | Door, StairN, StairN, _) => Sprite::Empty(SouthDecoration::StairNDouble),
        (Empty | Door, StairE, _, _) => Sprite::Empty(SouthDecoration::StairE),
        (Empty | Door, StairS, Wall, _) => Sprite::Empty(SouthDecoration::StairSWall),
        (Door, Wall, Wall, _) | (Door, Wall, stairs!(), Wall) => {
            Sprite::DoorV(SouthDoorDecoration::WallHorizontal)
        }
        (Door, Wall, stairs!(), Empty | Door | stairs!()) | (Door, Wall, Empty | Door, _) => {
            Sprite::DoorV(SouthDoorDecoration::WallCorner)
        }
        (Wall, Empty, _, Empty) | (Wall, StairS, Empty | Door | stairs!(), Empty) => {
            Sprite::Wall(WallSprite::Corner {
                east: EastWallDecoration::Empty,
                south: SouthWallDecoration::Empty,
            })
        }
        (Wall, Door, _, Empty) | (Wall, StairS, Empty | Door | stairs!(), Door) => {
            Sprite::Wall(WallSprite::Corner {
                east: EastWallDecoration::Door,
                south: SouthWallDecoration::Empty,
            })
        }
        (Wall, Empty, _, Door) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::Door,
        }),
        (Wall, Door, _, Door) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::Door,
        }),
        (Wall, Empty, _, Wall | stairs!())
        | (Wall, StairS, Empty | Door | stairs!(), Wall | stairs!()) => {
            Sprite::Wall(WallSprite::Horizontal {
                south: SouthWallDecoration::Empty,
            })
        }
        (Wall, Door, _, Wall | stairs!()) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::Door,
        }),
        (Wall, Wall, Empty | Door, Empty) => Sprite::Wall(WallSprite::Vertical {
            east: EastWallDecoration::Empty,
            south_east: SouthEastWallDecoration::Empty,
        }),
        (Wall, Wall, Empty | Door, Wall | stairs!()) => Sprite::Wall(WallSprite::InverseCorner),
        (Wall, Wall, Empty | Door, Door) => Sprite::Wall(WallSprite::Vertical {
            east: EastWallDecoration::Door,
            south_east: SouthEastWallDecoration::Empty,
        }),
        (Wall, Wall, Wall | stairs!(), Empty) => Sprite::Wall(WallSprite::Vertical {
            east: EastWallDecoration::Empty,
            south_east: SouthEastWallDecoration::Wall,
        }),
        (Wall, Wall, Wall | stairs!(), Door) => Sprite::Wall(WallSprite::Vertical {
            east: EastWallDecoration::Door,
            south_east: SouthEastWallDecoration::Wall,
        }),
        (Wall, Wall, Wall | stairs!(), Wall | stairs!()) => Sprite::Wall(WallSprite::Full),
        (Wall, StairN, Empty | Door | esw!(), Empty) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairN,
        }),
        (Wall, StairN, Empty | Door | esw!(), Door) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairN,
        }),
        (Wall, StairN, Empty | Door | esw!(), Wall | stairs!()) => {
            Sprite::Wall(WallSprite::Horizontal {
                south: SouthWallDecoration::StairN,
            })
        }
        (Wall, StairN, Wall, Empty) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairNWall,
        }),
        (Wall, StairN, Wall, Door) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairNWall,
        }),
        (Wall, StairN, Wall, Wall | stairs!()) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::StairNWall,
        }),
        (Wall, StairN, StairN, Empty) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairNDouble,
        }),
        (Wall, StairN, StairN, Door) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairNDouble,
        }),
        (Wall, StairN, StairN, Wall | stairs!()) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::StairNDouble,
        }),
        (Wall, StairS, Wall, Empty) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairSWall,
        }),
        (Wall, StairS, Wall, Door) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairSWall,
        }),
        (Wall, StairS, Wall, Wall | stairs!()) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::StairSWall,
        }),
        (Wall, StairE, _, Empty) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairE,
        }),
        (Wall, StairE, _, Door) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairE,
        }),
        (Wall, StairE, Empty | Door | stairs!(), Wall | stairs!()) => {
            Sprite::Wall(WallSprite::Horizontal {
                south: SouthWallDecoration::StairE,
            })
        }
        (Wall, StairE, Wall, Wall | stairs!()) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::StairE,
        }),
        (Wall, StairW, _, Empty) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Empty,
            south: SouthWallDecoration::StairW,
        }),
        (Wall, StairW, _, Door) => Sprite::Wall(WallSprite::Corner {
            east: EastWallDecoration::Door,
            south: SouthWallDecoration::StairW,
        }),
        (Wall, StairW, Empty | Door | stairs!(), Wall | stairs!()) => {
            Sprite::Wall(WallSprite::Horizontal {
                south: SouthWallDecoration::StairW,
            })
        }
        (Wall, StairW, Wall, Wall | stairs!()) => Sprite::Wall(WallSprite::Horizontal {
            south: SouthWallDecoration::StairW,
        }),
        (StairN, Empty | Door | StairW, _, Empty | Door | esw!())
        | (StairN, StairS, Empty | Door | stairs!(), Empty | Door | esw!()) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::Empty,
        },
        (StairN, Empty | Door | StairW, _, Wall)
        | (StairN, StairS, Empty | Door | stairs!(), Wall) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::Empty,
        },
        (StairN, Empty | Door | StairW, _, StairN)
        | (StairN, StairS, Empty | Door | stairs!(), StairN) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::Empty,
        },
        (StairN, Wall, Empty | Door, Empty | Door | esw!()) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::WallCorner,
        },
        (StairN, Wall, Empty | Door, Wall) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::WallCorner,
        },
        (StairN, Wall, Empty | Door, StairN) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::WallCorner,
        },
        (StairN, Wall, Wall | stairs!(), Empty | Door | esw!()) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::WallHorizontal,
        },
        (StairN, Wall, Wall | stairs!(), Wall) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::WallHorizontal,
        },
        (StairN, Wall, Wall | stairs!(), StairN) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::WallHorizontal,
        },
        (StairN, StairN, Empty | Door | esw!(), Empty | Door | esw!()) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairN,
        },
        (StairN, StairN, Empty | Door | esw!(), Wall) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairN,
        },
        (StairN, StairN, Empty | Door | esw!(), StairN) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairN,
        },
        (StairN, StairN, Wall, Empty | Door | esw!()) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairNWall,
        },
        (StairN, StairN, Wall, Wall) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairNWall,
        },
        (StairN, StairN, Wall, StairN) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairNWall,
        },
        (StairN, StairN, StairN, Empty | Door | esw!()) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairNDouble,
        },
        (StairN, StairN, StairN, Wall) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairNDouble,
        },
        (StairN, StairN, StairN, StairN) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairNDouble,
        },
        (StairN, StairS, Wall, Empty | Door | esw!()) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairSWall,
        },
        (StairN, StairS, Wall, Wall) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairSWall,
        },
        (StairN, StairS, Wall, StairN) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairSWall,
        },
        (StairN, StairE, _, Empty | Door | esw!()) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairE,
        },
        (StairN, StairE, _, Wall) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairE,
        },
        (StairN, StairE, _, StairN) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairE,
        },
        (StairS, Empty | Door | StairW, _, Empty | Door | ewn!())
        | (StairS, StairS, Empty | Door | stairs!(), Empty | Door | ewn!()) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::Empty,
        },
        (StairS, Empty | Door | StairW, _, Wall)
        | (StairS, StairS, Empty | Door | stairs!(), Wall) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::Empty,
        },
        (StairS, Empty | Door | StairW, _, StairS)
        | (StairS, StairS, Empty | Door | stairs!(), StairS) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::Empty,
        },
        (StairS, Wall, Empty | Door, Empty | Door | ewn!()) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::WallCorner,
        },
        (StairS, Wall, Empty | Door, Wall) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::WallCorner,
        },
        (StairS, Wall, Empty | Door, StairS) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::WallCorner,
        },
        (StairS, Wall, Wall | stairs!(), Empty | Door | ewn!()) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::WallHorizontal,
        },
        (StairS, Wall, Wall | stairs!(), Wall) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::WallHorizontal,
        },
        (StairS, Wall, Wall | stairs!(), StairS) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::WallHorizontal,
        },
        (StairS, StairN, Empty | Door | esw!(), Empty | Door | ewn!()) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairN,
        },
        (StairS, StairN, Empty | Door | esw!(), Wall) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairN,
        },
        (StairS, StairN, Empty | Door | esw!(), StairS) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairN,
        },
        (StairS, StairN, Wall, Empty | Door | ewn!()) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairNWall,
        },
        (StairS, StairN, Wall, Wall) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairNWall,
        },
        (StairS, StairN, Wall, StairS) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairNWall,
        },
        (StairS, StairN, StairN, Empty | Door | ewn!()) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairNDouble,
        },
        (StairS, StairN, StairN, Wall) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairNDouble,
        },
        (StairS, StairN, StairN, StairS) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairNDouble,
        },
        (StairS, StairS, Wall, Empty | Door | ewn!()) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairSWall,
        },
        (StairS, StairS, Wall, Wall) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairSWall,
        },
        (StairS, StairS, Wall, StairS) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairSWall,
        },
        (StairS, StairE, _, Empty | Door | ewn!()) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::StairE,
        },
        (StairS, StairE, _, Wall) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairE,
        },
        (StairS, StairE, _, StairS) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairE,
        },
        (StairE, Empty | Door | StairW, _, _) | (StairE, StairS, Empty | Door | stairs!(), _) => {
            Sprite::StairE {
                south: SouthDecoration::Empty,
            }
        }
        (StairE, Wall, Empty | Door, _) => Sprite::StairE {
            south: SouthDecoration::WallCorner,
        },
        (StairE, Wall, Wall | stairs!(), _) => Sprite::StairE {
            south: SouthDecoration::WallHorizontal,
        },
        (StairE, StairN, Empty | Door | esw!(), _) => Sprite::StairE {
            south: SouthDecoration::StairN,
        },
        (StairE, StairN, Wall, _) => Sprite::StairE {
            south: SouthDecoration::StairNWall,
        },
        (StairE, StairN, StairN, _) => Sprite::StairE {
            south: SouthDecoration::StairNDouble,
        },
        (StairE, StairS, Wall, _) => Sprite::StairE {
            south: SouthDecoration::StairSWall,
        },
        (StairE, StairE, _, _) => Sprite::StairE {
            south: SouthDecoration::StairE,
        },
        (StairW, Empty | Door, _, _) | (StairW, StairS, Empty | Door | stairs!(), _) => {
            Sprite::StairW {
                south: SouthDecoration::Empty,
            }
        }
        (StairW, Wall, Empty | Door, _) => Sprite::StairW {
            south: SouthDecoration::WallCorner,
        },
        (StairW, Wall, Wall | stairs!(), _) => Sprite::StairW {
            south: SouthDecoration::WallHorizontal,
        },
        (StairW, StairN, Empty | Door | esw!(), _) => Sprite::StairW {
            south: SouthDecoration::StairN,
        },
        (StairW, StairN, Wall, _) => Sprite::StairW {
            south: SouthDecoration::StairNWall,
        },
        (StairW, StairN, StairN, _) => Sprite::StairW {
            south: SouthDecoration::StairNDouble,
        },
        (StairW, StairS, Wall, _) => Sprite::StairW {
            south: SouthDecoration::StairSWall,
        },
        (StairW, StairE, _, _) => Sprite::StairW {
            south: SouthDecoration::StairE,
        },
        (StairW, StairW, _, _) => Sprite::StairWDouble,
    }
}
