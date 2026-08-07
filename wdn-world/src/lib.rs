pub mod combat;
pub mod door;
pub mod ground;
pub mod path;
pub mod pawn;
pub mod stair;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use crate::combat::CombatPlugin;
use crate::door::DoorPlugin;
use crate::ground::GroundPlugin;
use crate::path::PathPlugin;
use crate::pawn::PawnPlugin;

pub struct WorldPlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub enum WorldSystems {
    ApplyPawnActions,
    ApplyProjectiles,
    UpdateRegions,
    UpdateFlowFields,
    UpdateDoors,
    UpdateGround,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CombatPlugin,
            DoorPlugin,
            GroundPlugin,
            PawnPlugin,
            PathPlugin,
        ));
    }
}
