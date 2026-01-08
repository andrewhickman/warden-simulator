use bevy_app::prelude::*;
use bevy_asset::{UntypedAssetId, prelude::*};
use bevy_ecs::prelude::*;
use bevy_image::{ImageArrayLayout, ImageLoaderSettings, ImageSampler, prelude::*};

pub struct AssetsPlugin;

#[derive(Debug, Resource)]
pub struct AssetHandles {
    pub tileset: Handle<Image>,
    pub pawn: Handle<Image>,
    pub pawn_projectile: Handle<Image>,
}

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load);
    }
}

impl AssetHandles {
    pub fn asset_ids(&self) -> impl Iterator<Item = UntypedAssetId> + '_ {
        let AssetHandles {
            tileset,
            pawn,
            pawn_projectile,
        } = self;

        [tileset.into(), pawn.into(), pawn_projectile.into()].into_iter()
    }
}

pub fn load(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(AssetHandles {
        tileset: assets.load_with_settings("image/tileset.png", set_tileset),
        pawn: assets.load_with_settings("image/pawn.png", set_nearest),
        pawn_projectile: assets.load_with_settings("image/pawn_projectile.png", set_nearest),
    });
}

fn set_tileset(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::nearest();
    settings.array_layout = Some(ImageArrayLayout::RowCount { rows: 2 });
}

fn set_nearest(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::nearest();
}

#[cfg(test)]
mod tests {
    use bevy_app::{ScheduleRunnerPlugin, prelude::*};
    use bevy_asset::{AssetPlugin as BevyAssetPlugin, LoadState, prelude::*};
    use bevy_ecs::prelude::*;
    use bevy_image::*;
    use bevy_render::texture::TexturePlugin;

    use crate::assets::{AssetHandles, AssetsPlugin};

    #[test]
    fn load_assets() {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            ScheduleRunnerPlugin::default(),
            BevyAssetPlugin {
                file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/../assets").to_owned(),
                ..Default::default()
            },
            ImagePlugin::default(),
            TexturePlugin,
            AssetsPlugin,
        ));

        app.add_systems(Update, update);

        assert_eq!(app.run(), AppExit::Success);
    }

    fn update(
        assets: Res<AssetHandles>,
        asset_server: Res<AssetServer>,
        mut exit_e: MessageWriter<AppExit>,
    ) {
        for asset_id in assets.asset_ids() {
            match asset_server.get_load_state(asset_id) {
                None | Some(LoadState::NotLoaded) => panic!("Asset not loading"),
                Some(LoadState::Loading) => return,
                Some(LoadState::Loaded) => continue,
                Some(LoadState::Failed(error)) => {
                    panic!("Failed to load asset: {error}")
                }
            }
        }

        exit_e.write(AppExit::Success);
    }
}
