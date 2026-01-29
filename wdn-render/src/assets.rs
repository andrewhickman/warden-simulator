use bevy_app::prelude::*;
use bevy_asset::{UntypedAssetId, prelude::*};
use bevy_ecs::prelude::*;
use bevy_image::{ImageArrayLayout, ImageLoaderSettings, ImageSampler, prelude::*};
use bevy_math::prelude::*;
use bevy_sprite::prelude::*;

pub const PAWN_INDEX: usize = 0;
pub const PAWN_RECT: URect = URect {
    min: UVec2::new(16, 16),
    max: UVec2::new(208, 208),
};

pub const PAWN_PROJECTILE_INDEX: usize = 1;
pub const PAWN_PROJECTILE_RECT: URect = URect {
    min: UVec2::new(224, 16),
    max: UVec2::new(256, 48),
};

pub const DOOR_INDEX: usize = 2;
pub const DOOR_RECT: URect = URect {
    min: UVec2::new(16, 224),
    max: UVec2::new(400, 608),
};

pub struct AssetsPlugin;

#[derive(Debug, Resource)]
pub struct AssetHandles {
    tileset: Handle<Image>,
    atlas: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

pub fn sprite_size(rect: URect) -> Vec2 {
    rect.size().as_vec2() * 0.0025
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
            atlas,
            layout,
        } = self;

        [tileset.into(), atlas.into(), layout.into()].into_iter()
    }

    pub fn tileset(&self) -> Handle<Image> {
        self.tileset.clone()
    }

    pub fn atlas(&self) -> Handle<Image> {
        self.atlas.clone()
    }

    pub fn pawn(&self) -> Sprite {
        Sprite {
            image: self.atlas(),
            texture_atlas: Some(TextureAtlas {
                layout: self.layout.clone(),
                index: PAWN_INDEX,
            }),
            custom_size: Some(sprite_size(PAWN_RECT)),
            ..Default::default()
        }
    }

    pub fn pawn_projectile(&self) -> Sprite {
        Sprite {
            image: self.atlas(),
            texture_atlas: Some(TextureAtlas {
                layout: self.layout.clone(),
                index: PAWN_PROJECTILE_INDEX,
            }),
            custom_size: Some(sprite_size(PAWN_PROJECTILE_RECT)),
            ..Default::default()
        }
    }

    pub fn door(&self) -> Sprite {
        Sprite {
            image: self.atlas(),
            texture_atlas: Some(TextureAtlas {
                layout: self.layout.clone(),
                index: DOOR_INDEX,
            }),
            custom_size: Some(sprite_size(DOOR_RECT)),
            ..Default::default()
        }
    }
}

pub fn load(mut commands: Commands, assets: ResMut<AssetServer>) {
    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(1024, 1024));
    assert_eq!(layout.add_texture(PAWN_RECT), PAWN_INDEX);
    assert_eq!(
        layout.add_texture(PAWN_PROJECTILE_RECT),
        PAWN_PROJECTILE_INDEX
    );
    assert_eq!(layout.add_texture(DOOR_RECT), DOOR_INDEX);

    commands.insert_resource(AssetHandles {
        tileset: assets.load_with_settings("image/tileset.png", configure_tileset),
        atlas: assets.load_with_settings("image/atlas.png", configure_atlas),
        layout: assets.add(layout),
    });
}

fn configure_tileset(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::linear();
    settings.array_layout = Some(ImageArrayLayout::RowHeight { pixels: 64 });
}

fn configure_atlas(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::linear();
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
            TextureAtlasPlugin,
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
