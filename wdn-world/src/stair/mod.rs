use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use wdn_physics::{
    collision::TileCollider,
    tile::{
        commands::TileCommandsExt,
        material::{TileMaterial, TileMaterialFlags},
        position::TilePosition,
    },
};

#[derive(Component, Clone, Copy, Debug)]
#[require(TileCollider::new(false), TileMaterial::STAIR)]
#[component(on_add = Stair::on_add)]
pub enum Stair {
    North,
    South,
    East,
    West,
}

impl Stair {
    pub fn flags(&self) -> TileMaterialFlags {
        match self {
            Stair::North => TileMaterialFlags::STAIR_DIRECTION_NORTH,
            Stair::South => TileMaterialFlags::STAIR_DIRECTION_SOUTH,
            Stair::East => TileMaterialFlags::STAIR_DIRECTION_EAST,
            Stair::West => TileMaterialFlags::STAIR_DIRECTION_WEST,
        }
    }

    pub fn from_flags(flags: TileMaterialFlags) -> Self {
        match flags.intersection(TileMaterialFlags::STAIR_DIRECTION_MASK) {
            TileMaterialFlags::STAIR_DIRECTION_NORTH => Stair::North,
            TileMaterialFlags::STAIR_DIRECTION_SOUTH => Stair::South,
            TileMaterialFlags::STAIR_DIRECTION_EAST => Stair::East,
            TileMaterialFlags::STAIR_DIRECTION_WEST => Stair::West,
            _ => unreachable!("invalid stair flags: {flags:?}"),
        }
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let stair = *world.get::<Stair>(context.entity).unwrap();
        let position = *world.get::<TilePosition>(context.entity).unwrap();
        world.commands().set_material(
            position,
            TileMaterial::STAIR.with_flags(TileMaterialFlags::STAIR_DIRECTION_MASK, stair.flags()),
        );
    }
}
