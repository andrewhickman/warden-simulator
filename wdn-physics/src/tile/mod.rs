pub mod index;
pub mod map;
#[cfg(test)]
mod tests;

use std::fmt;

use bevy::{
    ecs::{lifecycle::HookContext, relationship::Relationship, world::DeferredWorld},
    prelude::*,
};
use parking_lot::Mutex;

use crate::tile::index::{TileChanged, TileIndex};

pub struct TilePlugin;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Component)]
#[component(on_remove = Tile::on_remove)]
pub struct Tile {
    layer: Entity,
    position: IVec2,
}

pub fn update_tile(
    mut entities: Query<
        (Entity, &ChildOf, &Transform, &mut Tile),
        Or<(Changed<Transform>, Changed<ChildOf>)>,
    >,
    writer: MessageWriter<TileChanged>,
) {
    let writer = Mutex::new(writer);

    entities
        .par_iter_mut()
        .for_each(|(id, parent, transform, mut tile)| {
            let new = Tile::floor(parent.get(), transform.translation.xy());
            let old = if tile.is_added() {
                None
            } else if *tile != new {
                Some(*tile)
            } else {
                return;
            };

            *tile = new;
            writer.lock().write(TileChanged {
                id,
                old,
                new: Some(new),
            });
        });
}

pub fn update_index(mut reader: MessageReader<TileChanged>, mut index: ResMut<TileIndex>) {
    for event in reader.read() {
        if let Some(old) = event.old {
            index.remove(event.id, old);
        }

        if let Some(new) = event.new {
            index.insert(event.id, new);
        }
    }
}

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileIndex>()
            .add_message::<TileChanged>();

        app.add_systems(FixedUpdate, (update_tile, update_index).chain());
    }
}

impl Tile {
    pub fn new(layer: Entity, position: IVec2) -> Self {
        Tile { layer, position }
    }

    pub fn floor(layer: Entity, position: Vec2) -> Self {
        Tile::new(layer, IVec2::new(floor(position.x), floor(position.y)))
    }

    pub fn layer(&self) -> Entity {
        self.layer
    }

    pub fn position(&self) -> IVec2 {
        self.position
    }

    pub fn x(&self) -> i32 {
        self.position.x
    }

    pub fn y(&self) -> i32 {
        self.position.y
    }

    pub fn neighborhood(&self) -> [Tile; 9] {
        let layer = self.layer();
        let (x, y) = (self.x(), self.y());

        [
            Tile::new(layer, IVec2::new(x - 1, y - 1)),
            Tile::new(layer, IVec2::new(x, y - 1)),
            Tile::new(layer, IVec2::new(x + 1, y - 1)),
            Tile::new(layer, IVec2::new(x - 1, y)),
            Tile::new(layer, IVec2::new(x, y)),
            Tile::new(layer, IVec2::new(x + 1, y)),
            Tile::new(layer, IVec2::new(x - 1, y + 1)),
            Tile::new(layer, IVec2::new(x, y + 1)),
            Tile::new(layer, IVec2::new(x + 1, y + 1)),
        ]
    }

    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        let tile = world.entity(context.entity).get_ref::<Tile>().unwrap();
        if tile.last_changed() != tile.added() {
            world.write_message(TileChanged {
                id: context.entity,
                old: Some(*tile),
                new: None,
            });
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            layer: Entity::PLACEHOLDER,
            position: IVec2::ZERO,
        }
    }
}

impl fmt::Debug for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tile({:?}, {}, {})", self.layer(), self.x(), self.y())
    }
}

fn floor(value: f32) -> i32 {
    if value.is_sign_negative() {
        (value as i32) - 1
    } else {
        value as i32
    }
}

#[test]
fn test_floor() {
    assert_eq!(floor(0.0), 0);
    assert_eq!(floor(-0.0), -1);
    assert_eq!(floor(f32::EPSILON), 0);
    assert_eq!(floor(-f32::EPSILON), -1);
    assert_eq!(floor(0.00001), 0);
    assert_eq!(floor(-0.00001), -1);
    assert_eq!(floor(0.1), 0);
    assert_eq!(floor(-0.1), -1);
    assert_eq!(floor(0.9), 0);
    assert_eq!(floor(-0.9), -1);
    assert_eq!(floor(0.99999), 0);
    assert_eq!(floor(-0.99999), -1);
    assert_eq!(floor(1.0), 1);
    assert_eq!(floor(f32::next_down(1.0)), 0);
    assert_eq!(floor(-1.0), -2);
    assert_eq!(floor(f32::next_up(-1.0)), -1);
    assert_eq!(floor(1.00001), 1);
    assert_eq!(floor(-1.00001), -2);
    assert_eq!(floor(1.1), 1);
    assert_eq!(floor(-1.1), -2);
    assert_eq!(floor(1.5), 1);
    assert_eq!(floor(-1.5), -2);
    assert_eq!(floor(1.9), 1);
    assert_eq!(floor(-1.9), -2);
    assert_eq!(floor(2.0), 2);
    assert_eq!(floor(-2.0), -3);
    assert_eq!(floor(2.5), 2);
    assert_eq!(floor(-2.5), -3);
    assert_eq!(floor(42.0), 42);
    assert_eq!(floor(-42.0), -43);
    assert_eq!(floor(42.7), 42);
    assert_eq!(floor(-42.7), -43);
    assert_eq!(floor(99.99), 99);
    assert_eq!(floor(-99.99), -100);
    assert_eq!(floor(100.0), 100);
    assert_eq!(floor(-100.0), -101);
    assert_eq!(floor(999.99999), 1000);
    assert_eq!(floor(-999.99999), -1001);
    assert_eq!(floor(1000.0), 1000);
    assert_eq!(floor(-1000.0), -1001);
    assert_eq!(floor(1000000.0), 1000000);
    assert_eq!(floor(1000000.5), 1000000);
    assert_eq!(floor(-1000000.0), -1000001);
    assert_eq!(floor(-1000000.5), -1000001);
    assert_eq!(floor(2147483520.0), 2147483520);
    assert_eq!(floor(-2147483520.0), -2147483521);
}
