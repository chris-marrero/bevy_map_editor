//! Example demonstrating hot-reload workflow with bevy_map_editor
//!
//! This example shows the recommended way to load maps in your game:
//! - Maps are loaded as Bevy assets via AssetServer
//! - Hot-reload is supported when using the `file_watcher` feature
//!
//! # Running with Hot-Reload
//!
//! ```bash
//! # Run with hot-reload enabled (recommended for development)
//! cargo run --example hot_reload --features bevy/file_watcher
//!
//! # Or just run normally (no hot-reload)
//! cargo run --example hot_reload
//! ```
//!
//! # Workflow
//!
//! 1. Start this example
//! 2. Open the map file in bevy_map_editor
//! 3. Make changes and save
//! 4. Watch the game update automatically!

use bevy::prelude::*;
use bevy_map_runtime::{MapHandle, MapRuntimePlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy_map_editor - Hot Reload Example".to_string(),
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera
    commands.spawn((Camera2d, Transform::from_xyz(64.0, 64.0, 0.0)));

    // Load map via asset system - supports hot-reload!
    // The map will automatically spawn once loaded
    commands.spawn((
        MapHandle(asset_server.load("maps/example_project.map.json")),
        Transform::default(),
        Visibility::default(),
    ));

    info!("Loading map... Edit the .map.json file while running to see hot-reload!");
    info!("Controls: WASD/Arrows to pan, Q/E to zoom");
}

/// Simple camera controls: WASD to pan, Q/E to zoom
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
