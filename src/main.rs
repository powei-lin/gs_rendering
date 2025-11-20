use std::{env, f32::consts::FRAC_PI_2, fs};

use bevy::{
    asset::embedded_asset,
    core_pipeline::{Skybox, tonemapping::Tonemapping},
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};
use bevy_gaussian_splatting::{
    CloudSettings, GaussianCamera, GaussianSplattingPlugin, PlanarGaussian3d,
    PlanarGaussian3dHandle,
    sort::{SortConfig, SortTrigger},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use clap::Parser;
use gs_rendering::{
    CameraWithPose,
    asset_tracking::{LoadResource, ResourceHandles},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// ply file path
    file: String,

    /// camera json
    cameras: Option<String>,
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum GameState {
    #[default]
    Loading,
    InGame,
}

struct EmbeddedAssetPlugin;

impl Plugin for EmbeddedAssetPlugin {
    fn build(&self, app: &mut App) {
        // We get to choose some prefix relative to the workspace root which
        // will be ignored in "embedded://" asset paths.
        // Path to asset must be relative to this file, because that's how
        // include_bytes! works.
        embedded_asset!(app, "data/skybox.png");
    }
}

fn main() {
    let cli = Cli::parse();
    let current_dir_pathbuf = env::current_dir().expect("current dir");

    // Convert PathBuf to OsString, then to String
    let current_dir_string = current_dir_pathbuf
        .into_os_string()
        .into_string()
        .map_err(|os_string| format!("Invalid Unicode in path: {:?}", os_string))
        .expect("failed current dir");

    let cameras: Vec<CameraWithPose> = if let Some(cameras_json) = cli.cameras {
        let data = fs::read_to_string(&cameras_json).expect("cameras.json not found");
        serde_json::from_str(&data).expect("fail to deserialize camera json")
    } else {
        Vec::new()
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        file_path: current_dir_string,
        unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
        ..Default::default()
    }))
    .init_state::<GameState>()
    .add_plugins(gs_rendering::asset_tracking::plugin)
    .add_plugins(GaussianSplattingPlugin)
    .add_plugins(EmbeddedAssetPlugin)
    .insert_resource(SortConfig { period_ms: 1 })
    .add_plugins(PanOrbitCameraPlugin)
    .insert_resource(RenderingConfig {
        file_path: cli.file,
        cameras,
        current_camera_idx: 0,
    })
    // .register_type::<LevelAssets>()
    .load_resource::<LevelAssets>();

    app.add_systems(OnEnter(GameState::Loading), start_loading)
        .add_systems(OnExit(GameState::Loading), end_loading)
        .add_systems(
            Update,
            enter_game_play.run_if(in_state(GameState::Loading).and(all_assets_loaded)),
        )
        .add_systems(OnEnter(GameState::InGame), setup_gaussian_cloud)
        // .add_systems(Update, draw_axes.run_if(in_state(GameState::InGame)))
        .add_systems(Update, update_camera.run_if(in_state(GameState::InGame)))
        .run();
}

#[derive(Debug, Resource)]
struct RenderingConfig {
    pub file_path: String,
    pub cameras: Vec<CameraWithPose>,
    pub current_camera_idx: usize,
}

fn all_assets_loaded(resource_handels: Res<ResourceHandles>) -> bool {
    resource_handels.is_all_done()
}
fn start_loading() {
    println!("start loading");
}
fn end_loading() {
    println!("end loading");
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    cloud: Handle<PlanarGaussian3d>,
    #[dependency]
    skybox: Handle<Image>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let rendering_config = world.resource::<RenderingConfig>();
        Self {
            cloud: asset_server.load(&rendering_config.file_path),
            skybox: asset_server.load("embedded://gs_rendering/data/skybox.png"),
        }
    }
}

// #[derive(Component)]
// struct ShowAxes;

// fn draw_axes(mut gizmos: Gizmos, query: Query<(&Transform, &Aabb), With<ShowAxes>>) {
//     for (&transform, &aabb) in &query {
//         let length = aabb.half_extents.length();
//         gizmos.axes(transform, length);
//     }
// }

fn enter_game_play(mut stage: ResMut<NextState<GameState>>) {
    stage.set(GameState::InGame);
}

fn setup_gaussian_cloud(
    mut commands: Commands,
    level_asset: Res<LevelAssets>,
    mut images: ResMut<Assets<Image>>,
    rendering_config: Res<RenderingConfig>,
) {
    let cloud_entity_id = commands
        .spawn((
            PlanarGaussian3dHandle(level_asset.cloud.clone()),
            CloudSettings {
                gaussian_mode: bevy_gaussian_splatting::GaussianMode::Gaussian2d,
                ..Default::default()
            },
        ))
        .id();
    let camera_entity_id = commands
        .spawn((
            GaussianCamera::default(),
            Transform::from_translation(Vec3::new(0.0, 15.0, 50.0)),
            Tonemapping::None,
        ))
        .id();

    // skybox
    let image = images.get_mut(&level_asset.skybox).unwrap();
    if image.texture_descriptor.array_layer_count() == 1 {
        image.reinterpret_stacked_2d_as_array(image.height() / image.width());
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }
    let rotation = Quat::from_rotation_y(FRAC_PI_2) * Quat::from_rotation_x(-FRAC_PI_2);
    // camera
    if rendering_config.cameras.is_empty() {
        commands
            .entity(cloud_entity_id)
            .insert(Transform::from_rotation(rotation));
        commands.entity(camera_entity_id).insert((
            PanOrbitCamera::default(),
            Skybox {
                image: level_asset.skybox.clone(),
                brightness: 1000.0,
                ..default()
            },
        ));
    } else {
        commands.entity(camera_entity_id).insert((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            Skybox {
                image: level_asset.skybox.clone(),
                brightness: 1000.0,
                rotation: rotation.inverse(),
            },
        ));
    }
}

fn update_camera(
    mut query: Query<(&mut Transform, &mut Projection), With<GaussianCamera>>,
    sort_triggers: Query<&SortTrigger>,
    rendering_config: ResMut<RenderingConfig>,
) {
    if rendering_config.cameras.is_empty() {
        return;
    }
    for sort_trigger in sort_triggers {
        if sort_trigger.needs_sort {
            println!("request sort");
            return;
        }
    }
    let (mut transform, mut projection) = query.single_mut().expect("msg");
    println!("update camera idx {}", rendering_config.current_camera_idx);
    let c = &rendering_config.cameras[rendering_config.current_camera_idx];
    *transform = c.get_transform();
    *projection = Projection::Perspective(PerspectiveProjection {
        fov: 2.0 * (0.5 * c.height as f32 / c.fy).atan(),
        ..Default::default()
    });

    let mut rendering_config = rendering_config;
    rendering_config.current_camera_idx =
        (rendering_config.current_camera_idx + 1) % rendering_config.cameras.len();
}
