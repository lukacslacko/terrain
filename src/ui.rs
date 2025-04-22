use crate::state::*;

use bevy::asset::RenderAssetUsages;
use bevy::input::common_conditions::*;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::{TextureDimension, TextureFormat};

pub fn init(width: usize, height: usize) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(MapState::new(width, height))
        .add_systems(Startup, setup)
        .add_systems(Update, update_image)
        .add_systems(Update, pan_camera.run_if(input_pressed(MouseButton::Left)))
        .add_systems(Update, zoom_camera_around_cursor)
        .add_systems(
            Update,
            reset_zoom.run_if(input_just_pressed(KeyCode::Digit0)),
        )
        .add_systems(
            Update,
            zoom_in.run_if(input_just_pressed(KeyCode::NumpadAdd)),
        )
        .add_systems(
            Update,
            zoom_out.run_if(input_just_pressed(KeyCode::NumpadSubtract)),
        )
        .run();
}

#[derive(Component)]
struct MainCamera;

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

    commands.spawn((Camera2d, MainCamera));
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

    map_state.update_image(image);
}

fn pan_camera(
    mut motion_event_reader: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &MainCamera)>,
) {
    let mut transform = query.single_mut();
    for event in motion_event_reader.read() {
        transform.0.translation.x -= event.delta.x * transform.0.scale.x;
        transform.0.translation.y += event.delta.y * transform.0.scale.y;
    }
}

fn zoom_camera_around_cursor(
    mut scroll_event_reader: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, &MainCamera)>,
) {
    let mut transform = query.single_mut();
    for event in scroll_event_reader.read() {
        transform.0.scale.x *= 1.0 + event.y;
        transform.0.scale.y *= 1.0 + event.y;
    }
}

fn reset_zoom(mut query: Query<(&mut Transform, &MainCamera)>) {
    let mut transform = query.single_mut();
    transform.0.scale.x = 1.0;
    transform.0.scale.y = 1.0
}

fn zoom_in(mut query: Query<(&mut Transform, &MainCamera)>) {
    let mut transform = query.single_mut();
    transform.0.scale.x /= 1.1;
    transform.0.scale.y /= 1.1;
}
fn zoom_out(mut query: Query<(&mut Transform, &MainCamera)>) {
    let mut transform = query.single_mut();
    transform.0.scale.x *= 1.1;
    transform.0.scale.y *= 1.1;
}
