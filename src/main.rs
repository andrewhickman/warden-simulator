#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;

use wdn_physics::{
    PhysicsPlugin as WdnPhysicsPlugin,
    tile::storage::{TileChunk, TileChunkOffset},
};
use wdn_render::RenderPlugin as WdnRenderPlugin;
use wdn_save::SavePlugin as WdnSavePlugin;
use wdn_tasks::TasksPlugin as WdnTasksPlugin;
use wdn_ui::UiPlugin as WdnUiPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Warden Simulator".to_string(),
                    canvas: Some("#bevy".to_owned()),
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            WdnPhysicsPlugin,
            WdnTasksPlugin,
            WdnSavePlugin,
            WdnRenderPlugin,
            WdnUiPlugin,
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(TileChunk::empty(TileChunkOffset::from_position(
        IVec2::ZERO,
    )));
}
