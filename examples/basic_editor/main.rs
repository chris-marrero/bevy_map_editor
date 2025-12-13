//! Basic editor example
//!
//! Demonstrates how to integrate the bevy_map_editor into your Bevy application.
//!
//! Run with: cargo run --example basic_editor -p bevy_map_editor_examples

use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use bevy_map_editor::EditorPlugin;
use std::path::PathBuf;

fn main() {
    // Determine the assets path - for examples, this is in the examples/ directory
    // When running from workspace root: examples/assets
    let assets_path = get_assets_path();

    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "bevy_map_editor - Basic Editor Example".to_string(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: assets_path.to_string_lossy().to_string(),
                ..default()
            })
        )
        .add_plugins(EditorPlugin::new().with_assets_path(&assets_path))
        .run();
}

/// Determine the correct assets path for this example
fn get_assets_path() -> PathBuf {
    // First, check if we're running from the examples directory
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(&manifest_dir);
        let assets_in_manifest = manifest_path.join("assets");
        if assets_in_manifest.exists() {
            return assets_in_manifest;
        }
    }

    // Check for examples/assets from workspace root
    if let Ok(cwd) = std::env::current_dir() {
        let examples_assets = cwd.join("examples").join("assets");
        if examples_assets.exists() {
            return examples_assets;
        }

        // Fallback to regular assets folder
        let assets = cwd.join("assets");
        if assets.exists() {
            return assets;
        }
    }

    // Last resort
    PathBuf::from("assets")
}
