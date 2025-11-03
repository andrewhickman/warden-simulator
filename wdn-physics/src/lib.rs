pub mod tile;

use bevy::prelude::*;

use crate::tile::TilePlugin;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin);
    }
}
