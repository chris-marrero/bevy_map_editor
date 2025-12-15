//! Rendering utilities for runtime tilemaps
//!
//! This module provides helper functions for working with bevy_ecs_tilemap rendering.

use crate::entity_registry::EntityProperties;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_map_animation::SpriteData;
use bevy_map_core::Value;

/// Helper to create a TilemapTexture from an image handle
pub fn tilemap_texture_from_image(image: Handle<Image>) -> TilemapTexture {
    TilemapTexture::Single(image)
}

/// Calculate the world position of a tile
pub fn tile_to_world_pos(
    tile_pos: TilePos,
    map_size: TilemapSize,
    tile_size: TilemapTileSize,
    grid_size: TilemapGridSize,
    map_type: &TilemapType,
    anchor: &TilemapAnchor,
    map_transform: &Transform,
) -> Vec2 {
    let local = tile_pos.center_in_world(&map_size, &grid_size, &tile_size, map_type, anchor);
    let world = map_transform.transform_point(Vec3::new(local.x, local.y, 0.0));
    Vec2::new(world.x, world.y)
}

/// Calculate the tile position from a world position
pub fn world_to_tile_pos(
    world_pos: Vec2,
    map_size: TilemapSize,
    tile_size: TilemapTileSize,
    grid_size: TilemapGridSize,
    map_type: &TilemapType,
    anchor: &TilemapAnchor,
    map_transform: &Transform,
) -> Option<TilePos> {
    // Transform world to local map space
    let inverse = map_transform.compute_affine().inverse();
    let local = inverse.transform_point3(Vec3::new(world_pos.x, world_pos.y, 0.0));

    // Convert local position to tile coordinates
    TilePos::from_world_pos(
        &Vec2::new(local.x, local.y),
        &map_size,
        &grid_size,
        &tile_size,
        map_type,
        anchor,
    )
}

// ============================================================================
// Sprite spawning systems for automatic sprite component creation
// ============================================================================

/// Marker component for sprite slots on entities
#[derive(Component, Debug, Clone)]
pub struct SpriteSlot {
    /// Property name containing the SpriteData
    pub property_name: String,
    /// The sprite data configuration
    pub sprite_data: SpriteData,
}

/// System that detects entities with sprite properties and spawns child sprite components
pub fn spawn_sprite_components(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &EntityProperties), Without<SpriteSlot>>,
) {
    for (entity, props) in query.iter() {
        // Collect all sprite properties
        let sprite_properties: Vec<(String, SpriteData)> = props
            .properties
            .iter()
            .filter_map(|(name, value)| {
                match value {
                    Value::Object(obj) => {
                        // Try to deserialize as SpriteData
                        serde_json::to_value(obj)
                            .ok()
                            .and_then(|json| serde_json::from_value::<SpriteData>(json).ok())
                            .map(|sprite_data| (name.clone(), sprite_data))
                    }
                    _ => None,
                }
            })
            .collect();

        if sprite_properties.is_empty() {
            continue;
        }

        // Spawn child entities for each sprite property
        for (prop_name, sprite_data) in sprite_properties {
            let texture_path = sprite_data.sheet_path.clone();
            let texture_handle = if !texture_path.is_empty() {
                asset_server.load::<Image>(&texture_path)
            } else {
                Handle::default()
            };

            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    SpriteSlot {
                        property_name: prop_name,
                        sprite_data,
                    },
                    Sprite {
                        image: texture_handle,
                        ..default()
                    },
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::default(),
                ));
            });
        }
    }
}

/// System that completes sprite setup once assets are loaded
pub fn complete_sprite_loads(mut query: Query<(&SpriteSlot, &mut Sprite)>) {
    for (slot, mut sprite) in query.iter_mut() {
        let sprite_data = &slot.sprite_data;

        // Calculate sprite rect from frame index and grid
        if let Some(anim) = sprite_data.animations.values().next() {
            if let Some(&first_frame) = anim.frames.first() {
                let columns = sprite_data.columns;
                if columns > 0 {
                    let row = first_frame as u32 / columns;
                    let col = first_frame as u32 % columns;

                    sprite.rect = Some(Rect {
                        min: Vec2::new(
                            col as f32 * sprite_data.frame_width as f32,
                            row as f32 * sprite_data.frame_height as f32,
                        ),
                        max: Vec2::new(
                            (col + 1) as f32 * sprite_data.frame_width as f32,
                            (row + 1) as f32 * sprite_data.frame_height as f32,
                        ),
                    });
                }
            }
        }
    }
}
