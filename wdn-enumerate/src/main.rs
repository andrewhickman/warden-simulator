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
        north: NorthStairDecoration,
        south: SouthDecoration,
    },
    StairW {
        north: NorthStairDecoration,
        south: SouthDecoration,
    },
    StairWDouble {
        north: NorthStairDecoration,
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
        TileKind::Empty | TileKind::Door | TileKind::StairE | TileKind::StairS | TileKind::StairW
    };
}

macro_rules! ewn {
    () => {
        TileKind::Empty | TileKind::Door | TileKind::StairN | TileKind::StairE | TileKind::StairW
    };
}

macro_rules! swn {
    () => {
        TileKind::Empty | TileKind::Door | TileKind::StairS | TileKind::StairW | TileKind::StairN
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
                    for north in TileKind::values() {
                        let tile = tile_sprite(center, south, south_east, east, north);

                        if variants.insert(tile) {
                            ordered.push(tile);
                        }
                    }
                }
            }
        }
    }

    // println!("Ordered tiles: {:?}", ordered);

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
            for north in NorthStairDecoration::values() {
                set.insert(Sprite::StairE { south, north });
            }
            for north in NorthStairDecoration::values() {
                set.insert(Sprite::StairW { south, north });
            }
        }
        for north in NorthStairDecoration::values() {
            set.insert(Sprite::StairWDouble { north });
        }
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

    let mut count = 0;
    for tile in ordered {
        println!("    {},", tile);

        if count == 15 {
            println!();
            count = 0;
        } else {
            count += 1;
        }
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
        (Empty | Door, Empty | Door | StairW, _, _, _)
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
        (Wall, Door, _, Empty, _) | (Wall, StairS, Empty | Door | stairs!(), Door, _) => {
            Sprite::Wall(WallSprite::Corner {
                east: EastWallDecoration::Door,
                south: SouthWallDecoration::Empty,
            })
        }
        (Wall, Empty, _, Door, _) => Sprite::Wall(WallSprite::Corner {
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
        (StairN, Empty | Door | StairW, _, esw!(), _)
        | (StairN, StairS, Empty | Door | stairs!(), esw!(), _) => Sprite::StairN {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::Empty,
        },
        (StairN, Empty | Door | StairW, _, Wall, _)
        | (StairN, StairS, Empty | Door | stairs!(), Wall, _) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::Empty,
        },
        (StairN, Empty | Door | StairW, _, StairN, _)
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
        (StairN, StairE, _, Wall, _) => Sprite::StairN {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairE,
        },
        (StairN, StairE, _, StairN, _) => Sprite::StairN {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairE,
        },
        (StairS, Empty | Door | StairW, _, ewn!(), _)
        | (StairS, StairS, Empty | Door | stairs!(), ewn!(), _) => Sprite::StairS {
            east: EastStairDecoration::Empty,
            south: SouthDecoration::Empty,
        },
        (StairS, Empty | Door | StairW, _, Wall, _)
        | (StairS, StairS, Empty | Door | stairs!(), Wall, _) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::Empty,
        },
        (StairS, Empty | Door | StairW, _, StairS, _)
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
        (StairS, StairE, _, Wall, _) => Sprite::StairS {
            east: EastStairDecoration::Wall,
            south: SouthDecoration::StairE,
        },
        (StairS, StairE, _, StairS, _) => Sprite::StairS {
            east: EastStairDecoration::Stair,
            south: SouthDecoration::StairE,
        },
        (StairE, Empty | Door | StairW, _, _, swn!())
        | (StairE, StairS, Empty | Door | stairs!(), _, swn!()) => Sprite::StairE {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::Empty,
        },
        (StairE, Empty | Door | StairW, _, _, Wall)
        | (StairE, StairS, Empty | Door | stairs!(), _, Wall) => Sprite::StairE {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::Empty,
        },
        (StairE, Empty | Door | StairW, _, _, StairE)
        | (StairE, StairS, Empty | Door | stairs!(), _, StairE) => Sprite::StairE {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::Empty,
        },
        (StairE, Wall, Empty | Door, _, swn!()) => Sprite::StairE {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::WallCorner,
        },
        (StairE, Wall, Empty | Door, _, Wall) => Sprite::StairE {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::WallCorner,
        },
        (StairE, Wall, Empty | Door, _, StairE) => Sprite::StairE {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::WallCorner,
        },
        (StairE, Wall, Wall | stairs!(), _, swn!()) => Sprite::StairE {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::WallHorizontal,
        },
        (StairE, Wall, Wall | stairs!(), _, Wall) => Sprite::StairE {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::WallHorizontal,
        },
        (StairE, Wall, Wall | stairs!(), _, StairE) => Sprite::StairE {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::WallHorizontal,
        },
        (StairE, StairN, esw!(), _, swn!()) => Sprite::StairE {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::StairN,
        },
        (StairE, StairN, esw!(), _, Wall) => Sprite::StairE {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::StairN,
        },
        (StairE, StairN, esw!(), _, StairE) => Sprite::StairE {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::StairN,
        },
        (StairE, StairN, Wall, _, swn!()) => Sprite::StairE {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::StairNWall,
        },
        (StairE, StairN, Wall, _, Wall) => Sprite::StairE {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::StairNWall,
        },
        (StairE, StairN, Wall, _, StairE) => Sprite::StairE {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::StairNWall,
        },
        (StairE, StairN, StairN, _, swn!()) => Sprite::StairE {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::StairNDouble,
        },
        (StairE, StairN, StairN, _, Wall) => Sprite::StairE {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::StairNDouble,
        },
        (StairE, StairN, StairN, _, StairE) => Sprite::StairE {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::StairNDouble,
        },
        (StairE, StairS, Wall, _, swn!()) => Sprite::StairE {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::StairSWall,
        },
        (StairE, StairS, Wall, _, Wall) => Sprite::StairE {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::StairSWall,
        },
        (StairE, StairS, Wall, _, StairE) => Sprite::StairE {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::StairSWall,
        },
        (StairE, StairE, _, _, swn!()) => Sprite::StairE {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::StairE,
        },
        (StairE, StairE, _, _, Wall) => Sprite::StairE {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::StairE,
        },
        (StairE, StairE, _, _, StairE) => Sprite::StairE {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::StairE,
        },
        (StairW, Empty | Door, _, _, esn!())
        | (StairW, StairS, Empty | Door | stairs!(), _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::Empty,
        },
        (StairW, Empty | Door, _, _, Wall)
        | (StairW, StairS, Empty | Door | stairs!(), _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::Empty,
        },
        (StairW, Empty | Door, _, _, StairW)
        | (StairW, StairS, Empty | Door | stairs!(), _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::Empty,
        },
        (StairW, Wall, Empty | Door, _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::WallCorner,
        },
        (StairW, Wall, Empty | Door, _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::WallCorner,
        },
        (StairW, Wall, Empty | Door, _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::WallCorner,
        },
        (StairW, Wall, Wall | stairs!(), _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::WallHorizontal,
        },
        (StairW, Wall, Wall | stairs!(), _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::WallHorizontal,
        },
        (StairW, Wall, Wall | stairs!(), _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::WallHorizontal,
        },
        (StairW, StairN, esw!(), _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::StairN,
        },
        (StairW, StairN, esw!(), _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::StairN,
        },
        (StairW, StairN, esw!(), _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::StairN,
        },
        (StairW, StairN, Wall, _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::StairNWall,
        },
        (StairW, StairN, Wall, _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::StairNWall,
        },
        (StairW, StairN, Wall, _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::StairNWall,
        },
        (StairW, StairN, StairN, _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::StairNDouble,
        },
        (StairW, StairN, StairN, _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::StairNDouble,
        },
        (StairW, StairN, StairN, _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::StairNDouble,
        },
        (StairW, StairS, Wall, _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::StairSWall,
        },
        (StairW, StairS, Wall, _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::StairSWall,
        },
        (StairW, StairS, Wall, _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::StairSWall,
        },
        (StairW, StairE, _, _, esn!()) => Sprite::StairW {
            north: NorthStairDecoration::Empty,
            south: SouthDecoration::StairE,
        },
        (StairW, StairE, _, _, Wall) => Sprite::StairW {
            north: NorthStairDecoration::Wall,
            south: SouthDecoration::StairE,
        },
        (StairW, StairE, _, _, StairW) => Sprite::StairW {
            north: NorthStairDecoration::Stair,
            south: SouthDecoration::StairE,
        },
        (StairW, StairW, _, _, esn!()) => Sprite::StairWDouble {
            north: NorthStairDecoration::Empty,
        },
        (StairW, StairW, _, _, Wall) => Sprite::StairWDouble {
            north: NorthStairDecoration::Wall,
        },
        (StairW, StairW, _, _, StairW) => Sprite::StairWDouble {
            north: NorthStairDecoration::Stair,
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
            Sprite::StairE { south, north } => write!(f, "StairE{south}{north}"),
            Sprite::StairW { south, north } => write!(f, "StairW{south}{north}"),
            Sprite::StairWDouble { north } => write!(f, "StairW_SouthStairW{north}"),
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
            SouthDecoration::StairN => write!(f, "_SouthStairN"),
            SouthDecoration::StairNDouble => write!(f, "_SouthStairNDouble"),
            SouthDecoration::StairNWall => write!(f, "_SouthStairNWall"),
            SouthDecoration::StairSWall => write!(f, "_SouthStairSWall"),
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
