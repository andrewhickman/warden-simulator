#[cfg(test)]
mod tests;

use bevy_app::prelude::*;
use bevy_ecs::{
    change_detection::Tick, prelude::*, relationship::Relationship, system::SystemChangeTick,
};
use bevy_math::prelude::*;
use bevy_transform::prelude::*;

use crate::{
    PhysicsSystems,
    kinematics::Velocity,
    layer::{Layer, LayerPosition, LayerVelocity},
    lerp::start_interpolation,
    tile::TilePosition,
};

pub struct SyncPlugin;

#[derive(Debug, Default, Resource)]
pub struct SyncChangeTick(Tick);

pub fn sync_kinematics(
    commands: ParallelCommands,
    mut entities: Query<(
        Entity,
        Ref<ChildOf>,
        Ref<Transform>,
        Option<Ref<Velocity>>,
        Ref<TilePosition>,
        &mut LayerPosition,
        Option<&mut LayerVelocity>,
    )>,
    parents: Query<(Ref<ChildOf>, Ref<Transform>, Option<Ref<Velocity>>)>,
    layers: Query<&Layer>,
    ticks: SystemChangeTick,
    mut last_run: ResMut<SyncChangeTick>,
) {
    entities.par_iter_mut().for_each(
        |(id, mut parent, transform, velocity, old, mut layer_position, layer_velocity)| {
            let mut has_parent = !layers.contains(parent.get());
            let mut any_changed = position_changed(
                &transform,
                &parent,
                &velocity,
                last_run.tick(),
                ticks.this_run(),
            );
            if !has_parent && !any_changed {
                return;
            }

            let mut isometry = transform_to_isometry(&transform);
            let mut angular = velocity.as_ref().map_or(0.0, |v| v.angular());
            let mut linear = velocity.as_ref().map_or(Vec2::ZERO, |v| v.linear());

            while has_parent {
                let (ancestor_parent, ancestor_transform, ancestor_velocity) =
                    parents.get(parent.get()).expect("invalid parent");

                has_parent = !layers.contains(ancestor_parent.get());
                any_changed = any_changed
                    || position_changed(
                        &ancestor_transform,
                        &ancestor_parent,
                        &ancestor_velocity,
                        last_run.tick(),
                        ticks.this_run(),
                    );
                if !has_parent && !any_changed {
                    return;
                }

                let ancestor_isometry = transform_to_isometry(&ancestor_transform);

                if let Some(ancestor_velocity) = &ancestor_velocity {
                    linear += isometry.translation.perp() * ancestor_velocity.angular();
                }

                linear = ancestor_isometry.rotation * linear;

                if let Some(ancestor_velocity) = &ancestor_velocity {
                    linear += ancestor_velocity.linear();
                    angular += ancestor_velocity.angular();
                }

                isometry = ancestor_isometry * isometry;
                parent = ancestor_parent;
            }

            *layer_position = LayerPosition::new(isometry);
            if let Some(mut layer_velocity) = layer_velocity {
                *layer_velocity = LayerVelocity::new(linear, angular);
            }

            let new = TilePosition::floor(parent.get(), layer_position.position());
            if *old != new {
                commands.command_scope(move |mut commands| {
                    commands.entity(id).insert(new);
                });
            }
        },
    );

    last_run.set_tick(ticks.this_run());
}

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SyncChangeTick>();

        app.add_systems(FixedUpdate, sync_kinematics.in_set(PhysicsSystems::Sync));
        app.add_systems(
            RunFixedMainLoop,
            sync_kinematics
                .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop)
                .before(start_interpolation),
        );
    }
}

impl SyncChangeTick {
    pub fn tick(&self) -> Tick {
        self.0
    }

    pub fn set_tick(&mut self, tick: Tick) {
        self.0 = tick;
    }
}

fn position_changed(
    transform: &Ref<Transform>,
    parent: &Ref<ChildOf>,
    velocity: &Option<Ref<Velocity>>,
    last_run: Tick,
    this_run: Tick,
) -> bool {
    transform.last_changed().is_newer_than(last_run, this_run)
        || parent.last_changed().is_newer_than(last_run, this_run)
        || velocity
            .as_ref()
            .is_some_and(|v| v.last_changed().is_newer_than(last_run, this_run))
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
