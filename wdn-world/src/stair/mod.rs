use bevy_ecs::{lifecycle::HookContext, prelude::*, world::DeferredWorld};
use wdn_physics::{collision::TileCollider, tile::material::TileMaterial};

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
    pub const NORTH: TileMaterial = TileMaterial::STAIR.with_id(Self::NORTH_ID);
    pub const SOUTH: TileMaterial = TileMaterial::STAIR.with_id(Self::SOUTH_ID);
    pub const EAST: TileMaterial = TileMaterial::STAIR.with_id(Self::EAST_ID);
    pub const WEST: TileMaterial = TileMaterial::STAIR.with_id(Self::WEST_ID);

    pub const NORTH_ID: u16 = 0;
    pub const SOUTH_ID: u16 = 1;
    pub const EAST_ID: u16 = 2;
    pub const WEST_ID: u16 = 3;

    pub fn to_material_id(&self) -> u16 {
        match self {
            Stair::North => Self::NORTH_ID,
            Stair::South => Self::SOUTH_ID,
            Stair::East => Self::EAST_ID,
            Stair::West => Self::WEST_ID,
        }
    }

    pub fn from_material_id(id: u16) -> Self {
        match id & 0x0003 {
            Self::NORTH_ID => Stair::North,
            Self::SOUTH_ID => Stair::South,
            Self::EAST_ID => Stair::East,
            Self::WEST_ID => Stair::West,
            _ => panic!("invalid Stair id: {id}"),
        }
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let stair = *world.get::<Stair>(context.entity).unwrap();
        world
            .get_mut::<TileMaterial>(context.entity)
            .unwrap()
            .set_id(stair.to_material_id());
    }
}
