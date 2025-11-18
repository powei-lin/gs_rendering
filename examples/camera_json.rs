use std::fs;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CameraWithPose {
    id: u32,
    img_name: String,
    width: u32,
    height: u32,
    position: [f32; 3],
    rotation: [[f32; 3]; 3],
    fx: f32,
    fy: f32,
}
impl CameraWithPose {
    pub fn get_transform(&self) -> Transform {
        let rotation = Quat::from_mat3(&mat3(
            Vec3 {
                x: self.rotation[0][0],
                y: self.rotation[0][1],
                z: self.rotation[0][2],
            },
            Vec3 {
                x: self.rotation[1][0],
                y: self.rotation[1][1],
                z: self.rotation[1][2],
            },
            Vec3 {
                x: self.rotation[2][0],
                y: self.rotation[2][1],
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
fn main() {
    // 讀取檔案內容
    let data = fs::read_to_string("cameras.json").unwrap();

    // 解析 JSON
    let config: Vec<CameraWithPose> = serde_json::from_str(&data).unwrap();
    for (i, c) in config.iter().enumerate() {
        println!("{} {} {}", i, c.id, c.img_name);
        // println!("{:?}", c.get_transform());
    }
}
