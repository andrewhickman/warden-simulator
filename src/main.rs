use bevy::prelude::*;

use wdn_physics::WdnPhysicsPlugin;
use wdn_render::WdnRenderPlugin;
use wdn_save::WdnSavePlugin;
use wdn_tasks::WdnTasksPlugin;
use wdn_ui::WdnUiPlugin;

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
