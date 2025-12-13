//! Example demonstrating how to load and render a map using bevy_map_runtime
//!
//! Run with: cargo run --example load_map

use bevy::prelude::*;
use bevy_map_core::MapProject;
use bevy_map_runtime::{MapRuntimePlugin, SpawnMapProjectEvent, TilesetTextures};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy_map_editor - Load Map Example".to_string(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MapRuntimePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_controls)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_events: bevy::ecs::message::MessageWriter<SpawnMapProjectEvent>,
) {
    // Spawn camera
    commands.spawn((
        Camera2d,
        Transform::from_xyz(64.0, 64.0, 0.0),
    ));

    // Load map project from embedded JSON
    let json = include_str!("../assets/maps/example_project.map.json");
    let project: MapProject = serde_json::from_str(json).expect("Failed to parse map JSON");

    // Load tileset textures
    let mut textures = TilesetTextures::new();
    textures.load_from_project(&project, &asset_server);

    // Send event to spawn the map
    spawn_events.write(SpawnMapProjectEvent {
        project,
        textures,
        transform: Transform::default(),
    });

    info!("Map spawned!");
}

/// Simple camera controls: WASD to pan, scroll to zoom
fn camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut projection)) = query.single_mut() else {
        return;
    };

    let speed = 200.0 * time.delta_secs();

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        transform.translation.y += speed;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= speed;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= speed;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        transform.translation.x += speed;
    }

    // Zoom with Q/E
    if let Projection::Orthographic(ref mut ortho) = *projection {
        if keyboard.pressed(KeyCode::KeyQ) {
            ortho.scale *= 1.0 + time.delta_secs();
        }
        if keyboard.pressed(KeyCode::KeyE) {
            ortho.scale *= 1.0 - time.delta_secs();
        }
        ortho.scale = ortho.scale.clamp(0.25, 4.0);
    }
}
