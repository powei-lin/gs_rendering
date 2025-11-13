use std::{env, f32::consts::FRAC_PI_2};

use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};
use bevy_gaussian_splatting::{
    CloudSettings, GaussianCamera, GaussianSplattingPlugin, PlanarGaussian3d,
    PlanarGaussian3dHandle,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// ply file path
    file: String,
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
        })
        .add_systems(Startup, setup_gaussian_cloud)
        .run();
}

#[derive(Debug, Resource)]
struct RenderingConfig {
    pub file_path: String,
}

fn setup_gaussian_cloud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rendering_config: Res<RenderingConfig>,
) {
    let cloud: Handle<PlanarGaussian3d> = asset_server.load(&rendering_config.file_path);
    let rotation = Quat::from_rotation_y(FRAC_PI_2) * Quat::from_rotation_x(-FRAC_PI_2);
    commands.spawn((
        PlanarGaussian3dHandle(cloud.clone()),
        CloudSettings {
            gaussian_mode: bevy_gaussian_splatting::GaussianMode::Gaussian2d,
            ..Default::default()
        },
        Transform::from_rotation(rotation),
    ));
    commands.spawn((
        GaussianCamera { warmup: true },
        Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        PanOrbitCamera::default(),
        Tonemapping::None,
    ));
}
