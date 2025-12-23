//! Rendering utilities for runtime tilemaps
//!
//! This module provides helper functions for working with bevy_ecs_tilemap rendering.

use crate::entity_registry::EntityProperties;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_map_animation::{AnimatedSprite, SpriteData};
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

/// System that detects entities with sprite properties and adds Sprite + AnimatedSprite components
///
/// This follows the Bevy pattern of adding sprite components directly to the entity
/// rather than spawning child entities. This makes animation control much simpler
/// as games can query `AnimatedSprite` directly on their entity types.
pub fn spawn_sprite_components(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sprite_data_assets: ResMut<Assets<SpriteData>>,
    query: Query<(Entity, &EntityProperties), Without<SpriteSlot>>,
) {
    for (entity, props) in query.iter() {
        // Find first sprite property (we only support one sprite per entity)
        let sprite_property = props.properties.iter().find_map(|(name, value)| {
            match value {
                Value::Object(_) => {
                    // Try to deserialize as SpriteData
                    // Convert our Value to serde_json::Value first, then deserialize
                    let json = value.to_json();
                    serde_json::from_value::<SpriteData>(json)
                        .ok()
                        .map(|sprite_data| (name.clone(), sprite_data))
                }
                _ => None,
            }
        });

        let Some((prop_name, sprite_data)) = sprite_property else {
            continue;
        };

        // Load texture
        let texture_path = sprite_data.sheet_path.clone();
        let texture_handle = if !texture_path.is_empty() {
            asset_server.load::<Image>(&texture_path)
        } else {
            Handle::default()
        };

        // Calculate initial sprite rect from first frame
        let initial_rect = if let Some(anim) = sprite_data.animations.values().next() {
            if let Some(&first_frame) = anim.frames.first() {
                let columns = sprite_data.columns;
                if columns > 0 {
                    let row = first_frame as u32 / columns;
                    let col = first_frame as u32 % columns;
                    Some(bevy::math::Rect::new(
                        col as f32 * sprite_data.frame_width as f32,
                        row as f32 * sprite_data.frame_height as f32,
                        (col + 1) as f32 * sprite_data.frame_width as f32,
                        (row + 1) as f32 * sprite_data.frame_height as f32,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Add Sprite and SpriteSlot directly to the entity
        commands.entity(entity).insert((
            SpriteSlot {
                property_name: prop_name,
                sprite_data: sprite_data.clone(),
            },
            Sprite {
                image: texture_handle,
                rect: initial_rect,
                ..default()
            },
        ));

        // If sprite has animations, add AnimatedSprite component
        if !sprite_data.animations.is_empty() {
            let sprite_data_handle = sprite_data_assets.add(sprite_data.clone());
            let mut animated = AnimatedSprite::new(sprite_data_handle);

            // Auto-play "idle" animation if available, otherwise first animation
            let initial_anim = sprite_data
                .animations
                .get("idle")
                .map(|_| "idle".to_string())
                .or_else(|| sprite_data.animations.keys().next().cloned());

            if let Some(ref anim_name) = initial_anim {
                animated.play(anim_name);
            }

            commands.entity(entity).insert(animated);
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
