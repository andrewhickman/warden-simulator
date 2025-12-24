use bevy::{
    asset::UntypedAssetId,
    image::{ImageArrayLayout, ImageLoaderSettings, ImageSampler},
    prelude::*,
};

pub struct AssetsPlugin;

#[derive(Debug, Resource)]
pub struct AssetHandles {
    pub tileset: Handle<Image>,
    pub pawn: Handle<Image>,
}

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreStartup, load);
    }
}

impl AssetHandles {
    pub fn asset_ids(&self) -> impl Iterator<Item = UntypedAssetId> + '_ {
        let AssetHandles { tileset, pawn } = self;

        [tileset.into(), pawn.into()].into_iter()
    }
}

pub fn load(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(AssetHandles {
        tileset: assets.load_with_settings("image/tileset.png", set_tileset),
        pawn: assets.load_with_settings("image/pawn.png", set_nearest),
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
    use bevy::{
        asset::{AssetPlugin as BevyAssetPlugin, LoadState},
        prelude::*,
        render::texture::TexturePlugin,
    };

    use crate::assets::{AssetHandles, AssetsPlugin};

    #[test]
    fn load_assets() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            BevyAssetPlugin {
                file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/../assets").to_owned(),
                ..default()
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
