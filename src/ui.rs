use crate::state::*;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::{TextureDimension, TextureFormat};

pub fn init(width: usize, height: usize) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(MapState::new(width, height))
        .add_systems(Startup, setup)
        .add_systems(Update, update_image)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, map_state: Res<MapState>) {
    let image = Image::new_fill(
        Extent3d {
            width: map_state.width as u32,
            height: map_state.height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &Srgba::new(0.5, 0.5, 0.5, 1.0).to_u8_array(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let image_handle = images.add(image).clone();

    commands.insert_resource(ImageHandle(image_handle.clone()));
    commands.insert_resource(GameTime { time: 0.0 });

    commands.spawn(Sprite::from_image(image_handle));

    commands.spawn(Camera2d);
}

fn update_image(
    time: Res<Time>,
    mut images: ResMut<Assets<Image>>,
    image_handle: Res<ImageHandle>,
    mut game_time: ResMut<GameTime>,
    map_state: Res<MapState>,
) {
    game_time.time += time.delta_secs();
    let image = images.get_mut(&image_handle.0).unwrap();

    map_state.update_image(image, game_time.time);
}
