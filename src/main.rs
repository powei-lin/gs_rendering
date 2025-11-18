use std::{env, f32::consts::FRAC_PI_2, fs};

use bevy::{
    app, asset::LoadedUntypedAsset, camera::primitives::Aabb,
    core_pipeline::tonemapping::Tonemapping, prelude::*,
};
use bevy_gaussian_splatting::{
    CloudSettings, GaussianCamera, GaussianSplattingPlugin, PlanarGaussian3d,
    PlanarGaussian3dHandle,
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
    .add_plugins(PanOrbitCameraPlugin)
    .insert_resource(RenderingConfig {
        file_path: cli.file,
        cameras,
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
        .add_systems(Update, draw_axes.run_if(in_state(GameState::InGame)))
        .run();
}

#[derive(Debug, Resource)]
struct RenderingConfig {
    pub file_path: String,
    pub cameras: Vec<CameraWithPose>,
}

fn all_assets_loaded(resource_handels: Res<ResourceHandles>) -> bool {
    resource_handels.is_all_done()
}
fn start_loading() {
    println!("start");
}
fn end_loading() {
    println!("end loading");
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    cloud: Handle<PlanarGaussian3d>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let rendering_config = world.resource::<RenderingConfig>();
        Self {
            cloud: asset_server.load(&rendering_config.file_path),
        }
    }
}

#[derive(Component)]
struct ShowAxes;

fn draw_axes(mut gizmos: Gizmos, query: Query<(&Transform, &Aabb), With<ShowAxes>>) {
    for (&transform, &aabb) in &query {
        let length = aabb.half_extents.length();
        gizmos.axes(transform, length);
    }
}

fn enter_game_play(mut stage: ResMut<NextState<GameState>>) {
    stage.set(GameState::InGame);
}

fn setup_gaussian_cloud(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    level_asset: Res<LevelAssets>,
    rendering_config: Res<RenderingConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let cloud: Handle<PlanarGaussian3d> = asset_server.load(&rendering_config.file_path);
    let rotation = Quat::from_rotation_y(FRAC_PI_2) * Quat::from_rotation_x(-FRAC_PI_2);
    commands.spawn((
        PlanarGaussian3dHandle(level_asset.cloud.clone()),
        CloudSettings {
            gaussian_mode: bevy_gaussian_splatting::GaussianMode::Gaussian2d,
            ..Default::default()
        },
        // Transform::from_rotation(rotation),
    ));
    for c in &rendering_config.cameras {
        if !c.img_name.contains("FRONT") {
            continue;
        }
        let transform = c.get_transform();
        let s = 0.1;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(s, s, s))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            ShowAxes,
            transform,
        ));
    }
    commands.spawn((
        GaussianCamera { warmup: true },
        Transform::from_translation(Vec3::new(0.0, 15.0, 50.0)),
        PanOrbitCamera::default(),
        Tonemapping::None,
    ));
}
