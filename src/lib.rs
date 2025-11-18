pub mod asset_tracking;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraWithPose {
    pub id: u32,
    pub img_name: String,
    pub width: u32,
    pub height: u32,
    pub position: [f32; 3],
    pub rotation: [[f32; 3]; 3],
    pub fx: f32,
    pub fy: f32,
}
impl CameraWithPose {
    pub fn get_transform(&self) -> Transform {
        let rotation = Quat::from_mat3(&mat3(
            Vec3 {
                x: self.rotation[0][0],
                y: self.rotation[1][0],
                z: self.rotation[2][0],
            },
            Vec3 {
                x: self.rotation[0][1],
                y: self.rotation[1][1],
                z: self.rotation[2][1],
            },
            Vec3 {
                x: self.rotation[0][2],
                y: self.rotation[1][2],
                z: self.rotation[2][2],
            },
        ));
        let translation = Vec3::from_array(self.position);
        let scale = Vec3::splat(1.0);
        Transform {
            rotation,
            translation,
            scale,
        }
    }
}
