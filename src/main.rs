use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::{TextureDimension, TextureFormat};

mod terrain;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_image)
        .run();
}

#[derive(Resource)]
struct ImageHandle(Handle<Image>);

#[derive(Resource)]
struct GameTime {
    time: f32,
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
            let pixel = image.pixel_bytes_mut(UVec3::new(i, j, 0)).unwrap();
            pixel[0] = i as u8;
            pixel[1] = j as u8;
        }
    }

    let image_handle = images.add(image).clone();

    commands.insert_resource(ImageHandle(image_handle.clone()));
    commands.insert_resource(GameTime{time: 0.0});

    commands.spawn(Sprite::from_image(image_handle));

    commands.spawn(Camera2d);
}

fn update_image(
    time: Res<Time>,
    mut images: ResMut<Assets<Image>>,
    image_handle: Res<ImageHandle>,
    mut game_time: ResMut<GameTime>,
) {
    game_time.time += time.delta_secs();
    let image = images.get_mut(&image_handle.0).unwrap();

    for i in 0..256 {
        for j in 0..256 {
            let pixel = image.pixel_bytes_mut(UVec3::new(i, j, 0)).unwrap();
            let shift = ((game_time.time / 5.0).fract() * 256.0) as u32;
            pixel[0] = ((i + shift) % 256) as u8;
            pixel[1] = ((j + shift) % 256) as u8;
        }
    }
}
