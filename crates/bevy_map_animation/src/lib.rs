//! bevy_map_animation - Animation and sprite types for bevy_map_editor
//!
//! This crate provides types for defining sprite sheets with multiple animations,
//! designed for use with the bevy_map_editor ecosystem.
//!
//! # Usage
//!
//! ```rust,ignore
//! use bevy_map_animation::{SpriteData, AnimationDef, LoopMode};
//!
//! let mut sprite = SpriteData::new("sprites/character.png", 32, 32);
//! sprite.add_animation("idle", AnimationDef {
//!     frames: vec![0, 1, 2, 3],
//!     frame_duration_ms: 200,
//!     loop_mode: LoopMode::Loop,
//! });
//! sprite.add_animation("attack", AnimationDef {
//!     frames: vec![4, 5, 6, 7, 8],
//!     frame_duration_ms: 80,
//!     loop_mode: LoopMode::Once,
//! });
//! ```

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Animation loop mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Reflect)]
#[serde(rename_all = "lowercase")]
pub enum LoopMode {
    /// Loop the animation continuously
    #[default]
    Loop,
    /// Play the animation once and stop on the last frame
    Once,
    /// Play forward then backward continuously
    PingPong,
}

impl LoopMode {
    /// Get the display name for this loop mode
    pub fn display_name(&self) -> &'static str {
        match self {
            LoopMode::Loop => "Loop",
            LoopMode::Once => "Once",
            LoopMode::PingPong => "Ping-Pong",
        }
    }

    /// Get all available loop modes
    pub fn all() -> &'static [LoopMode] {
        &[LoopMode::Loop, LoopMode::Once, LoopMode::PingPong]
    }
}

/// A single animation definition
#[derive(Debug, Clone, Serialize, Deserialize, Default, Reflect)]
pub struct AnimationDef {
    /// Frame indices into the spritesheet grid (left-to-right, top-to-bottom)
    pub frames: Vec<usize>,
    /// Duration of each frame in milliseconds
    #[serde(default = "default_frame_duration")]
    pub frame_duration_ms: u32,
    /// How the animation loops
    #[serde(default)]
    pub loop_mode: LoopMode,
}

fn default_frame_duration() -> u32 {
    100
}

impl AnimationDef {
    /// Create a new animation definition
    pub fn new(frames: Vec<usize>, frame_duration_ms: u32, loop_mode: LoopMode) -> Self {
        Self {
            frames,
            frame_duration_ms,
            loop_mode,
        }
    }

    /// Get the total duration of one loop of the animation in milliseconds
    pub fn total_duration_ms(&self) -> u32 {
        self.frames.len() as u32 * self.frame_duration_ms
    }

    /// Get the frame index for a given time in milliseconds
    pub fn frame_at_time(&self, time_ms: u32) -> Option<usize> {
        if self.frames.is_empty() {
            return None;
        }

        let total_duration = self.total_duration_ms();
        if total_duration == 0 {
            return self.frames.first().copied();
        }

        let loop_time = match self.loop_mode {
            LoopMode::Once => time_ms.min(total_duration.saturating_sub(1)),
            LoopMode::Loop => time_ms % total_duration,
            LoopMode::PingPong => {
                let double_duration = total_duration * 2;
                let t = time_ms % double_duration;
                if t < total_duration {
                    t
                } else {
                    double_duration - t
                }
            }
        };

        let frame_index = (loop_time / self.frame_duration_ms) as usize;
        self.frames.get(frame_index.min(self.frames.len() - 1)).copied()
    }
}

fn default_pivot() -> f32 {
    0.5
}

/// Sprite data with spritesheet reference and animations
///
/// This represents a complete sprite sheet definition including frame dimensions,
/// grid layout, pivot point, and named animations.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Asset, Reflect)]
pub struct SpriteData {
    /// Unique identifier for this sprite data
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    /// Display name for this animation asset
    #[serde(default)]
    pub name: String,
    /// Path to the spritesheet image (relative to assets)
    pub sheet_path: String,
    /// Width of each frame in pixels
    pub frame_width: u32,
    /// Height of each frame in pixels
    pub frame_height: u32,
    /// Number of columns in the spritesheet (calculated from image width / frame_width)
    #[serde(default)]
    pub columns: u32,
    /// Number of rows in the spritesheet (calculated from image height / frame_height)
    #[serde(default)]
    pub rows: u32,
    /// Pivot point X (0.0-1.0, where 0.5 is center)
    #[serde(default = "default_pivot")]
    pub pivot_x: f32,
    /// Pivot point Y (0.0-1.0, where 0.5 is center)
    #[serde(default = "default_pivot")]
    pub pivot_y: f32,
    /// Named animations (user-defined names like "idle", "attack", "death", etc.)
    #[serde(default)]
    #[reflect(ignore)]
    pub animations: HashMap<String, AnimationDef>,
}

