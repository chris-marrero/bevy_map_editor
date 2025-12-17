//! Collision spawning systems for Avian physics integration
//!
//! This module provides systems to spawn Avian2D colliders for tiles and entities
//! based on collision data defined in the editor.
//!
//! # Features
//!
//! - Automatic collider spawning for tiles with collision shapes
//! - Support for all collision shapes (Full, Rectangle, Circle, Polygon)
//! - One-way platform support
//! - Collision layers and masks
//!
//! # Usage
//!
//! Enable the `physics` feature in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! bevy_map_runtime = { version = "0.1", features = ["physics"] }
//! ```
//!
//! Then add the `MapCollisionPlugin` to your app:
//!
//! ```rust,ignore
//! use bevy_map_runtime::collision::MapCollisionPlugin;
//!
//! app.add_plugins(MapCollisionPlugin);
//! ```

use bevy::prelude::*;
use bevy_map_core::CollisionData;

#[cfg(feature = "physics")]
use bevy_map_core::{CollisionShape, OneWayDirection, PhysicsBody};

#[cfg(feature = "physics")]
use avian2d::prelude::*;

/// Plugin that provides collision spawning systems
///
/// This plugin automatically spawns Avian2D colliders for tiles with
/// collision data when maps are loaded.
#[cfg(feature = "physics")]
pub struct MapCollisionPlugin;

#[cfg(feature = "physics")]
impl Plugin for MapCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(avian2d::PhysicsPlugins::default())
            .add_systems(Update, spawn_tile_colliders);
    }
}

/// Marker component for map collision entities
#[derive(Component)]
pub struct MapCollider {
    /// The original collision data from the editor
    pub data: CollisionData,
}

/// Marker component for one-way platforms
#[cfg(feature = "physics")]
#[derive(Component)]
pub struct OneWayPlatform {
    /// Direction that allows pass-through
    pub direction: OneWayDirection,
}

/// System to spawn tile colliders after map load
///
/// This system runs when a `MapRoot` component is added and spawns
/// colliders for all tiles that have collision data.
#[cfg(feature = "physics")]
pub fn spawn_tile_colliders(
    mut commands: Commands,
    map_query: Query<(Entity, &super::MapRoot), Added<super::MapRoot>>,
    map_assets: Res<Assets<bevy_map_core::MapProject>>,
) {
    for (map_entity, map_root) in map_query.iter() {
        let Some(project) = map_assets.get(&map_root.handle) else {
            continue;
        };

        let tile_size = map_root.textures.tile_size;
        let level = &project.level;

        // Iterate through all tile layers
        for (_layer_index, layer) in level.layers.iter().enumerate() {
            if let bevy_map_core::LayerData::Tiles {
                tileset_id, tiles, ..
            } = &layer.data
            {
                // Get the tileset to look up collision data
                let Some(tileset) = project.get_tileset(*tileset_id) else {
                    continue;
                };

                // Spawn colliders for each tile with collision
                for y in 0..level.height {
                    for x in 0..level.width {
                        let idx = (y * level.width + x) as usize;
                        if let Some(&Some(tile_index)) = tiles.get(idx) {
                            // Check if this tile has collision
                            if let Some(props) = tileset.get_tile_properties(tile_index) {
                                if props.collision.has_collision() {
                                    spawn_collider_for_tile(
                                        &mut commands,
                                        map_entity,
                                        &props.collision,
                                        x,
                                        y,
                                        tile_size,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Spawn a collider entity for a single tile
#[cfg(feature = "physics")]
fn spawn_collider_for_tile(
    commands: &mut Commands,
    map_entity: Entity,
    collision: &CollisionData,
    tile_x: u32,
    tile_y: u32,
    tile_size: f32,
) {
    let Some(collider) = shape_to_collider(&collision.shape, tile_size) else {
        return;
    };

    // Calculate world position (bevy_ecs_tilemap uses bottom-left origin)
    // Tiles are rendered at their center, so add half tile size
    let world_x = tile_x as f32 * tile_size + tile_size / 2.0;
    let world_y = tile_y as f32 * tile_size + tile_size / 2.0;

    // Apply offset from collision shape
    let (offset_x, offset_y) = get_shape_offset(&collision.shape, tile_size);

    // Add collision layers using bitmasks
    // membership: the layer this collider belongs to (1 << layer)
    // filter: which layers this collider interacts with (mask)
    let membership = 1u32 << collision.layer;

    let collider_entity = commands
        .spawn((
            Transform::from_xyz(world_x + offset_x, world_y + offset_y, 0.0),
            Visibility::default(),
            body_type_to_rigid_body(collision.body_type),
            collider,
            CollisionLayers::from_bits(membership, collision.mask),
            MapCollider {
                data: collision.clone(),
            },
        ))
        .id();

    // Add one-way marker if applicable
    if collision.one_way.is_one_way() {
        commands.entity(collider_entity).insert(OneWayPlatform {
            direction: collision.one_way,
        });
    }

    // Make it a child of the map
    commands.entity(map_entity).add_child(collider_entity);
}

/// Convert CollisionShape to Avian Collider
#[cfg(feature = "physics")]
fn shape_to_collider(shape: &CollisionShape, tile_size: f32) -> Option<Collider> {
    match shape {
        CollisionShape::None => None,
        CollisionShape::Full => Some(Collider::rectangle(tile_size, tile_size)),
        CollisionShape::Rectangle { size, .. } => {
            Some(Collider::rectangle(size[0] * tile_size, size[1] * tile_size))
        }
        CollisionShape::Circle { radius, .. } => Some(Collider::circle(*radius * tile_size)),
        CollisionShape::Polygon { points } => {
            if points.len() < 3 {
                return None;
            }
            let scaled: Vec<Vec2> = points
                .iter()
                .map(|p| Vec2::new((p[0] - 0.5) * tile_size, (p[1] - 0.5) * tile_size))
                .collect();
            Collider::convex_hull(scaled)
        }
    }
}

/// Get the offset from collision shape (for Rectangle and Circle)
#[cfg(feature = "physics")]
fn get_shape_offset(shape: &CollisionShape, tile_size: f32) -> (f32, f32) {
    match shape {
        CollisionShape::Rectangle { offset, .. } => {
            (offset[0] * tile_size, offset[1] * tile_size)
        }
        CollisionShape::Circle { offset, .. } => (offset[0] * tile_size, offset[1] * tile_size),
        _ => (0.0, 0.0),
    }
}

/// Convert PhysicsBody to Avian RigidBody
#[cfg(feature = "physics")]
fn body_type_to_rigid_body(body: PhysicsBody) -> RigidBody {
    match body {
        PhysicsBody::Static => RigidBody::Static,
        PhysicsBody::Dynamic => RigidBody::Dynamic,
        PhysicsBody::Kinematic => RigidBody::Kinematic,
    }
}

// Non-physics stub implementations for when the feature is disabled
#[cfg(not(feature = "physics"))]
pub struct MapCollisionPlugin;

#[cfg(not(feature = "physics"))]
impl Plugin for MapCollisionPlugin {
    fn build(&self, _app: &mut App) {
        // No-op when physics feature is disabled
    }
}
