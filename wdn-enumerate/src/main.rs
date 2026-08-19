#![warn(clippy::match_same_arms)]

use std::collections::HashSet;

use wdn_enumerate::*;

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

    let mut mids = mids.into_iter().collect::<Vec<_>>();
    mids.sort_unstable();

    println!("Unique mids: {}", mids.len());
    for (i, mid) in mids.iter().enumerate() {
        // assert_eq!(mid.id(), i as u16);
        println!("  {mid} => {}", i);
    }

    let mut tops = tops.into_iter().collect::<Vec<_>>();
    tops.sort_unstable();

    println!("Unique tops: {}", tops.len());
    for (i, top) in tops.iter().enumerate() {
        // if matches!(
        //     top,
        //     TopSprite::Empty {
        //         south_east: SouthEastTopDecoration::None
        //     }
        // ) {
        //     assert_eq!(top.id(), 0);
        // } else {
        //     assert_eq!(
        //         top.id(),
        //         i as u16 + mids.len() as u16 - 1,
        //         "top: {top:?}, i: {i}"
        //     );
        // }

        println!("  {top} => {}", i + mids.len() - 1);
    }

    println!("Unique sprites: {}", sprites.len());

    println!(
        "sprites with bleed risk: {}",
        sprites.iter().filter(|s| s.bleeds()).count()
    );
    for sprite in sprites.iter().filter(|s| s.bleeds()) {
        println!("  {sprite:?}");
    }
}
