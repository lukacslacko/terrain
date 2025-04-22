use crate::terrain::height_map;
use bevy::prelude::*;

#[derive(Resource)]
pub struct ImageHandle(pub Handle<Image>);

#[derive(Resource)]
pub struct GameTime {
    pub time: f32,
}

#[derive(Resource)]
pub struct MapState {
    pub width: usize,
    pub height: usize,
    pub height_map: Vec<Vec<f32>>,
}

impl MapState {
    pub fn new(width: usize, height: usize) -> Self {
        MapState {
            width,
            height,
            height_map: height_map(width, height),
        }
    }

    pub fn update_image(&self, image: &mut Image) {
        let min_height = self
            .height_map
            .iter()
            .flatten()
            .cloned()
            .reduce(f32::min)
            .unwrap();
        let max_height = self
            .height_map
            .iter()
            .flatten()
            .cloned()
            .reduce(f32::max)
            .unwrap();
        for i in 0..self.width {
            for j in 0..self.height {
                let pixel = image
                    .pixel_bytes_mut(UVec3::new(i as u32, j as u32, 0))
                    .unwrap();
                let level = |x: f32| (x * 10.0).floor();
                if i + 1 < self.width
                    && j + 1 < self.height 
                    && (level(self.height_map[i][j]) != level(self.height_map[i + 1][j])
                        || level(self.height_map[i][j]) != level(self.height_map[i][j + 1]))
                {
                    pixel[0] = 255;
                    pixel[1] = 255;
                    pixel[2] = 255;
                } else {
                    use std::f32::consts::PI;
                    let value = 2.0
                        * PI
                        * ((self.height_map[j][i] - min_height) / (max_height - min_height));
                    pixel[0] = ((value.sin() + 1.0) * 127.5) as u8;
                    pixel[1] = (((value + 2.0 * PI / 3.0).sin() + 1.0) * 127.5) as u8;
                    pixel[2] = (((value + 4.0 * PI / 3.0).sin() + 1.0) * 127.5) as u8;
                }
            }
        }
    }
}
