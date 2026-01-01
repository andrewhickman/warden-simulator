use std::iter;

use bevy_ecs::{
    entity::EntityHashSet,
    prelude::*,
    query::{QueryData, ReadOnlyQueryData},
    relationship::Relationship,
};
use bevy_math::prelude::*;
use bevy_transform::prelude::*;

#[derive(Copy, Clone, Component, Debug, Default)]
#[require(Transform)]
pub struct TileLayer {}

#[derive(Copy, Clone, Component, Debug)]
#[relationship(relationship_target = LayerEntities)]
pub struct InLayer(pub Entity);

#[derive(QueryData, Debug)]
#[query_data(derive(Copy, Clone, Debug))]
pub struct LayerEntityQuery {
    layer: &'static InLayer,
    parent: &'static ChildOf,
}

#[derive(Component, Debug)]
#[relationship_target(relationship = InLayer, linked_spawn)]
pub struct LayerEntities(EntityHashSet);

#[derive(Copy, Clone, Component, Debug, Default)]
pub struct LayerTransform(Isometry2d);

pub fn child_added(
    insert: On<Insert, ChildOf>,
    mut commands: Commands,
    parents: Query<&ChildOf>,
    layers: Query<AnyOf<((Entity, &TileLayer), &InLayer)>>,
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

impl<'w, 's: 'w> LayerEntityQueryItem<'w, 's> {
    pub fn new(layer: &'w InLayer, parent: &'w ChildOf) -> LayerEntityQueryItem<'w, 's> {
        LayerEntityQueryItem { layer, parent }
    }

    pub fn ancestors<T>(
        &self,
        value: T::Item<'w, 's>,
        query: &'w Query<'w, 's, (LayerEntityQuery, T)>,
    ) -> impl Iterator<Item = T::Item<'w, 's>>
    where
        T: ReadOnlyQueryData,
    {
        iter::successors(Some((*self, value)), move |(parent, _)| {
            if let Some(parent) = parent.parent() {
                query.get(parent).ok()
            } else {
                return None;
            }
        })
        .map(|(_, v)| v)
    }

    pub fn has_parent(&self) -> bool {
        self.layer.get() != self.parent.get()
    }

    pub fn parent(&self) -> Option<Entity> {
        if self.has_parent() {
            Some(self.parent.get())
        } else {
            None
        }
    }

    pub fn layer(&self) -> Entity {
        self.layer.get()
    }
}

impl LayerTransform {
    pub fn from_ancestor_transforms<'a>(transforms: impl Iterator<Item = &'a Transform>) -> Self {
        let isometry = transforms
            .map(|transform| transform_to_isometry(&transform))
            .reduce(|a, b| b * a)
            .unwrap_or(Isometry2d::IDENTITY);
        LayerTransform(isometry)
    }

    pub fn position(&self) -> Vec2 {
        self.0.translation
    }

    pub fn rotation(&self) -> Rot2 {
        self.0.rotation
    }
}

fn transform_to_isometry(transform: &Transform) -> Isometry2d {
    let translation = transform.translation.xy();
    let rotation = quat_to_rot(transform.rotation);
    Isometry2d::new(translation, rotation)
}

fn quat_to_rot(quat: Quat) -> Rot2 {
    let y2 = quat.y + quat.y;
    let z2 = quat.z + quat.z;
    let cx = 1.0 - (quat.y * y2 + quat.z * z2);
    let cy = quat.w * z2 - quat.x * y2;

    match Dir2::from_xy(cx, cy) {
        Ok(dir) => Rot2::from_sin_cos(dir.y, dir.x),
        Err(_) => Rot2::IDENTITY,
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use approx::assert_relative_eq;
    use bevy_app::prelude::*;
    use bevy_ecs::prelude::*;
    use bevy_math::prelude::*;

    use crate::tile::{
        TilePlugin,
        layer::{InLayer, TileLayer, quat_to_rot},
    };

    #[test]
    fn quat_to_rot2_identity() {
        let rot = quat_to_rot(Quat::IDENTITY);
        assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
    }

    #[test]
    fn quat_to_rot2_z() {
        for angle in [
            -0.7853982, 0.0, 0.7853982, 1.0, 1.570796, 3.141593, 4.712389, 6.283185, 10.0,
        ] {
            let rot = quat_to_rot(Quat::from_rotation_z(angle));
            let expected = Rot2::radians(angle);
            assert_relative_eq!(rot, expected, epsilon = 1e-4);
        }
    }

    #[test]
    fn quat_to_rot2_x() {
        let rot = quat_to_rot(Quat::from_rotation_x(PI / 2.0));
        assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
    }

    #[test]
    fn quat_to_rot2_y() {
        let rot = quat_to_rot(Quat::from_rotation_y(PI / 2.0));
        assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
    }

    #[test]
    fn quat_to_rot2_xy() {
        let quat = Quat::from_rotation_x(1.0) * Quat::from_rotation_y(-1.5);
        let rot = quat_to_rot(quat);
        assert_relative_eq!(rot, Rot2::IDENTITY, epsilon = 1e-4);
    }

    #[test]
    fn quat_to_rot2_xz() {
        let angle = PI / 3.0;
        let rot = quat_to_rot(Quat::from_rotation_x(PI / 4.0) * Quat::from_rotation_z(angle));
        assert_relative_eq!(rot, Rot2::radians(angle), epsilon = 1e-5);
    }

    #[test]
    fn quat_to_rot2_yz() {
        let angle = PI / 3.0;
        let rot = quat_to_rot(Quat::from_rotation_y(PI / 3.0) * Quat::from_rotation_z(angle));
        assert_relative_eq!(rot, Rot2::radians(angle), epsilon = 1e-5);
    }

    #[test]
    fn quat_to_rot2_xyz() {
        let angle = 1.234;
        let quat = Quat::from_euler(EulerRot::XYZ, 1.5, 0.4, angle);
        let rot = quat_to_rot(quat);
        assert_relative_eq!(rot, Rot2::radians(angle), epsilon = 1e-5);
    }

    #[test]
    fn child_spawned() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer::default()).id();
        let child = app.world_mut().spawn(ChildOf(layer)).id();

        let in_layer = app.world().entity(child).get::<InLayer>().unwrap();
        assert_eq!(in_layer.0, layer);
    }

    #[test]
    fn grandchild_spawned() {
        let mut app = App::new();
        app.add_plugins(TilePlugin);

        let layer = app.world_mut().spawn(TileLayer::default()).id();
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
            .spawn((TileLayer::default(), children![(), children![(), ()]]))
            .id();

        let mut query = app.world_mut().query::<&InLayer>();
        assert_eq!(query.iter(app.world()).count(), 4);
        assert!(query.iter(app.world()).all(|i| i.0 == layer));
    }
}
