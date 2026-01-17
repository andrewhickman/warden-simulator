use std::time::Duration;

use bevy_app::prelude::*;
use bevy_color::{Color, Mix};
use bevy_ecs::prelude::*;
use bevy_math::{Curve, curve::ExponentialInCurve};
use bevy_sprite::prelude::*;
use bevy_time::{common_conditions::paused, prelude::*};
use wdn_world::combat::Damaged;

pub struct DamagePlugin;

#[derive(Copy, Clone, Component, Debug)]
pub struct DamageAnimation {
    pub target: Entity,
    pub elapsed: f32,
}

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_damage_animations,
                update_damage_animations.run_if(not(paused)),
            )
                .chain(),
        );
    }
}

pub fn spawn_damage_animations(
    mut commands: Commands,
    mut damage_messages: MessageReader<Damaged>,
) {
    damage_messages.read().for_each(|damaged| {
        if commands.get_entity(damaged.target).is_err() {
            return;
        }

        commands.spawn((
            DamageAnimation {
                target: damaged.target,
                elapsed: 0.0,
            },
            ChildOf(damaged.target),
        ));
    });
}

pub fn update_damage_animations(
    mut commands: Commands,
    mut animations: Query<(Entity, &mut DamageAnimation)>,
    mut sprites: Query<&mut Sprite>,
    time: Res<Time>,
) {
    animations.iter_mut().for_each(|(id, mut animation)| {
        if let Ok(mut sprite) = sprites.get_mut(animation.target) {
            sprite.color = if let Some(t) = ExponentialInCurve
                .sample(animation.elapsed / DamageAnimation::DURATION.as_secs_f32())
            {
                DamageAnimation::COLOR.mix(&Color::WHITE, t)
            } else {
                commands.entity(id).try_despawn();
                Color::WHITE
            };
        }

        animation.elapsed += time.delta_secs();
    });
}

impl DamageAnimation {
    pub const DURATION: Duration = Duration::from_millis(600);
    pub const COLOR: Color = Color::linear_rgb(0.7, 0.0, 0.0);
}
