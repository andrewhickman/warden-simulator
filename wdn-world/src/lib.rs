pub mod combat;
pub mod door;
pub mod pawn;

use bevy_app::prelude::*;

use crate::combat::CombatPlugin;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CombatPlugin);
    }
}
