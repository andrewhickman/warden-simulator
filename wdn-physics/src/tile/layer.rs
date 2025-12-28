use bevy_ecs::{entity::EntityHashSet, prelude::*, relationship::Relationship};
use bevy_math::prelude::*;
use bevy_transform::prelude::*;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Transform)]
pub struct Layer {}

#[derive(Copy, Clone, Component, Debug)]
#[relationship(relationship_target = LayerEntities)]
pub struct InLayer(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = InLayer, linked_spawn)]
pub struct LayerEntities(EntityHashSet);

#[derive(Copy, Clone, Component, Debug, Default)]
pub struct LayerPosition(pub Vec2);

pub fn child_added(
    insert: On<Insert, ChildOf>,
    mut commands: Commands,
    parents: Query<&ChildOf>,
    layers: Query<AnyOf<((Entity, &Layer), &InLayer)>>,
) {
    let parent = parents.get(insert.entity).unwrap();
    match layers.get(parent.get()) {
        Ok((Some((layer, _)), _) | (None, Some(&InLayer(layer)))) => {
            commands
                .entity(insert.entity)
                .insert_recursive::<Children>(InLayer(layer));
        }
        _ => {}
    }
}
