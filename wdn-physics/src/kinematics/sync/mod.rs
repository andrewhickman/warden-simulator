#[cfg(test)]
mod tests;

use bevy_ecs::{prelude::*, query::QueryData, relationship::Relationship};
use bevy_math::prelude::*;

use crate::{
    kinematics::{GlobalPosition, GlobalVelocity, Position, Velocity},
    layer::Layer,
    tile::TilePosition,
};

pub struct SyncPlugin;

#[derive(QueryData, Debug)]
#[query_data(mutable)]
pub struct SyncQuery {
    id: Entity,
    relative: SyncRelativeQuery,
    tile: &'static TilePosition,
    position: Mut<'static, GlobalPosition>,
    velocity: Option<Mut<'static, GlobalVelocity>>,
}

#[derive(QueryData, Debug)]
pub struct SyncRelativeQuery {
    parent: Ref<'static, ChildOf>,
    position: Ref<'static, Position>,
    velocity: Option<Ref<'static, Velocity>>,
}

pub fn sync_kinematics(
    commands: ParallelCommands,
    mut entities: Query<SyncQuery>,
    parents: Query<SyncRelativeQuery>,
    layers: Query<&Layer>,
) {
    entities.par_iter_mut().for_each(|mut item| {
        commands.command_scope(|mut commands| item.sync(&mut commands, &layers, &parents))
    });
}

pub fn sync_on_add(
    trigger: On<Add, GlobalPosition>,
    mut commands: Commands,
    mut entities: Query<SyncQuery>,
    parents: Query<SyncRelativeQuery>,
    layers: Query<&Layer>,
) -> Result {
    entities
        .get_mut(trigger.entity)?
        .sync(&mut commands, &layers, &parents);
    Ok(())
}

impl SyncQueryItem<'_, '_> {
    pub fn sync(
        &mut self,
        commands: &mut Commands,
        layers: &Query<&Layer>,
        parents: &Query<SyncRelativeQuery>,
    ) {
        let mut parent = self.relative.parent.get();
        let mut has_parent = !layers.contains(parent);
        let mut any_changed = self.relative.changed();

        if !has_parent && !any_changed {
            return;
        }

        let mut isometry = self.relative.position.isometry;
        let mut angular = self.relative.velocity.as_ref().map_or(0.0, |v| v.angular());
        let mut linear = self
            .relative
            .velocity
            .as_ref()
            .map_or(Vec2::ZERO, |v| v.linear());

        while has_parent {
            let ancestor = parents.get(parent).expect("invalid parent");

            has_parent = !layers.contains(ancestor.parent.get());
            any_changed = any_changed || ancestor.changed();
            if !has_parent && !any_changed {
                return;
            }

            if let Some(ancestor_velocity) = &ancestor.velocity {
                linear += isometry.translation.perp() * ancestor_velocity.angular();
            }

            linear = ancestor.position.rotation() * linear;

            if let Some(ancestor_velocity) = &ancestor.velocity {
                linear += ancestor_velocity.linear();
                angular += ancestor_velocity.angular();
            }

            isometry = ancestor.position.isometry * isometry;
            parent = ancestor.parent.get();
        }

        *self.position = GlobalPosition::from_isometry(isometry);
        if let Some(velocity) = self.velocity.as_mut() {
            **velocity = GlobalVelocity::new(linear, angular);
        }

        let new_tile = TilePosition::floor(parent, self.position.position());
        if *self.tile != new_tile {
            commands.entity(self.id).try_insert(new_tile);
        }
    }
}

impl SyncRelativeQueryItem<'_, '_> {
    fn changed(&self) -> bool {
        self.position.is_changed()
            || self.parent.is_changed()
            || self.velocity.as_ref().is_some_and(|v| v.is_changed())
    }
}
