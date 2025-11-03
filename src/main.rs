use bevy::prelude::*;

use wdn_physics::PhysicsPlugin as WdnPhysicsPlugin;
use wdn_render::RenderPlugin as WdnRenderPlugin;
use wdn_save::SavePlugin as WdnSavePlugin;
use wdn_tasks::TasksPlugin as WdnTasksPlugin;
use wdn_ui::UiPlugin as WdnUiPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WdnPhysicsPlugin,
            WdnTasksPlugin,
            WdnSavePlugin,
            WdnRenderPlugin,
            WdnUiPlugin,
        ))
        .run();
}
