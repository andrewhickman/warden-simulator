pub mod combat;
pub mod door;
pub mod path;
pub mod pawn;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use crate::combat::CombatPlugin;
use crate::pawn::PawnPlugin;

pub struct WorldPlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub enum WorldSystems {
    ApplyPawnActions,
    ApplyProjectiles,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CombatPlugin, PawnPlugin));
    }
}
