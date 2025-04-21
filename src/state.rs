use bevy::prelude::*;

#[derive(Resource)]
pub struct ImageHandle(pub Handle<Image>);

#[derive(Resource)]
pub struct GameTime {
    pub time: f32,
}

#[derive(Resource)]
pub struct MapSize {
    pub width: usize,
    pub height: usize,
}

#[derive(Resource)]
pub struct MapState {
    pub height_map: Vec<Vec<f32>>,
}

impl MapState {
    pub fn new(map_size: &MapSize) -> Self {
        MapState {
            height_map: vec![vec![0.0; map_size.width]; map_size.height],
        }
    }

    pub fn update_image(&self, image: &mut Image, time: f32) {
        for i in 0..256 {
            for j in 0..256 {
                let pixel = image.pixel_bytes_mut(UVec3::new(i, j, 0)).unwrap();
                let shift = ((time / 5.0).fract() * 256.0) as u32;
                pixel[0] = ((i + shift) % 256) as u8;
                pixel[1] = ((j + shift) % 256) as u8;
            }
        }

    }
}
