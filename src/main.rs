use std::{env, f32::consts::FRAC_PI_2, fs};

use bevy::{camera::primitives::Aabb, core_pipeline::tonemapping::Tonemapping, prelude::*};
use bevy_gaussian_splatting::{
    CloudSettings, GaussianCamera, GaussianSplattingPlugin, PlanarGaussian3d,
    PlanarGaussian3dHandle,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use clap::Parser;
use gs_rendering::CameraWithPose;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// ply file path
    file: String,

    /// camera json
    cameras: Option<String>,
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

    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: current_dir_string,
            unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
            ..Default::default()
        }))
        .add_plugins(GaussianSplattingPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(RenderingConfig {
            file_path: cli.file,
            cameras,
        })
        .add_systems(Startup, setup_gaussian_cloud)
        .add_systems(Update, draw_axes)
        .run();
}

#[derive(Debug, Resource)]
struct RenderingConfig {
    pub file_path: String,
    pub cameras: Vec<CameraWithPose>,
}

#[derive(Component)]
struct ShowAxes;

fn draw_axes(mut gizmos: Gizmos, query: Query<(&Transform, &Aabb), With<ShowAxes>>) {
    for (&transform, &aabb) in &query {
        let length = aabb.half_extents.length();
        gizmos.axes(transform, length);
    }
}

fn setup_gaussian_cloud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rendering_config: Res<RenderingConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cloud: Handle<PlanarGaussian3d> = asset_server.load(&rendering_config.file_path);
    let rotation = Quat::from_rotation_y(FRAC_PI_2) * Quat::from_rotation_x(-FRAC_PI_2);
    commands.spawn((
        PlanarGaussian3dHandle(cloud.clone()),
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
