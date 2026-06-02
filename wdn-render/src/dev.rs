use bevy_app::prelude::*;
use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_gizmos::prelude::*;
use bevy_math::prelude::*;
use wdn_physics::kinematics::Position;
use wdn_world::path::flow::{DoorRegions, FlowField};
use wdn_world::pawn::Pawn;

use crate::RenderSystems;

pub struct DevPlugin;

#[derive(Resource)]
pub struct DevRenderSettings {
    pub draw_pawn_colliders: bool,
    pub draw_door_flow_fields: Option<Entity>,
}

pub fn draw_pawn_colliders_enabled(settings: Res<DevRenderSettings>) -> bool {
    settings.draw_pawn_colliders
}

pub fn draw_pawn_colliders(mut gizmos: Gizmos, pawns: Query<&Position, With<Pawn>>) {
    pawns.iter().for_each(|position| {
        gizmos.circle_2d(
            position.position(),
            Pawn::RADIUS,
            Color::srgb(0.3, 0.95, 0.35),
        );
    });
}

pub fn draw_door_flow_fields_enabled(settings: Res<DevRenderSettings>) -> bool {
    settings.draw_door_flow_fields.is_some()
}

pub fn draw_door_flow_fields(
    mut gizmos: Gizmos,
    settings: Res<DevRenderSettings>,
    door_regions: Query<&DoorRegions>,
    flow_fields: Query<&FlowField>,
) {
    let Some(door) = settings.draw_door_flow_fields else {
        return;
    };

    let Ok(regions) = door_regions.get(door) else {
        return;
    };

    for flow_id in regions.flow_fields() {
        let Ok(flow) = flow_fields.get(flow_id) else {
            continue;
        };

        for (tile_pos, dir) in flow.iter() {
            let center = Vec2::new(tile_pos.x() as f32 + 0.5, tile_pos.y() as f32 + 0.5);
            let end = center + dir.as_vec2() * 0.4;
            gizmos.arrow_2d(center, end, Color::srgb(0.95, 0.75, 0.1));
        }
    }
}

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DevRenderSettings {
            draw_pawn_colliders: true,
            draw_door_flow_fields: None,
        });

        app.add_systems(
            Update,
            (
                draw_pawn_colliders.run_if(draw_pawn_colliders_enabled),
                draw_door_flow_fields.run_if(draw_door_flow_fields_enabled),
            )
                .in_set(RenderSystems::RenderDev),
        );
    }
}
