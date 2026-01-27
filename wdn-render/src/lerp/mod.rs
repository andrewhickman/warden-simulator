#[cfg(test)]
mod tests;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_time::prelude::*;
use bevy_transform::prelude::*;

use wdn_physics::{kinematics::Position, layer::Layer};

use crate::RenderSystems;

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
        &Position,
        &mut Transform,
        &mut InterpolateState,
    )>,
    time: Res<Time<Fixed>>,
) {
    let overstep = time.overstep_fraction();

    transforms
        .par_iter_mut()
        .for_each(|(interpolate, position, mut transform, mut state)| {
            if interpolate.translation {
                if let Some(interpolated_translation) =
                    state
                        .translation
                        .interpolate(position.position(), overstep, count.updates > 0)
                {
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
        });

    count.updates = 0;
}

impl Plugin for InterpolatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FixedUpdateCount>();

        app.register_required_components::<Layer, Transform>();

        app.add_systems(FixedLast, count_fixed_update);

        app.add_systems(
            PostUpdate,
            interpolate
                .in_set(RenderSystems::Interpolate)
                .before(TransformSystems::Propagate),
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

impl Interpolate {
    pub fn translation() -> Self {
        Self {
            translation: true,
            rotation: false,
        }
    }

    pub fn rotation() -> Self {
        Self {
            translation: false,
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
