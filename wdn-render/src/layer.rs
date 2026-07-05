use std::any::type_name_of_val;

use bevy_app::prelude::*;
use bevy_camera::visibility::Visibility;
use bevy_ecs::{prelude::*, relationship::Relationship};
use bevy_transform::components::Transform;
use wdn_physics::layer::{Layer, LayerStack};

use crate::depth::LAYER_HEIGHT;

#[derive(Resource)]
pub struct LayerView {
    pub stack: Entity,
    pub height: i32,
}

pub struct LayerPlugin;

impl Plugin for LayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<LayerStack, Transform>();
        app.register_required_components::<LayerStack, Visibility>();
        app.register_required_components::<Layer, Transform>();
        app.register_required_components_with::<Layer, Visibility>(|| Visibility::Hidden);

        app.world_mut()
            .add_observer(on_add_layer)
            .insert(Name::new(format!(
                "Observer({})",
                type_name_of_val(&on_add_layer)
            )));

        app.add_systems(
            Update,
            update_layer_visibility.run_if(resource_changed_or_removed::<LayerView>),
        );
    }
}

pub fn on_add_layer(
    trigger: On<Insert, Layer>,
    mut layers: Query<(&Layer, &mut Transform)>,
) -> Result {
    let (layer, mut transform) = layers.get_mut(trigger.entity)?;
    transform.translation.z = layer.height() as f32 * LAYER_HEIGHT;
    Ok(())
}

fn update_layer_visibility(
    layer_view: Option<Res<LayerView>>,
    mut layers: Query<(&Layer, &ChildOf, &mut Visibility)>,
) {
    for (layer, parent, mut visibility) in &mut layers {
        let new_visibility = if layer_view
            .as_ref()
            .is_some_and(|view| view.visible(layer, parent.get()))
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        visibility.set_if_neq(new_visibility);
    }
}

impl LayerView {
    pub fn new(stack: Entity, height: i32) -> Self {
        Self { stack, height }
    }

    pub fn visible(&self, layer: &Layer, parent: Entity) -> bool {
        parent == self.stack && layer.height() <= self.height
    }
}
