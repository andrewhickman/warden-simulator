pub mod door;
pub mod health;
pub mod pawn;

use bevy_app::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, _: &mut App) {}
}