impl SpriteData {
    /// Create a new sprite data with the given sheet path and frame dimensions
    pub fn new(sheet_path: impl Into<String>, frame_width: u32, frame_height: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            sheet_path: sheet_path.into(),
            frame_width,
            frame_height,
            columns: 0,
            rows: 0,
            pivot_x: 0.5,
            pivot_y: 0.5,
            animations: HashMap::new(),
        }
    }

    /// Create a new named sprite data with the given sheet path and frame dimensions
    pub fn new_named(name: impl Into<String>, sheet_path: impl Into<String>, frame_width: u32, frame_height: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            sheet_path: sheet_path.into(),
            frame_width,
            frame_height,
            columns: 0,
            rows: 0,
            pivot_x: 0.5,
            pivot_y: 0.5,
            animations: HashMap::new(),
        }
    }

    /// Get total frame count based on grid
    pub fn total_frames(&self) -> usize {
        (self.columns * self.rows) as usize
    }

    /// Convert frame index to grid position (col, row)
    pub fn frame_to_grid(&self, frame: usize) -> (u32, u32) {
        if self.columns == 0 {
            return (0, 0);
        }
        let col = (frame as u32) % self.columns;
        let row = (frame as u32) / self.columns;
        (col, row)
    }

    /// Convert grid position to frame index
    pub fn grid_to_frame(&self, col: u32, row: u32) -> usize {
        (row * self.columns + col) as usize
    }

    /// Get the UV rectangle for a specific frame
    pub fn frame_uv(&self, frame: usize) -> (f32, f32, f32, f32) {
        let (col, row) = self.frame_to_grid(frame);
        let u = col as f32 / self.columns.max(1) as f32;
        let v = row as f32 / self.rows.max(1) as f32;
        let w = 1.0 / self.columns.max(1) as f32;
        let h = 1.0 / self.rows.max(1) as f32;
        (u, v, w, h)
    }

    /// Add an animation definition
    pub fn add_animation(&mut self, name: impl Into<String>, animation: AnimationDef) {
        self.animations.insert(name.into(), animation);
    }

    /// Get an animation by name
    pub fn get_animation(&self, name: &str) -> Option<&AnimationDef> {
        self.animations.get(name)
    }

    /// Get all animation names
    pub fn animation_names(&self) -> impl Iterator<Item = &str> {
        self.animations.keys().map(|s| s.as_str())
    }

    /// Update grid dimensions from image size
    pub fn update_from_image_size(&mut self, image_width: u32, image_height: u32) {
        if self.frame_width > 0 {
            self.columns = image_width / self.frame_width;
        }
        if self.frame_height > 0 {
            self.rows = image_height / self.frame_height;
        }
    }
}

/// Component for playing sprite animations
#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct AnimatedSprite {
    /// Handle to the sprite data asset
    #[reflect(ignore)]
    pub sprite_data: Handle<SpriteData>,
    /// Current animation name
    pub current_animation: Option<String>,
    /// Elapsed time in the current animation (milliseconds)
    pub elapsed_ms: u32,
    /// Whether the animation is playing
    pub playing: bool,
}

impl AnimatedSprite {
    /// Create a new animated sprite component
    pub fn new(sprite_data: Handle<SpriteData>) -> Self {
        Self {
            sprite_data,
            current_animation: None,
            elapsed_ms: 0,
            playing: false,
        }
    }

    /// Play an animation by name
    pub fn play(&mut self, animation_name: impl Into<String>) {
        let name = animation_name.into();
        if self.current_animation.as_ref() != Some(&name) {
            self.current_animation = Some(name);
            self.elapsed_ms = 0;
        }
        self.playing = true;
    }

    /// Stop the current animation
    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// Reset the animation to the beginning
    pub fn reset(&mut self) {
        self.elapsed_ms = 0;
    }
}

/// Plugin for sprite animation support
pub struct SpriteAnimationPlugin;

impl Plugin for SpriteAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SpriteData>()
            .register_type::<LoopMode>()
            .register_type::<AnimationDef>()
            .register_type::<SpriteData>()
            .register_type::<AnimatedSprite>()
            .add_systems(Update, update_animated_sprites);
    }
}

/// System to update animated sprites
fn update_animated_sprites(
    time: Res<Time>,
    sprite_assets: Res<Assets<SpriteData>>,
    mut query: Query<(&mut AnimatedSprite, &mut Sprite)>,
) {
    for (mut animated, mut sprite) in query.iter_mut() {
        if !animated.playing {
            continue;
        }

        // Get the sprite data
        let Some(sprite_data) = sprite_assets.get(&animated.sprite_data) else {
            continue;
        };

        // Get the current animation
        let Some(animation_name) = &animated.current_animation else {
            continue;
        };
        let Some(animation) = sprite_data.get_animation(animation_name) else {
            continue;
        };

        // Update elapsed time
        animated.elapsed_ms += (time.delta_secs() * 1000.0) as u32;

        // Get the current frame
        let Some(frame_index) = animation.frame_at_time(animated.elapsed_ms) else {
            continue;
        };

        // Update the sprite rect
        let (u, v, w, h) = sprite_data.frame_uv(frame_index);
        if let Some(ref mut rect) = sprite.rect {
            // Convert UV to pixels
            let pixel_x = (u * sprite_data.columns as f32 * sprite_data.frame_width as f32) as u32;
            let pixel_y = (v * sprite_data.rows as f32 * sprite_data.frame_height as f32) as u32;
            let pixel_w = (w * sprite_data.columns as f32 * sprite_data.frame_width as f32) as u32;
            let pixel_h = (h * sprite_data.rows as f32 * sprite_data.frame_height as f32) as u32;
            *rect = bevy::math::Rect::new(
                pixel_x as f32,
                pixel_y as f32,
                (pixel_x + pixel_w) as f32,
                (pixel_y + pixel_h) as f32,
            );
        }

        // Check if animation ended (for Once mode)
        if animation.loop_mode == LoopMode::Once {
            let total = animation.total_duration_ms();
            if animated.elapsed_ms >= total {
                animated.playing = false;
            }
        }
    }
}
