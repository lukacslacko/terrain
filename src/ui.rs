use crate::dijkstra::{self, DijkstraCommand, DijkstraUpdate};
use crate::state::*;

use bevy::asset::RenderAssetUsages;
use bevy::input::common_conditions::*;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::{TextureDimension, TextureFormat};
use bevy::render::view::window;
use bevy::state::state;
use bevy::window::SystemCursorIcon;
use bevy::winit::cursor::CursorIcon;
use crossbeam_channel::{Receiver, Sender, bounded};

pub fn init(width: usize, height: usize) {
    App::new()
        .add_event::<DijkstraEvent>()
        .add_plugins(DefaultPlugins)
        .insert_resource(MapState::new(width, height))
        .add_systems(Startup, setup)
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
        .add_systems(FixedUpdate, read_dijkstra_stream)
        .add_systems(Update, process_dijkstra_updates)
        .add_systems(
            Update,
            on_mouse_left_click.run_if(input_just_pressed(MouseButton::Left)),
        )
        .add_systems(
            Update,
            on_mouse_right_click.run_if(input_just_pressed(MouseButton::Right)),
        )
        .run();
}

#[derive(Component)]
struct MainCamera;

#[derive(Resource, Deref)]
struct DijkstraReceiver(Receiver<DijkstraUpdate>);

#[derive(Event)]
struct DijkstraEvent(DijkstraUpdate);

#[derive(Resource)]
struct DijkstraCommandHolder(DijkstraCommand);

#[derive(Resource)]
struct DijkstraCommandSender(Sender<DijkstraCommand>);

fn read_dijkstra_stream(
    dijkstra_receiver: Res<DijkstraReceiver>,
    mut event_writer: EventWriter<DijkstraEvent>,
) {
    if let Ok(path) = dijkstra_receiver.try_recv() {
        event_writer.send(DijkstraEvent(path));
    }
}

fn process_dijkstra_updates(
    mut commands: Commands,
    mut event_reader: EventReader<DijkstraEvent>,
    image_handle: Res<ImageHandle>,
    mut images: ResMut<Assets<Image>>,
    win_entity: Single<Entity, With<Window>>,
    mut map_state: ResMut<MapState>,
) {
    let image = images.get_mut(&image_handle.0).unwrap();
    for event in event_reader.read() {
        commands
            .entity(*win_entity)
            .insert(CursorIcon::from(SystemCursorIcon::Default));
        map_state.process_dijsktra_update(&event.0, image);
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, map_state: Res<MapState>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: map_state.dijkstra.width as u32,
            height: map_state.dijkstra.height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &Srgba::new(0.5, 0.5, 0.5, 1.0).to_u8_array(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    map_state.render_image(&mut image);

    let image_handle = images.add(image).clone();

    let (tx, rx) = bounded::<DijkstraUpdate>(1);
    let (tx_command, rx_command) = bounded::<DijkstraCommand>(1);
    commands.insert_resource(DijkstraCommandSender(tx_command));
    commands.insert_resource(DijkstraCommandHolder(DijkstraCommand {
        a: (0, 0),
        b: (0, 0),
    }));
    let mut other_dijkstra = map_state.dijkstra.clone();
    std::thread::spawn(move || {
        // other_dijkstra.connect_randoms_forever(tx);
        other_dijkstra.connect_selected(&rx_command, tx);
    });
    commands.insert_resource(DijkstraReceiver(rx));

    commands.insert_resource(ImageHandle(image_handle.clone()));
    commands.insert_resource(GameTime { _time: 0.0 });

    commands.spawn(Sprite::from_image(image_handle));

    commands.spawn((Camera2d, MainCamera));
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

fn on_mouse_left_click(
    query: Query<(&GlobalTransform, &Camera, &MainCamera)>,
    windows: Query<&Window>,
    state: Res<MapState>,
    mut dijkstra_command_holder: ResMut<DijkstraCommandHolder>,
) {
    let (global_transform, camera, _) = query.single();
    let window = windows.single();
    let cursor_position = window.cursor_position().unwrap();
    if let Ok(ray) = camera.viewport_to_world_2d(global_transform, cursor_position) {
        let x = (ray.x + state.dijkstra.width as f32 / 2.0)
            .clamp(0.0, state.dijkstra.width as f32 - 1.0) as usize;
        let y = (-ray.y + state.dijkstra.height as f32 / 2.0)
            .clamp(0.0, state.dijkstra.height as f32 - 1.0) as usize;
        let station = state.near_station(y, x).unwrap_or((y, x));
        if state.dijkstra.is_water[station.0][station.1] {
            return;
        }
        dijkstra_command_holder.0.a = station;
    }
}

fn on_mouse_right_click(
    mut commands: Commands,
    query: Query<(&GlobalTransform, &Camera, &MainCamera)>,
    windows: Query<&Window>,
    win_entity: Single<Entity, With<Window>>,
    state: Res<MapState>,
    mut dijkstra_command_holder: ResMut<DijkstraCommandHolder>,
    dijkstra_command_sender: Res<DijkstraCommandSender>,
) {
    let (global_transform, camera, _) = query.single();
    let window = windows.single();
    let cursor_position = window.cursor_position().unwrap();
    if let Ok(ray) = camera.viewport_to_world_2d(global_transform, cursor_position) {
        let x = (ray.x + state.dijkstra.width as f32 / 2.0)
            .clamp(0.0, state.dijkstra.width as f32 - 1.0) as usize;
        let y = (-ray.y + state.dijkstra.height as f32 / 2.0)
            .clamp(0.0, state.dijkstra.height as f32 - 1.0) as usize;
        let station = state.near_station(y, x).unwrap_or((y, x));
        if state.dijkstra.is_water[station.0][station.1] {
            return;
        }
        dijkstra_command_holder.0.b = station;
    }
    let _ = dijkstra_command_sender
        .0
        .send(dijkstra_command_holder.0.clone())
        .unwrap();
    commands
        .entity(*win_entity)
        .insert(CursorIcon::from(SystemCursorIcon::Progress));
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
