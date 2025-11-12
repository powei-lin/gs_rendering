use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};
use bevy_gaussian_splatting::{
    CloudSettings, GaussianCamera, GaussianScene, GaussianSceneHandle, GaussianSplattingPlugin,
    PlanarGaussian3d, PlanarGaussian3dHandle,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GaussianSplattingPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup_gaussian_cloud)
        .run();
}

fn setup_gaussian_cloud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut gaussian_3d_assets: ResMut<Assets<PlanarGaussian3d>>,
) {
    let input_uri = "point_cloud2.ply";
    // let input_uri = "icecream.ply";
    // CloudSettings and Visibility are automatically added
    // let cloud = gaussian_3d_assets.add(PlanarGaussian3d::test_model());
    let cloud: Handle<PlanarGaussian3d> = asset_server.load(input_uri);

    commands.spawn((
        PlanarGaussian3dHandle(cloud.clone()),
        CloudSettings {
            gaussian_mode: bevy_gaussian_splatting::GaussianMode::Gaussian2d,
            ..Default::default()
        },
    ));
    commands.spawn((
        GaussianCamera { warmup: true },
        Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        PanOrbitCamera::default(),
        Tonemapping::None,
    ));
}
