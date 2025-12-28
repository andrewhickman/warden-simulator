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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;

    use crate::tile::TilePlugin;

    #[test]
    fn child_spawned() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(Layer::default()).id();
        let child = app.world_mut().spawn(ChildOf(layer)).id();

        let in_layer = app.world().entity(child).get::<InLayer>().unwrap();
        assert_eq!(in_layer.0, layer);
    }

    #[test]
    fn grandchild_spawned() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(Layer::default()).id();
        let child = app.world_mut().spawn(ChildOf(layer)).id();
        let grandchild = app.world_mut().spawn(ChildOf(child)).id();

        let in_layer = app.world().entity(grandchild).get::<InLayer>().unwrap();
        assert_eq!(in_layer.0, layer);
    }

    #[test]
    fn hierarchy_spawned() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app
            .world_mut()
            .spawn((Layer::default(), children![(), children![(), ()]]))
            .id();

        let mut query = app.world_mut().query::<&InLayer>();
        assert_eq!(query.iter(app.world()).count(), 4);
        assert!(query.iter(app.world()).all(|i| i.0 == layer));
    }
}
