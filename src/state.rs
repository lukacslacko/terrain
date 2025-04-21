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
            height_map: vec![vec![0.0; width]; height],
        }
    }

    pub fn update_image(&self, image: &mut Image, time: f32) {
        for i in 0..self.width {
            for j in 0..self.height {
                let pixel = image.pixel_bytes_mut(UVec3::new(i as u32, j as u32, 0)).unwrap();
                let shift = ((time / 5.0).fract() * 256.0) as usize;
                pixel[0] = ((i + shift) % 256) as u8;
                pixel[1] = ((j + shift) % 256) as u8;
            }
        }

    }
}
