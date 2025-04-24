use crate::terrain::height_map;
use bevy::prelude::*;
use std::collections::HashSet;

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
    is_water: Vec<Vec<bool>>,
    min_height: f32,
    max_height: f32,
}

impl MapState {
    pub fn new(width: usize, height: usize) -> Self {
        let height_map = height_map(width, height);
        let mut points_with_height = Vec::new();
        let mut lake_id: Vec<Vec<usize>> = vec![vec![0; width]; height];
        let mut next_lake_id = 1;
        for i in 0..width {
            for j in 0..height {
                points_with_height.push((height_map[j][i], j, i));
            }
        }
        points_with_height.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut is_water = vec![vec![false; width]; height];
        let min_height = height_map
            .iter()
            .flatten()
            .cloned()
            .reduce(f32::min)
            .unwrap();
        let max_height = height_map
            .iter()
            .flatten()
            .cloned()
            .reduce(f32::max)
            .unwrap();
        MapState {
            width,
            height,
            height_map,
            min_height,
            max_height,
            is_water,
        }
    }

    pub fn render_image(&self, image: &mut Image) {
        for i in 0..self.width {
            for j in 0..self.height {
                let pixel = image
                    .pixel_bytes_mut(UVec3::new(i as u32, j as u32, 0))
                    .unwrap();
                let level = |x: f32| (x * 30.0).floor();
                if i + 1 < self.width
                    && j + 1 < self.height
                    && (level(self.height_map[j][i]) != level(self.height_map[j + 1][i])
                        || level(self.height_map[j][i]) != level(self.height_map[j][i + 1]))
                {
                    pixel[0] = 255;
                    pixel[1] = 255;
                    pixel[2] = 255;
                } else if self.is_water[j][i] {
                    pixel[0] = 64;
                    pixel[1] = 64;
                } else {
                    // use std::f32::consts::PI;
                    // let value = 2.0
                    //     * PI
                    //     * ((self.height_map[j][i] - self.min_height) / (self.max_height - self.min_height));
                    // pixel[0] = ((value.sin() + 1.0) * 127.5) as u8;
                    // pixel[1] = (((value + 2.0 * PI / 3.0).sin() + 1.0) * 127.5) as u8;
                    // pixel[2] = (((value + 4.0 * PI / 3.0).sin() + 1.0) * 127.5) as u8;

                    let value = (self.height_map[j][i] - self.min_height)
                        / (self.max_height - self.min_height);
                    pixel[0] = (value * 255.0) as u8;
                    pixel[1] = (value * 255.0) as u8;
                    pixel[2] = (value * 255.0) as u8;
                }
            }
        }
    }
}
