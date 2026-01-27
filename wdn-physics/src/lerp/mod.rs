#[cfg(test)]
mod tests;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_time::prelude::*;
use bevy_transform::prelude::*;

use crate::kinematics::RelativePosition;

pub struct InterpolatePlugin;

#[derive(Debug, Clone, Copy, Component)]
#[require(InterpolateState, Transform)]
pub struct Interpolate {
    pub translation: bool,
    pub rotation: bool,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct InterpolateState {
    translation: ComponentInterpolateState<Vec2>,
    rotation: ComponentInterpolateState<Rot2>,
}

#[derive(Component, Clone, Copy, Debug, Default)]
enum ComponentInterpolateState<T> {
    #[default]
    Unset,
    Static {
        value: T,
    },
    Interpolating {
        start: T,
        end: T,
    },
}

#[derive(Resource, Default)]
pub struct FixedUpdateCount {
    pub updates: u64,
}

pub fn count_fixed_update(mut count: ResMut<FixedUpdateCount>) {
    count.updates += 1;
}

pub fn interpolate(
    mut count: ResMut<FixedUpdateCount>,
    mut transforms: Query<(
        &Interpolate,
        &RelativePosition,
        &mut Transform,
        &mut InterpolateState,
    )>,
    time: Res<Time<Fixed>>,
) {
    let overstep = time.overstep_fraction();

    transforms
        .par_iter_mut()
        .for_each(|(interpolate, position, mut transform, mut state)| {
            println!(
                "Interpolating entity: {:?} with overstep {} and value {position:?} and count {}",
                state, overstep, count.updates
            );

            if interpolate.translation {
                if let Some(interpolated_translation) =
                    state
                        .translation
                        .interpolate(position.position(), overstep, count.updates > 0)
                {
                    println!("interpolated translation: {:?}", interpolated_translation);
                    transform.translation.x = interpolated_translation.x;
                    transform.translation.y = interpolated_translation.y;
                }
            }

            if interpolate.rotation {
                if let Some(interpolated_rotation) =
                    state
                        .rotation
                        .interpolate(position.rotation(), overstep, count.updates > 0)
                {
                    transform.rotation = Quat::from_rotation_z(interpolated_rotation.as_radians());
                }
            }

            println!("new state: {:?}", *state);
        });

    count.updates = 0;
}

impl Plugin for InterpolatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FixedUpdateCount>();

        app.add_systems(FixedLast, count_fixed_update);

        app.add_systems(
            RunFixedMainLoop,
            interpolate.in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
        );
    }
}

impl Default for Interpolate {
    fn default() -> Self {
        Self {
            translation: true,
            rotation: true,
        }
    }
}

impl<T> ComponentInterpolateState<T>
where
    T: Copy + PartialEq + StableInterpolate,
{
    fn interpolate(&mut self, value: T, t: f32, updated: bool) -> Option<T> {
        match *self {
            ComponentInterpolateState::Unset => {
                *self = ComponentInterpolateState::Static { value };
                Some(value)
            }
            ComponentInterpolateState::Static { value: old_value } => {
                if old_value != value {
                    if updated {
                        *self = ComponentInterpolateState::Interpolating {
                            start: old_value,
                            end: value,
                        };
                        Some(old_value.interpolate_stable(&value, t))
                    } else {
                        *self = ComponentInterpolateState::Static { value };
                        Some(value)
                    }
                } else {
                    None
                }
            }
            ComponentInterpolateState::Interpolating { start, end } => {
                if end != value {
                    if updated {
                        *self = ComponentInterpolateState::Interpolating {
                            start: end,
                            end: value,
                        };
                        Some(end.interpolate_stable(&value, t))
                    } else {
                        *self = ComponentInterpolateState::Static { value };
                        Some(value)
                    }
                } else {
                    Some(start.interpolate_stable(&end, t))
                }
            }
        }
    }
}
