use bevy_app::prelude::*;
use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_gizmos::prelude::*;
use wdn_physics::kinematics::Position;
use wdn_world::path::door::DoorRegions;
use wdn_world::path::find::PathEntry;
use wdn_world::path::flow::FlowField;
use wdn_world::pawn::Pawn;
use wdn_world::pawn::path::PawnPath;

use crate::RenderSystems;

pub struct DevPlugin;

#[derive(Resource)]
pub struct DevRenderSettings {
    pub draw_pawn_colliders: bool,
    pub draw_door_flow_fields: Option<Entity>,
    pub draw_pawn_paths: Option<Entity>,
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

        // for (tile_pos, entry) in flow.iter() {
        //     let center = Vec2::new(tile_pos.x() as f32 + 0.5, tile_pos.y() as f32 + 0.5);
        //     let end = center + entry.dir().as_vec2() * 0.4;
        //     gizmos.arrow_2d(center, end, Color::srgb(0.95, 0.75, 0.1));
        // }
    }
}

pub fn draw_pawn_paths_enabled(settings: Res<DevRenderSettings>) -> bool {
    settings.draw_pawn_paths.is_some()
}

pub fn draw_pawn_paths(
    mut gizmos: Gizmos,
    settings: Res<DevRenderSettings>,
    pawn_paths: Query<&PawnPath>,
    flow_fields: Query<&FlowField>,
) {
    let Some(entity) = settings.draw_pawn_paths else {
        return;
    };

    let Ok(path) = pawn_paths.get(entity) else {
        return;
    };

    // let Some(target) = path.target() else {
    //     return;
    // };

    let Some(path) = path.path() else {
        return;
    };

    for entry in path.iter() {
        match entry {
            PathEntry::InRegion { .. } => {
                // for (tile_pos, entry) in cost_field.iter_flow(target.layer_offset()) {
                //     let color = Color::srgb(0.3, 0.95, 0.35);

                //     let center = Vec2::new(tile_pos.x() as f32 + 0.5, tile_pos.y() as f32 + 0.5);
                //     let end = center + entry.dir().as_vec2() * 0.4;
                //     gizmos.arrow_2d(center, end, color);
                // }
            }
            PathEntry::ToDoor { flow_field, .. } => {
                let Ok(flow) = flow_fields.get(*flow_field) else {
                    return;
                };

                // for (tile_pos, entry) in flow.iter() {
                //     let center = Vec2::new(tile_pos.x() as f32 + 0.5, tile_pos.y() as f32 + 0.5);
                //     let end = center + entry.dir().as_vec2() * 0.4;
                //     gizmos.arrow_2d(center, end, Color::srgb(0.6, 0.3, 0.8));
                // }
            }
        }
    }
}

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DevRenderSettings {
            draw_pawn_colliders: true,
            draw_door_flow_fields: None,
            draw_pawn_paths: None,
        });

        app.add_systems(
            Update,
            (
                draw_pawn_colliders.run_if(draw_pawn_colliders_enabled),
                draw_door_flow_fields.run_if(draw_door_flow_fields_enabled),
                draw_pawn_paths.run_if(draw_pawn_paths_enabled),
            )
                .in_set(RenderSystems::RenderDev),
        );
    }
}
