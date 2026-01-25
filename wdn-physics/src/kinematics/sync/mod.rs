#[cfg(test)]
mod tests;

use bevy_ecs::{prelude::*, query::QueryData, relationship::Relationship};
use bevy_math::prelude::*;
use bevy_transform::prelude::*;

use crate::{
    kinematics::{Position, RelativeVelocity, Velocity},
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
    position: Mut<'static, Position>,
    velocity: Option<Mut<'static, Velocity>>,
}

#[derive(QueryData, Debug)]
pub struct SyncRelativeQuery {
    parent: Ref<'static, ChildOf>,
    transform: Ref<'static, Transform>,
    velocity: Option<Ref<'static, RelativeVelocity>>,
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
    trigger: On<Add, Position>,
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

        let mut isometry = transform_to_isometry(&self.relative.transform);
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

            let ancestor_isometry = transform_to_isometry(&ancestor.transform);

            if let Some(ancestor_velocity) = &ancestor.velocity {
                linear += isometry.translation.perp() * ancestor_velocity.angular();
            }

            linear = ancestor_isometry.rotation * linear;

            if let Some(ancestor_velocity) = &ancestor.velocity {
                linear += ancestor_velocity.linear();
                angular += ancestor_velocity.angular();
            }

            isometry = ancestor_isometry * isometry;
            parent = ancestor.parent.get();
        }

        *self.position = Position::new(isometry);
        if let Some(velocity) = self.velocity.as_mut() {
            **velocity = Velocity::new(linear, angular);
        }

        let new_tile = TilePosition::floor(parent, self.position.position());
        if *self.tile != new_tile {
            commands.entity(self.id).insert(new_tile);
        }
    }
}

impl SyncRelativeQueryItem<'_, '_> {
    fn changed(&self) -> bool {
        self.transform.is_changed()
            || self.parent.is_changed()
            || self.velocity.as_ref().is_some_and(|v| v.is_changed())
    }
}

pub fn transform_to_isometry(transform: &Transform) -> Isometry2d {
    let translation = transform.translation.xy();
    let rotation = quat_to_rot(transform.rotation);
    Isometry2d::new(translation, rotation)
}

pub fn quat_to_rot(quat: Quat) -> Rot2 {
    let y2 = quat.y + quat.y;
    let z2 = quat.z + quat.z;
    let cx = 1.0 - (quat.y * y2 + quat.z * z2);
    let cy = quat.w * z2 - quat.x * y2;

    match Dir2::from_xy(cx, cy) {
        Ok(dir) => Rot2::from_sin_cos(dir.y, dir.x),
        Err(_) => Rot2::IDENTITY,
    }
}
