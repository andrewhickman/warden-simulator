use bevy::{camera::ScalingMode, prelude::*};

#[derive(Resource, Default)]
enum CardinalView {
    #[default]
    South,
    East,
    North,
    West,
}

impl CardinalView {
    fn next(&self) -> Self {
        match self {
            Self::South => Self::East,
            Self::East => Self::North,
            Self::North => Self::West,
            Self::West => Self::South,
        }
    }

    fn scene_yaw(&self) -> f32 {
        match self {
            Self::South => 0.0,
            Self::East => std::f32::consts::FRAC_PI_2,
            Self::North => std::f32::consts::PI,
            Self::West => -std::f32::consts::FRAC_PI_2,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<CardinalView>()
        .add_systems(Startup, (spawn_frustum_camera, setup))
        .add_systems(Update, cycle_scene_view)
        .run();
}

#[derive(Component)]
struct SceneRoot;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GlobalAmbientLight {
        brightness: 300.0,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        SceneRoot,
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("model/Walls2.glb"))),
    ));
}

const FRUSTUM_HEIGHT: f32 = 2.0;
const TOP_WIDTH: f32 = 0.641_473_4;
const VIEW_ANGLE_DEGREES: f32 = 76.987_12;

fn spawn_frustum_camera(mut commands: Commands) {
    let target = Vec3::new(0.0, FRUSTUM_HEIGHT * 0.5, 0.0);

    let angle = VIEW_ANGLE_DEGREES.to_radians();

    // Orthographic camera distance does not affect apparent size.
    // It only needs to be far enough away to avoid clipping.
    let camera_distance = 10.0;

    // Camera is above the target and in front of the +Z-facing front face.
    let camera_position = target + Vec3::new(0.0, angle.sin(), angle.cos()) * camera_distance;

    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            // Controls how many world-space units fit vertically onscreen.
            // Adjust this for framing without changing the projection geometry.
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 3.5,
            },

            near: 0.01,
            far: 100.0,

            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(camera_position).looking_at(target, Vec3::Y),
    ));
}

fn cycle_scene_view(
    mut scene: Query<&mut Transform, With<SceneRoot>>,
    mut view: ResMut<CardinalView>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        let next = view.next();
        *view = next;

        if let Ok(mut transform) = scene.single_mut() {
            transform.rotation = Quat::from_rotation_y(view.scene_yaw());
        }
    }
}
