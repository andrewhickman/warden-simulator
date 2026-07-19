use std::{collections::HashSet, fmt::Display};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum WallKind {
    Corner,
    Horizontal,
    Vertical,
    InverseCorner,
    Full,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum SouthWallKind {
    Horizontal,
    Corner,
    HorizontalDoor,
    CornerDoor,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum StairKind {
    HorizontalUpper,
    HorizontalLower,
    VerticalUpper,
    VerticalLower,
}

#[derive(Default, Hash, Eq, PartialEq, Clone, Copy, Debug)]
struct WallSprite {
    wall_kind: Option<WallKind>,
    // east_door: bool,
    // south_door: bool,
    stair: Option<StairKind>,
    south_wall: Option<SouthWallKind>,
    south_stair: Option<StairKind>,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum TileKind {
    Empty,
    Wall,
    Door,
    StairNorth,
    StairEast,
    StairSouth,
    StairWest,
}

fn main() {
    let mut variants = HashSet::new();
    // let mut ordered = Vec::new();
    let mut index = 0;

    for center in TileKind::values() {
        for east in TileKind::values() {
            for south_east in TileKind::values() {
                for south in TileKind::values() {
                    // for south_west in TileKind::values() {
                    //     for west in TileKind::values() {
                    let right_sprite = WallSprite::new(center, east, south, south_east);
                    // let left_sprite = WallSprite::new(center, west, south, south_west);

                    if variants.insert(right_sprite) {
                        index += 1;
                        println!(
                            "{index}: Found new wall sprite: {}, center: {:?}, south: {:?}, east: {:?}, south_east: {:?}",
                            right_sprite, center, south, east, south_east
                        );
                        // ordered.push((right_sprite, left_sprite));
                    }
                    //     }
                    // }
                }
            }
        }
    }

    println!("Found {} unique wall sprites", variants.len());
    // for sprite in ordered {
    //     println!("{}", sprite);
    // }
}

impl WallSprite {
    pub fn new(center: TileKind, east: TileKind, south: TileKind, south_west: TileKind) -> Self {
        let wall_kind = match (
            matches!(center, TileKind::Wall),
            // center.solid(),
            east.solid(),
            south.solid(),
            south_west.solid(),
        ) {
            (true, true, true, true) => Some(WallKind::Full),
            (true, true, true, false) => Some(WallKind::InverseCorner),
            (true, false, false, _) => Some(WallKind::Corner),
            (true, false, true, _) => Some(WallKind::Vertical),
            (true, true, false, _) => Some(WallKind::Horizontal),
            (false, _, _, _) => None,
        };

        let south_wall = match (center, south, south_west.solid()) {
            (TileKind::Empty, TileKind::Wall, false) => Some(SouthWallKind::Corner),
            (TileKind::Door, TileKind::Wall, false) => Some(SouthWallKind::CornerDoor),
            (TileKind::Empty, TileKind::Wall, true) => Some(SouthWallKind::Horizontal),
            (TileKind::Door, TileKind::Wall, true) => Some(SouthWallKind::HorizontalDoor),
            _ => None,
        };

        let east_door = center.solid() && matches!(east, TileKind::Door);
        let south_door = center.solid() && matches!(south, TileKind::Door);

        let stair = match center {
            TileKind::Empty | TileKind::Wall | TileKind::Door => None,
            TileKind::StairNorth => Some(StairKind::VerticalLower),
            TileKind::StairEast => Some(StairKind::HorizontalLower),
            TileKind::StairSouth => Some(StairKind::VerticalUpper),
            TileKind::StairWest => Some(StairKind::HorizontalUpper),
        };

        let south_stair = match south {
            TileKind::Empty | TileKind::Wall | TileKind::Door => None,
            TileKind::StairNorth => {
                if south_west.solid() {
                    Some(StairKind::VerticalUpper)
                } else {
                    None
                }
            }
            TileKind::StairEast => Some(StairKind::HorizontalUpper),
            TileKind::StairSouth => Some(StairKind::VerticalLower),
            TileKind::StairWest => Some(StairKind::HorizontalLower),
        };

        WallSprite {
            wall_kind,
            south_wall,
            // east_door,
            // south_door,
            stair,
            south_stair,
        }
    }
}

impl TileKind {
    fn values() -> [TileKind; 7] {
        [
            TileKind::Empty,
            TileKind::Wall,
            TileKind::Door,
            TileKind::StairNorth,
            TileKind::StairEast,
            TileKind::StairSouth,
            TileKind::StairWest,
        ]
    }

    fn solid(&self) -> bool {
        matches!(
            self,
            TileKind::Wall
                | TileKind::StairEast
                | TileKind::StairNorth
                | TileKind::StairSouth
                | TileKind::StairWest
        )
    }
}

impl Display for WallSprite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(wall_kind) = self.wall_kind {
            write!(f, "Wall")?;
            match wall_kind {
                WallKind::Corner => write!(f, "Corner")?,
                WallKind::Horizontal => write!(f, "Horizontal")?,
                WallKind::Vertical => write!(f, "Vertical")?,
                WallKind::InverseCorner => write!(f, "InverseCorner")?,
                WallKind::Full => write!(f, "Full")?,
            }
        }

        if let Some(center_stair) = self.stair {
            write!(f, "Stair")?;
            match center_stair {
                StairKind::HorizontalUpper => write!(f, "HorizontalUpper")?,
                StairKind::HorizontalLower => write!(f, "HorizontalLower")?,
                StairKind::VerticalUpper => write!(f, "VerticalUpper")?,
                StairKind::VerticalLower => write!(f, "VerticalLower")?,
            }
        }

        if self.wall_kind.is_none() && self.stair.is_none() {
            write!(f, "Empty")?;
        }

        if let Some(south_wall) = self.south_wall {
            match south_wall {
                SouthWallKind::Horizontal => write!(f, "WallTopHorizontal")?,
                SouthWallKind::Corner => write!(f, "WallTopCorner")?,
                SouthWallKind::HorizontalDoor => write!(f, "WallTopHorizontalDoor")?,
                SouthWallKind::CornerDoor => write!(f, "WallTopCornerDoor")?,
            }
        }

        if let Some(south_stair) = self.south_stair {
            write!(f, "StairTop")?;
            match south_stair {
                StairKind::HorizontalUpper => write!(f, "HorizontalUpper")?,
                StairKind::HorizontalLower => write!(f, "HorizontalLower")?,
                StairKind::VerticalUpper => write!(f, "VerticalUpper")?,
                StairKind::VerticalLower => write!(f, "VerticalLower")?,
            }
        }

        // match (self.east_door, self.south_door) {
        //     (true, true) => write!(f, "DoorEastSouth")?,
        //     (true, false) => write!(f, "DoorEast")?,
        //     (false, true) => write!(f, "DoorSouth")?,
        //     (false, false) => {}
        // }

        Ok(())
    }
}
