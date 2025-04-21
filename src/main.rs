use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::{TextureDimension, TextureFormat};

mod terrain;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &Srgba::new(0.5, 0.5, 0.5, 1.0).to_u8_array(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    for i in 0..256 {
        for j in 0..256 {
            let pixel = image.pixel_bytes_mut(UVec3::new(i,j,0)).unwrap();
            pixel[0] = i as u8;
            pixel[1] = j as u8;
        }
    }

    commands.spawn(Sprite::from_image(images.add(image).clone()));

    commands.spawn(Camera2d);
}
