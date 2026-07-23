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

enum TileKind2 {
    Wall(WallKind),
    Door,
}

enum WallKind {
    Corner {
        east: EastWallDecoration,
        south: SouthWallDecoration,
    },
    Horizontal {
        south: SouthWallDecoration,
    },
    Vertical {
        east: EastWallDecoration,
    },
    InverseCorner,
    Full,
}

enum SouthWallDecoration {
    Corner,
    Horizontal,
}

enum EastWallDecoration {
    Door,
    StairN,
    SouthS,
}

enum SouthWallDecoration {
    Door,
    Stair,
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
                    let tile = tile_kind(center, south, south_east, east);
                    let tile = tile.replace("_", "");

                    if variants.insert(tile.clone()) {
                        ordered.push(tile);
                    }
                }
            }
        }
    }

    // println!("Ordered tiles: {:?}", ordered);

    let mut count = 0;
    for tile in ordered {
        println!("    {},", tile);

        if count == 14 {
            println!();
            count = 0;
        } else {
            count += 1;
        }
    }

    println!("Found {} unique wall sprites", variants.len());
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

fn tile_kind(
    center: TileKind,
    south: TileKind,
    south_east: TileKind,
    east: TileKind,
) -> &'static str {
    use TileKind::*;

    match (center, south, south_east, east) {
        (Empty | Door, Empty | Door | StairW | StairS, _, _) => "Empty",
        (Empty, Wall, Wall, _) | (Empty, Wall, stairs!(), Wall) => "Empty_WallTopHorizontal",
        (Empty, Wall, stairs!(), Empty | Door | stairs!()) | (Empty, Wall, Empty | Door, _) => {
            "Empty_WallTopCorner"
        }
        (Empty | Door, StairN, _, _) => "Empty_StairTopVerticalUpper",
        (Empty | Door, StairE, _, _) => "Empty_StairTopHorizontalUpper",
        (Door, Wall, Wall, _) | (Door, Wall, stairs!(), Wall) => "Door_WallTopHorizontal",
        (Door, Wall, stairs!(), Empty | Door | stairs!()) | (Door, Wall, Empty | Door, _) => {
            "Door_WallTopCorner_DoorSouth"
        }
        (Wall, Empty | StairS, _, Empty) => "WallCorner",
        (Wall, Door, _, Empty) => "WallCorner_DoorSouth",
        (Wall, Empty | StairS, _, Door) => "WallCorner_DoorEast",
        (Wall, Door, _, Door) => "WallCorner_DoorEastSouth",
        (Wall, Empty | StairS, _, Wall | stairs!()) => "WallHorizontal",
        (Wall, Door, _, Wall | stairs!()) => "WallHorizontal_DoorSouth",
        (Wall, Wall, Empty | Door, Empty) => "WallVertical",
        (Wall, Wall, Empty | Door, Wall | stairs!()) => "WallInverseCorner",
        (Wall, Wall, Empty | Door, Door) => "WallVertical_DoorEast",
        (Wall, Wall, Wall | stairs!(), Empty) => "WallVertical_WallTop",
        (Wall, Wall, Wall | stairs!(), Door) => "WallVertical_WallTop_DoorEast",
        (Wall, Wall, Wall | stairs!(), Wall | stairs!()) => "WallFull",
        (Wall, StairN, Empty | Door | esw!(), Empty) => "WallCorner_StairTopVerticalUpper",
        (Wall, StairN, Empty | Door | esw!(), Door) => "WallCorner_StairTopVerticalUpper_DoorEast",
        (Wall, StairN, Empty | Door | esw!(), Wall | stairs!()) => {
            "WallHorizontal_StairTopVerticalUpper"
        }
        (Wall, StairN, Wall, Empty) => "WallHorizontal_StairTopVerticalUpper_WallTop",
        (Wall, StairN, Wall, Door) => "WallHorizontal_StairTopVerticalUpper_WallTop_DoorEast",
        (Wall, StairN, Wall, Wall | stairs!()) => "WallFull_StairTopVerticalUpper",
        (Wall, StairN, StairN, Empty) => "WallCorner_DoubleStairTopVerticalUpper",
        (Wall, StairN, StairN, Door) => "WallCorner_DoubleStairTopVerticalUpper_DoorEast",
        (Wall, StairN, StairN, Wall | stairs!()) => {
            "WallHorizontal_DoubleStairTopVerticalUpper_DoorEast"
        }
        (Wall, StairE, Empty | Door | stairs!(), Empty) => "WallCorner_StairTopHorizontalUpper",
        (Wall, StairE, Empty | Door | stairs!(), Door) => {
            "WallCorner_StairTopHorizontalUpper_DoorEast"
        }
        (Wall, StairE, Empty | Door | stairs!(), Wall | stairs!()) => {
            "WallHorizontal_StairTopHorizontalUpper"
        }
        (Wall, StairE, Wall, Empty) => "WallHorizontal_StairTopHorizontalUpper_WallTop",
        (Wall, StairE, Wall, Door) => "WallHorizontal_StairTopHorizontalUpper_WallTop_DoorEast",
        (Wall, StairE, Wall, Wall | stairs!()) => "WallFull_StairTopHorizontalUpper",
        (Wall, StairW, Empty | Door | stairs!(), Empty) => "WallCorner_StairTopHorizontalLower",
        (Wall, StairW, Empty | Door | stairs!(), Door) => {
            "WallCorner_StairTopHorizontalLower_DoorEast"
        }
        (Wall, StairW, Empty | Door | stairs!(), Wall | stairs!()) => {
            "WallHorizontal_StairTopHorizontalLower"
        }
        (Wall, StairW, Wall, Empty) => "WallHorizontal_StairTopHorizontalLower_WallTop",
        (Wall, StairW, Wall, Door) => "WallHorizontal_StairTopHorizontalLower_WallTop_DoorEast",
        (Wall, StairW, Wall, Wall | stairs!()) => "WallFull_StairTopHorizontalLower",
        (StairN, Empty | Door | StairS, _, Empty | Door | esw!()) => "StairVerticalLower",
        (StairN, Empty | Door | StairS, _, Wall) => "StairVerticalLower_WallEast",
        (StairN, Empty | Door | StairS, _, StairN) => "DoubleStairVerticalLower",
        (StairN, Wall, Empty | Door, Empty | Door | esw!()) => "StairVerticalLower_WallTopCorner",
        (StairN, Wall, Empty | Door, Wall) => "StairVerticalLower_WallEastSouthCorner",
        (StairN, Wall, Empty | Door, StairN) => "DoubleStairVerticalLower_WallTopCorner",
        (StairN, Wall, Wall | stairs!(), Empty | Door | esw!()) => {
            "StairVerticalLower_WallTopHorizontal"
        }
        (StairN, Wall, Wall | stairs!(), Wall) => "StairVerticalLower_WallFull",
        (StairN, Wall, Wall | stairs!(), StairN) => "DoubleStairVerticalLower_WallTopHorizontal",
        (StairN, StairN, Empty | Door | esw!(), Empty | Door | esw!()) => {
            "StairVerticalLower_StairTopVerticalUpper"
        }
        (StairN, StairN, Empty | Door | esw!(), Wall) => {
            "StairVerticalLower_StairTopVerticalUpper_WallEast"
        }
        (StairN, StairN, Empty | Door | esw!(), StairN) => {
            "DoubleStairVerticalLower_StairTopVerticalUpper"
        }
        (StairN, StairN, Wall, Empty | Door | esw!()) => {
            "StairVerticalLower_StairTopVerticalUpper_WallTop"
        }
        (StairN, StairN, Wall, Wall) => "StairVerticalLower_StairTopVerticalUpper_WallEastSouth",
        (StairN, StairN, Wall, StairN) => "DoubleStairVerticalLower_StairTopVerticalUpper_WallTop",
        (StairN, StairN, StairN, Empty | Door | esw!()) => {
            "StairVerticalLower_DoubleStairTopVerticalUpper"
        }
        (StairN, StairN, StairN, Wall) => "StairVerticalLower_DoubleStairTopVerticalUpper_WallEast",
        (StairN, StairN, StairN, StairN) => "DoubleStairVerticalLower_DoubleStairTopVerticalUpper",
        (StairN, StairE, Empty | Door | stairs!(), Empty | Door | esw!()) => {
            "StairVerticalLower_StairTopHorizontalUpper"
        }
        (StairN, StairE, Empty | Door | stairs!(), Wall) => {
            "StairVerticalLower_StairTopHorizontalUpper_WallEast"
        }
        (StairN, StairE, Empty | Door | stairs!(), StairN) => {
            "DoubleStairVerticalLower_StairTopHorizontalUpper"
        }
        (StairN, StairE, Wall, Empty | Door | esw!()) => {
            "StairVerticalLower_StairTopHorizontalUpper_WallTop"
        }
        (StairN, StairE, Wall, Wall) => "StairVerticalLower_StairTopHorizontalUpper_WallEastSouth",
        (StairN, StairE, Wall, StairN) => {
            "DoubleStairVerticalLower_StairTopHorizontalUpper_WallTop"
        }
        (StairN, StairW, Empty | Door | stairs!(), Empty | Door | esw!()) => {
            "StairVerticalLower_StairTopHorizontalLower"
        }
        (StairN, StairW, Empty | Door | stairs!(), Wall) => {
            "StairVerticalLower_StairTopHorizontalLower_WallEast"
        }
        (StairN, StairW, Empty | Door | stairs!(), StairN) => {
            "DoubleStairVerticalLower_StairTopHorizontalLower"
        }
        (StairN, StairW, Wall, Empty | Door | esw!()) => {
            "StairVerticalLower_StairTopHorizontalLower_WallTop"
        }
        (StairN, StairW, Wall, Wall) => "StairVerticalLower_StairTopHorizontalLower_WallEastSouth",
        (StairN, StairW, Wall, StairN) => {
            "DoubleStairVerticalLower_StairTopHorizontalLower_WallTop"
        }
        (StairS, Empty | Door | StairS, _, Empty | Door | ewn!()) => "StairVerticalUpper",
        (StairS, Empty | Door | StairS, _, Wall) => "StairVerticalUpper_WallEast",
        (StairS, Empty | Door | StairS, _, StairS) => "DoubleStairVerticalUpper",
        (StairS, Wall, Empty | Door, Empty | Door | ewn!()) => "StairVerticalUpper_WallTopCorner",
        (StairS, Wall, Empty | Door, Wall) => "StairVerticalUpper_WallEastSouthCorner",
        (StairS, Wall, Empty | Door, StairS) => "DoubleStairVerticalUpper_WallTopCorner",
        (StairS, Wall, Wall | stairs!(), Empty | Door | ewn!()) => {
            "StairVerticalUpper_WallTopHorizontal"
        }
        (StairS, Wall, Wall | stairs!(), Wall) => "StairVerticalUpper_WallFull",
        (StairS, Wall, Wall | stairs!(), StairS) => "DoubleStairVerticalUpper_WallTopHorizontal",
        (StairS, StairN, Empty | Door | esw!(), Empty | Door | ewn!()) => {
            "StairVerticalUpper_StairTopVerticalUpper"
        }
        (StairS, StairN, Empty | Door | esw!(), Wall) => {
            "StairVerticalUpper_StairTopVerticalUpper_WallEast"
        }
        (StairS, StairN, Empty | Door | esw!(), StairS) => {
            "DoubleStairVerticalUpper_StairTopVerticalUpper"
        }
        (StairS, StairN, Wall, Empty | Door | ewn!()) => {
            "StairVerticalUpper_StairTopVerticalUpper_WallTop"
        }
        (StairS, StairN, Wall, Wall) => "StairVerticalUpper_StairTopVerticalUpper_WallEastSouth",
        (StairS, StairN, Wall, StairS) => "DoubleStairVerticalUpper_StairTopVerticalUpper_WallTop",
        (StairS, StairN, StairN, Empty | Door | ewn!()) => {
            "StairVerticalUpper_DoubleStairTopVerticalUpper"
        }
        (StairS, StairN, StairN, Wall) => "StairVerticalUpper_DoubleStairTopVerticalUpper_WallEast",
        (StairS, StairN, StairN, StairS) => "DoubleStairVerticalUpper_DoubleStairTopVerticalUpper",
        (StairS, StairE, Empty | Door | stairs!(), Empty | Door | ewn!()) => {
            "StairVerticalUpper_StairTopHorizontalUpper"
        }
        (StairS, StairE, Empty | Door | stairs!(), Wall) => {
            "StairVerticalUpper_StairTopHorizontalUpper_WallEast"
        }
        (StairS, StairE, Empty | Door | stairs!(), StairS) => {
            "DoubleStairVerticalUpper_StairTopHorizontalUpper"
        }
        (StairS, StairE, Wall, Empty | Door | ewn!()) => {
            "StairVerticalUpper_StairTopHorizontalUpper_WallTop"
        }
        (StairS, StairE, Wall, Wall) => "StairVerticalUpper_StairTopHorizontalUpper_WallEastSouth",
        (StairS, StairE, Wall, StairS) => {
            "DoubleStairVerticalUpper_StairTopHorizontalUpper_WallTop"
        }
        (StairS, StairW, Empty | Door | stairs!(), Empty | Door | ewn!()) => {
            "StairVerticalUpper_StairTopHorizontalLower"
        }
        (StairS, StairW, Empty | Door | stairs!(), Wall) => {
            "StairVerticalUpper_StairTopHorizontalLower_WallEast"
        }
        (StairS, StairW, Empty | Door | stairs!(), StairS) => {
            "DoubleStairVerticalUpper_StairTopHorizontalLower"
        }
        (StairS, StairW, Wall, Empty | Door | ewn!()) => {
            "StairVerticalUpper_StairTopHorizontalLower_WallTop"
        }
        (StairS, StairW, Wall, Wall) => "StairVerticalUpper_StairTopHorizontalLower_WallEastSouth",
        (StairS, StairW, Wall, StairS) => {
            "DoubleStairVerticalUpper_StairTopHorizontalLower_WallTop"
        }
        (StairE, Empty | Door | StairS, _, _) => "StairHorizontalUpper",
        (StairE, Wall, Empty | Door, _) => "StairHorizontalUpper_WallTopCorner",
        (StairE, Wall, Wall | stairs!(), _) => "StairHorizontalUpper_WallTopHorizontal",
        (StairE, StairN, Empty | Door | esw!(), _) => "StairHorizontalUpper_StairTopVerticalUpper",
        (StairE, StairN, Wall, _) => "StairHorizontalUpper_StairTopVerticalUpper_WallTop",
        (StairE, StairN, StairN, _) => "StairHorizontalUpper_DoubleStairTopVerticalUpper",
        (StairE, StairE, _, _) => "DoubleStairHorizontalUpper",
        (StairE, StairW, _, _) => "StairHorizontalUpper_StairTopHorizontalLower",
        (StairW, Empty | Door, _, _) => "StairHorizontalLower",
        (StairW, Wall, Empty | Door, _) => "StairHorizontalLower_WallTopCorner",
        (StairW, Wall, Wall | stairs!(), _) => "StairHorizontalLower_WallTopHorizontal",
        (StairW, StairN, Empty | Door | esw!(), _) => "StairHorizontalLower_StairTopVerticalUpper",
        (StairW, StairN, Wall, _) => "StairHorizontalLower_StairTopVerticalUpper_WallTop",
        (StairW, StairN, StairN, _) => "StairHorizontalLower_DoubleStairTopVerticalUpper",
        (StairW, StairS, Empty | Door | ewn!(), _) => "StairHorizontalLower_StairTopVerticalLower",
        (StairW, StairS, Wall, _) => "StairHorizontalLower_StairTopVerticalLower_WallTop",
        (StairW, StairS, StairS, _) => "StairHorizontalLower_DoubleStairTopVerticalLower",
        (StairW, StairE, _, _) => "StairHorizontalLower_StairTopHorizontalUpper",
        (StairW, StairW, _, _) => "DoubleStairHorizontalLower",
    }
}
