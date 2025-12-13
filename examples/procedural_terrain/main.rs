//! Procedural terrain generation example
//!
//! Demonstrates how to use bevy_map_autotile to generate terrain procedurally.
//!
//! Run with: cargo run --example procedural_terrain -p bevy_map_editor_examples

use bevy::prelude::*;
use bevy_map_autotile::{Color as AutotileColor, TerrainSet, TerrainSetType};
use bevy_map_core::{Layer, LayerData, Level, MapProject, Tileset};
use bevy_map_runtime::{MapRuntimePlugin, SpawnMapProjectEvent, TilesetTextures};
use std::collections::HashMap;
use uuid::Uuid;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy_map_editor - Procedural Terrain Example".to_string(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MapRuntimePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_controls)
        .add_systems(Update, regenerate_terrain)
        .run();
}

/// Resource to track terrain generation state
#[derive(Resource)]
struct TerrainState {
    tileset_id: Uuid,
    seed: u64,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_events: bevy::ecs::message::MessageWriter<SpawnMapProjectEvent>,
) {
    // Spawn camera
    commands.spawn((Camera2d, Transform::from_xyz(256.0, 256.0, 0.0)));

    // Create a tileset for procedural generation
    let tileset_id = Uuid::new_v4();
    let tileset = Tileset::new(
        "procedural_tiles".to_string(),
        "tiles/example_tileset.png".to_string(),
        32,
        4,
        4,
    );

    // Generate initial terrain
    let level = generate_terrain(32, 32, tileset_id, 42);

    // Create project
    let mut tilesets = HashMap::new();
    tilesets.insert(tileset_id, tileset);
    let project = MapProject {
        level,
        tilesets,
        version: 1,
    };

    // Load tileset texture
    let mut textures = TilesetTextures::new();
    textures.load_from_project(&project, &asset_server);

    // Spawn the map
    spawn_events.write(SpawnMapProjectEvent {
        project,
        textures,
        transform: Transform::default(),
    });

    // Store state for regeneration
    commands.insert_resource(TerrainState {
        tileset_id,
        seed: 42,
    });

    info!("Procedural terrain generated!");
    info!("Press R to regenerate with a new seed");
}

/// Generate a procedural terrain level using noise
fn generate_terrain(width: u32, height: u32, tileset_id: Uuid, seed: u64) -> Level {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut level = Level::new("Procedural Level".to_string(), width, height);

    // Create a tile layer
    let mut layer = Layer::new_tile_layer("Ground".to_string(), tileset_id, width, height);

    // Generate tiles using simple noise
    if let LayerData::Tiles { ref mut tiles, .. } = layer.data {
        for y in 0..height {
            for x in 0..width {
                // Simple noise function using hash
                let mut hasher = DefaultHasher::new();
                (x, y, seed).hash(&mut hasher);
                let hash = hasher.finish();

                // Choose tile based on noise value
                let noise = (hash % 1000) as f32 / 1000.0;
                let tile_index = if noise < 0.3 {
                    0 // Water/deep
                } else if noise < 0.5 {
                    1 // Sand/shore
                } else if noise < 0.8 {
                    2 // Grass
                } else {
                    3 // Forest/dense
                };

                let idx = (y * width + x) as usize;
                tiles[idx] = Some(tile_index);
            }
        }
    }

    level.layers.push(layer);
    level
}

/// Generate terrain with autotile transitions (placeholder for demonstration)
#[allow(dead_code)]
fn generate_terrain_with_autotile(
    width: u32,
    height: u32,
    tileset_id: Uuid,
    seed: u64,
) -> Level {
    // Create a terrain set for smooth transitions
    let mut terrain_set = TerrainSet::new("Ground Transitions".to_string(), tileset_id, TerrainSetType::Corner);

    // Add terrain types with colors
    terrain_set.add_terrain("water".to_string(), AutotileColor::rgb(0.0, 0.0, 1.0));
    terrain_set.add_terrain("grass".to_string(), AutotileColor::rgb(0.0, 1.0, 0.0));

    // Generate base terrain data
    let mut terrain_map = vec![0u8; (width * height) as usize];

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    for y in 0..height {
        for x in 0..width {
            let mut hasher = DefaultHasher::new();
            (x, y, seed).hash(&mut hasher);
            let noise = (hasher.finish() % 100) as f32 / 100.0;

            let idx = (y * width + x) as usize;
            terrain_map[idx] = if noise < 0.4 { 0 } else { 1 };
        }
    }

    // Create level with terrain data
    let mut level = Level::new("Autotiled Level".to_string(), width, height);
    let mut layer = Layer::new_tile_layer("Ground".to_string(), tileset_id, width, height);

    // For now, use basic tiles (full autotile would require Wang tile lookup)
    if let LayerData::Tiles { ref mut tiles, .. } = layer.data {
        for (idx, &terrain) in terrain_map.iter().enumerate() {
            tiles[idx] = Some(terrain as u32);
        }
    }

    level.layers.push(layer);
    level
}

/// Simple camera controls
fn camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut projection)) = query.single_mut() else {
        return;
    };

    let speed = 200.0 * time.delta_secs();

    if keyboard.pressed(KeyCode::KeyW) {
        transform.translation.y += speed;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        transform.translation.y -= speed;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        transform.translation.x -= speed;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        transform.translation.x += speed;
    }

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

/// Regenerate terrain on key press
fn regenerate_terrain(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<TerrainState>,
    asset_server: Res<AssetServer>,
    mut spawn_events: bevy::ecs::message::MessageWriter<SpawnMapProjectEvent>,
    map_query: Query<Entity, With<bevy_map_runtime::RuntimeMap>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        // Increment seed
        state.seed = state.seed.wrapping_add(1);
        info!("Regenerating terrain with seed: {}", state.seed);

        // Despawn existing map
        for entity in map_query.iter() {
            commands.entity(entity).despawn();
        }

        // Generate new terrain
        let tileset_id = state.tileset_id;
        let tileset = Tileset::new(
            "procedural_tiles".to_string(),
            "tiles/example_tileset.png".to_string(),
            32,
            4,
            4,
        );

        let level = generate_terrain(32, 32, tileset_id, state.seed);

        let mut tilesets = HashMap::new();
        tilesets.insert(tileset_id, tileset);
        let project = MapProject {
            level,
            tilesets,
            version: 1,
        };

        let mut textures = TilesetTextures::new();
        textures.load_from_project(&project, &asset_server);

        spawn_events.write(SpawnMapProjectEvent {
            project,
            textures,
            transform: Transform::default(),
        });
    }
}
