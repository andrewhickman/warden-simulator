use bevy_app::prelude::*;
use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_gizmos::prelude::*;
use wdn_physics::kinematics::Position;
use wdn_world::pawn::Pawn;

pub struct DevPlugin;

#[derive(Resource)]
pub struct DevRenderSettings {
    pub draw_pawn_colliders: bool,
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

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DevRenderSettings {
            draw_pawn_colliders: true,
        });

        app.add_systems(
            Update,
            draw_pawn_colliders.run_if(draw_pawn_colliders_enabled),
        );
    }
}
