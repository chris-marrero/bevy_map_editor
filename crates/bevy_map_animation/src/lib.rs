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
use serde_json::Value;
use std::collections::{HashMap, HashSet};
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

// ============================================================================
// Animation Triggers & Windows
// ============================================================================

fn default_volume() -> f32 {
    1.0
}

/// Payload data for animation triggers and windows
#[derive(Debug, Clone, Serialize, Deserialize, Default, Reflect, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum TriggerPayload {
    /// No payload - just a trigger event
    #[default]
    None,
    /// Play a sound effect
    Sound {
        /// Path to the sound asset
        path: String,
        /// Volume (0.0 - 1.0)
        #[serde(default = "default_volume")]
        volume: f32,
    },
    /// Spawn a particle/VFX effect
    Particle {
        /// Name or path of the particle effect
        effect: String,
        /// Offset from entity position (x, y)
        #[serde(default)]
        offset: (f32, f32),
    },
    /// Custom game event with string identifier
    Custom {
        /// Event name (e.g., "footstep", "attack_hitbox", "combo_window")
        event_name: String,
        /// Optional key-value parameters
        #[serde(default)]
        #[reflect(ignore)]
        params: HashMap<String, Value>,
    },
}

impl TriggerPayload {
    /// Get the display name for this payload type
    pub fn display_name(&self) -> &'static str {
        match self {
            TriggerPayload::None => "None",
            TriggerPayload::Sound { .. } => "Sound",
            TriggerPayload::Particle { .. } => "Particle",
            TriggerPayload::Custom { .. } => "Custom",
        }
    }

    /// Get all available payload types for UI
    pub fn all_types() -> &'static [&'static str] {
        &["None", "Sound", "Particle", "Custom"]
    }
}

/// A one-shot animation trigger - fires once at a specific time
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct AnimationTrigger {
    /// Unique identifier for this trigger
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    /// Display name (for editor UI)
    #[serde(default)]
    pub name: String,
    /// Time in milliseconds when this trigger fires
    pub time_ms: u32,
    /// The payload/action for this trigger
    #[serde(default)]
    pub payload: TriggerPayload,
    /// Editor display color (RGB), None = use default orange
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<[u8; 3]>,
}

impl AnimationTrigger {
    /// Create a new trigger at the specified time
    pub fn new(name: impl Into<String>, time_ms: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            time_ms,
            payload: TriggerPayload::None,
            color: None,
        }
    }

    /// Create a new trigger with a payload
    pub fn with_payload(name: impl Into<String>, time_ms: u32, payload: TriggerPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            time_ms,
            payload,
            color: None,
        }
    }
}

/// Phase of an animation window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect, Default)]
#[serde(rename_all = "lowercase")]
pub enum WindowPhase {
    /// Window just started
    #[default]
    Begin,
    /// Window is active (fires every frame while active)
    Tick,
    /// Window just ended
    End,
}

impl WindowPhase {
    /// Get the display name for this phase
    pub fn display_name(&self) -> &'static str {
        match self {
            WindowPhase::Begin => "Begin",
            WindowPhase::Tick => "Tick",
            WindowPhase::End => "End",
        }
    }
}

/// A duration-based animation window - has begin, tick, and end phases
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct AnimationWindow {
    /// Unique identifier for this window
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    /// Display name (for editor UI)
    #[serde(default)]
    pub name: String,
    /// Start time in milliseconds
    pub start_ms: u32,
    /// End time in milliseconds
    pub end_ms: u32,
    /// The payload/action for this window
    #[serde(default)]
    pub payload: TriggerPayload,
    /// Editor display color (RGB), None = use default green
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<[u8; 3]>,
}

impl AnimationWindow {
    /// Create a new window with the specified time range
    pub fn new(name: impl Into<String>, start_ms: u32, end_ms: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            start_ms,
            end_ms,
            payload: TriggerPayload::None,
            color: None,
        }
    }

    /// Create a new window with a payload
    pub fn with_payload(
        name: impl Into<String>,
        start_ms: u32,
        end_ms: u32,
        payload: TriggerPayload,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            start_ms,
            end_ms,
            payload,
            color: None,
        }
    }

    /// Check if a time is within this window's active range
    pub fn is_active_at(&self, time_ms: u32) -> bool {
        time_ms >= self.start_ms && time_ms < self.end_ms
    }

    /// Get the duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        self.end_ms.saturating_sub(self.start_ms)
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
    /// One-shot triggers (fire at specific times)
    #[serde(default)]
    #[reflect(ignore)]
    pub triggers: Vec<AnimationTrigger>,
    /// Duration-based windows (begin/tick/end phases)
    #[serde(default)]
    #[reflect(ignore)]
    pub windows: Vec<AnimationWindow>,
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
            triggers: Vec::new(),
            windows: Vec::new(),
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
        self.frames
            .get(frame_index.min(self.frames.len() - 1))
            .copied()
    }

    /// Get all triggers that should fire between prev_ms (exclusive) and current_ms (inclusive)
    pub fn triggers_in_range(&self, prev_ms: u32, current_ms: u32) -> Vec<&AnimationTrigger> {
        self.triggers
            .iter()
            .filter(|t| t.time_ms > prev_ms && t.time_ms <= current_ms)
            .collect()
    }

    /// Get all windows active at a given time
    pub fn active_windows_at(&self, time_ms: u32) -> Vec<&AnimationWindow> {
        self.windows
            .iter()
            .filter(|w| w.is_active_at(time_ms))
            .collect()
    }

    /// Convert frame index to time in milliseconds
    pub fn frame_to_time_ms(&self, frame_index: usize) -> u32 {
        frame_index as u32 * self.frame_duration_ms
    }

    /// Convert time to frame index
    pub fn time_to_frame(&self, time_ms: u32) -> usize {
        if self.frame_duration_ms == 0 {
            return 0;
        }
        (time_ms / self.frame_duration_ms) as usize
    }

    /// Add a trigger to this animation
    pub fn add_trigger(&mut self, trigger: AnimationTrigger) {
        self.triggers.push(trigger);
    }

    /// Add a window to this animation
    pub fn add_window(&mut self, window: AnimationWindow) {
        self.windows.push(window);
    }

    /// Remove a trigger by ID
    pub fn remove_trigger(&mut self, id: Uuid) -> bool {
        let len = self.triggers.len();
        self.triggers.retain(|t| t.id != id);
        self.triggers.len() != len
    }

    /// Remove a window by ID
    pub fn remove_window(&mut self, id: Uuid) -> bool {
        let len = self.windows.len();
        self.windows.retain(|w| w.id != id);
        self.windows.len() != len
    }

    /// Get a trigger by ID
    pub fn get_trigger(&self, id: Uuid) -> Option<&AnimationTrigger> {
        self.triggers.iter().find(|t| t.id == id)
    }

    /// Get a mutable trigger by ID
    pub fn get_trigger_mut(&mut self, id: Uuid) -> Option<&mut AnimationTrigger> {
        self.triggers.iter_mut().find(|t| t.id == id)
    }

    /// Get a window by ID
    pub fn get_window(&self, id: Uuid) -> Option<&AnimationWindow> {
        self.windows.iter().find(|w| w.id == id)
    }

    /// Get a mutable window by ID
    pub fn get_window_mut(&mut self, id: Uuid) -> Option<&mut AnimationWindow> {
        self.windows.iter_mut().find(|w| w.id == id)
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
    pub fn new_named(
        name: impl Into<String>,
        sheet_path: impl Into<String>,
        frame_width: u32,
        frame_height: u32,
    ) -> Self {
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
///
/// Automatically requires [`WindowTracker`] component for window event tracking.
#[derive(Component, Debug, Clone, Default, Reflect)]
#[require(WindowTracker)]
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

/// Tracks active animation windows for an entity
///
/// This component is automatically added when [`AnimatedSprite`] is inserted (via `#[require]`).
/// It tracks which windows are currently active to properly fire Begin/Tick/End events.
#[derive(Component, Debug, Clone, Default)]
pub struct WindowTracker {
    /// IDs of currently active windows
    pub active_windows: HashSet<Uuid>,
    /// Previous elapsed time (for detecting trigger crossings)
    pub prev_elapsed_ms: u32,
}

// ============================================================================
// Animation Events
// ============================================================================

/// Event fired when an AnimationTrigger fires (one-shot)
#[derive(Message, Debug, Clone)]
pub struct AnimationTriggerEvent {
    /// The entity that triggered this event
    pub entity: Entity,
    /// Animation name
    pub animation: String,
    /// The trigger ID
    pub trigger_id: Uuid,
    /// Name of the trigger
    pub trigger_name: String,
    /// The payload data
    pub payload: TriggerPayload,
}

/// Event fired for AnimationWindow phase changes (duration-based)
#[derive(Message, Debug, Clone)]
pub struct AnimationWindowEvent {
    /// The entity that triggered this event
    pub entity: Entity,
    /// Animation name
    pub animation: String,
    /// The window ID
    pub window_id: Uuid,
    /// Name of the window
    pub window_name: String,
    /// Current phase (Begin, Tick, or End)
    pub phase: WindowPhase,
    /// The payload data
    pub payload: TriggerPayload,
    /// Progress through the window (0.0 - 1.0), only meaningful for Tick
    pub progress: f32,
}

/// Convenience event for sound payloads
#[derive(Message, Debug, Clone)]
pub struct AnimationSoundEvent {
    /// The entity that triggered this event
    pub entity: Entity,
    /// Path to the sound asset
    pub path: String,
    /// Volume (0.0 - 1.0)
    pub volume: f32,
}

/// Convenience event for particle/VFX payloads
#[derive(Message, Debug, Clone)]
pub struct AnimationParticleEvent {
    /// The entity that triggered this event
    pub entity: Entity,
    /// Name or path of the particle effect
    pub effect: String,
    /// Offset from entity position (x, y)
    pub offset: (f32, f32),
}

/// Convenience event for custom payloads
#[derive(Message, Debug, Clone)]
pub struct AnimationCustomEvent {
    /// The entity that triggered this event
    pub entity: Entity,
    /// Event name
    pub event_name: String,
    /// Optional key-value parameters
    pub params: HashMap<String, Value>,
}

// ============================================================================
// Entity-Scoped Observer Events (for use with .observe())
// ============================================================================

/// Entity-scoped event fired when an animation trigger fires.
///
/// Use this with Bevy's Observer pattern for entity-specific handling:
///
/// ```rust,ignore
/// commands.spawn((AnimatedSpriteHandle::new(...), Transform::default()))
///     .observe(|trigger: Trigger<AnimationTriggered>| {
///         info!("Trigger '{}' fired!", trigger.event().name);
///     });
/// ```
#[derive(EntityEvent, Debug, Clone)]
pub struct AnimationTriggered {
    /// The entity this event targets
    pub entity: Entity,
    /// Name of the trigger
    pub name: String,
    /// The trigger ID
    pub trigger_id: Uuid,
    /// Animation name
    pub animation: String,
    /// Time in animation when trigger fired (ms)
    pub time_ms: u32,
    /// The payload data
    pub payload: TriggerPayload,
}

/// Entity-scoped event fired when an animation window changes phase.
///
/// Use this with Bevy's Observer pattern for entity-specific handling:
///
/// ```rust,ignore
/// commands.spawn((AnimatedSpriteHandle::new(...), Transform::default()))
///     .observe(|trigger: Trigger<AnimationWindowChanged>| {
///         match trigger.event().phase {
///             WindowPhase::Begin => enable_hitbox(),
///             WindowPhase::End => disable_hitbox(),
///             _ => {}
///         }
///     });
/// ```
#[derive(EntityEvent, Debug, Clone)]
pub struct AnimationWindowChanged {
    /// The entity this event targets
    pub entity: Entity,
    /// Name of the window
    pub name: String,
    /// The window ID
    pub window_id: Uuid,
    /// Animation name
    pub animation: String,
    /// Current phase (Begin, Tick, or End)
    pub phase: WindowPhase,
    /// Progress through the window (0.0 - 1.0), only meaningful for Tick
    pub progress: f32,
    /// The payload data
    pub payload: TriggerPayload,
}

// ============================================================================
// Custom Trigger/Window Type System
// ============================================================================

use std::marker::PhantomData;

/// Trait for types that can be animation triggers.
///
/// Implement this trait to define custom type-safe trigger events that are
/// automatically dispatched when a `TriggerPayload::Custom` with a matching
/// `event_name` is encountered.
///
/// # Example
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_map_animation::AnimationTriggerType;
///
/// #[derive(Event, Clone)]
/// pub struct AttackHitbox {
///     pub damage: i32,
///     pub knockback: f32,
/// }
///
/// impl AnimationTriggerType for AttackHitbox {
///     fn trigger_name() -> &'static str { "attack_hitbox" }
///
///     fn from_params(params: &std::collections::HashMap<String, serde_json::Value>) -> Option<Self> {
///         Some(Self {
///             damage: params.get("damage")?.as_i64()? as i32,
///             knockback: params.get("knockback").and_then(|v| v.as_f64()).unwrap_or(5.0) as f32,
///         })
///     }
/// }
/// ```
pub trait AnimationTriggerType: EntityEvent + Clone + Send + Sync + 'static
where
    for<'a> <Self as Event>::Trigger<'a>: Default,
{
    /// The trigger name as used in the editor (matches `TriggerPayload::Custom` event_name)
    fn trigger_name() -> &'static str;

    /// Create an instance from the params HashMap.
    /// Return None if required params are missing or invalid.
    fn from_params(params: &HashMap<String, Value>) -> Option<Self>;
}

/// Trait for types that can be animation windows.
///
/// Similar to `AnimationTriggerType`, but for duration-based events with
/// Begin/Tick/End phases.
pub trait AnimationWindowType: EntityEvent + Clone + Send + Sync + 'static
where
    for<'a> <Self as Event>::Trigger<'a>: Default,
{
    /// The window name as used in the editor
    fn window_name() -> &'static str;

    /// Create an instance from the params HashMap.
    fn from_params(params: &HashMap<String, Value>) -> Option<Self>;
}

// Internal trait for type-erased trigger dispatch
trait TriggerDispatcher: Send + Sync {
    fn dispatch(
        &self,
        commands: &mut Commands,
        entity: Entity,
        animation: &str,
        params: &HashMap<String, Value>,
    );
}

struct TypedTriggerDispatcher<T: AnimationTriggerType>
where
    for<'a> <T as Event>::Trigger<'a>: Default,
{
    _marker: PhantomData<T>,
}

impl<T: AnimationTriggerType> TriggerDispatcher for TypedTriggerDispatcher<T>
where
    for<'a> <T as Event>::Trigger<'a>: Default,
{
    fn dispatch(
        &self,
        commands: &mut Commands,
        entity: Entity,
        _animation: &str,
        params: &HashMap<String, Value>,
    ) {
        if let Some(payload) = T::from_params(params) {
            // Fire Bevy Observer trigger on the entity
            commands.entity(entity).trigger(move |_| payload);
        }
    }
}

/// Registry for custom animation trigger types.
///
/// This resource stores registered trigger types and dispatches typed events
/// when custom triggers fire during animation playback.
#[derive(Resource, Default)]
pub struct AnimationTriggerRegistry {
    dispatchers: HashMap<String, Box<dyn TriggerDispatcher>>,
}

impl AnimationTriggerRegistry {
    /// Register a custom trigger type
    pub fn register<T: AnimationTriggerType>(&mut self)
    where
        for<'a> <T as Event>::Trigger<'a>: Default,
    {
        self.dispatchers.insert(
            T::trigger_name().to_string(),
            Box::new(TypedTriggerDispatcher::<T> {
                _marker: PhantomData,
            }),
        );
    }

    /// Check if a trigger name is registered
    pub fn is_registered(&self, name: &str) -> bool {
        self.dispatchers.contains_key(name)
    }

    /// Dispatch a custom trigger to registered handlers
    pub fn dispatch(
        &self,
        commands: &mut Commands,
        entity: Entity,
        animation: &str,
        event_name: &str,
        params: &HashMap<String, Value>,
    ) {
        if let Some(dispatcher) = self.dispatchers.get(event_name) {
            dispatcher.dispatch(commands, entity, animation, params);
        }
    }
}

// Internal trait for type-erased window dispatch
trait WindowDispatcher: Send + Sync {
    fn dispatch(
        &self,
        commands: &mut Commands,
        entity: Entity,
        animation: &str,
        phase: WindowPhase,
        progress: f32,
        params: &HashMap<String, Value>,
    );
}

struct TypedWindowDispatcher<T: AnimationWindowType>
where
    for<'a> <T as Event>::Trigger<'a>: Default,
{
    _marker: PhantomData<T>,
}

impl<T: AnimationWindowType> WindowDispatcher for TypedWindowDispatcher<T>
where
    for<'a> <T as Event>::Trigger<'a>: Default,
{
    fn dispatch(
        &self,
        commands: &mut Commands,
        entity: Entity,
        _animation: &str,
        _phase: WindowPhase,
        _progress: f32,
        params: &HashMap<String, Value>,
    ) {
        if let Some(payload) = T::from_params(params) {
            // Fire Bevy Observer trigger on the entity
            commands.entity(entity).trigger(move |_| payload);
        }
    }
}

/// Registry for custom animation window types.
#[derive(Resource, Default)]
pub struct AnimationWindowRegistry {
    dispatchers: HashMap<String, Box<dyn WindowDispatcher>>,
}

impl AnimationWindowRegistry {
    /// Register a custom window type
    pub fn register<T: AnimationWindowType>(&mut self)
    where
        for<'a> <T as Event>::Trigger<'a>: Default,
    {
        self.dispatchers.insert(
            T::window_name().to_string(),
            Box::new(TypedWindowDispatcher::<T> {
                _marker: PhantomData,
            }),
        );
    }

    /// Check if a window name is registered
    pub fn is_registered(&self, name: &str) -> bool {
        self.dispatchers.contains_key(name)
    }

    /// Dispatch a custom window event to registered handlers
    pub fn dispatch(
        &self,
        commands: &mut Commands,
        entity: Entity,
        animation: &str,
        phase: WindowPhase,
        progress: f32,
        event_name: &str,
        params: &HashMap<String, Value>,
    ) {
        if let Some(dispatcher) = self.dispatchers.get(event_name) {
            dispatcher.dispatch(commands, entity, animation, phase, progress, params);
        }
    }
}

/// Extension trait for registering animation trigger and window types.
///
/// # Example
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_map_animation::{SpriteAnimationPlugin, AnimationEventExt};
///
/// App::new()
///     .add_plugins(SpriteAnimationPlugin)
///     .register_animation_trigger::<AttackHitbox>()
///     .register_animation_window::<ComboWindow>()
///     .run();
/// ```
pub trait AnimationEventExt {
    /// Register a custom trigger type for type-safe event handling
    fn register_animation_trigger<T: AnimationTriggerType>(&mut self) -> &mut Self
    where
        for<'a> <T as Event>::Trigger<'a>: Default;

    /// Register a custom window type for type-safe event handling
    fn register_animation_window<T: AnimationWindowType>(&mut self) -> &mut Self
    where
        for<'a> <T as Event>::Trigger<'a>: Default;
}

impl AnimationEventExt for App {
    fn register_animation_trigger<T: AnimationTriggerType>(&mut self) -> &mut Self
    where
        for<'a> <T as Event>::Trigger<'a>: Default,
    {
        // Ensure registry exists
        if !self.world().contains_resource::<AnimationTriggerRegistry>() {
            self.insert_resource(AnimationTriggerRegistry::default());
        }

        // Register the dispatcher
        self.world_mut()
            .resource_mut::<AnimationTriggerRegistry>()
            .register::<T>();

        self
    }

    fn register_animation_window<T: AnimationWindowType>(&mut self) -> &mut Self
    where
        for<'a> <T as Event>::Trigger<'a>: Default,
    {
        // Ensure registry exists
        if !self.world().contains_resource::<AnimationWindowRegistry>() {
            self.insert_resource(AnimationWindowRegistry::default());
        }

        // Register the dispatcher
        self.world_mut()
            .resource_mut::<AnimationWindowRegistry>()
            .register::<T>();

        self
    }
}

/// Plugin for sprite animation support
pub struct SpriteAnimationPlugin;

impl Plugin for SpriteAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SpriteData>()
            // Type registration
            .register_type::<LoopMode>()
            .register_type::<AnimationDef>()
            .register_type::<SpriteData>()
            .register_type::<AnimatedSprite>()
            .register_type::<TriggerPayload>()
            .register_type::<AnimationTrigger>()
            .register_type::<AnimationWindow>()
            .register_type::<WindowPhase>()
            // Message registration (for basic animation events)
            .add_message::<AnimationTriggerEvent>()
            .add_message::<AnimationWindowEvent>()
            .add_message::<AnimationSoundEvent>()
            .add_message::<AnimationParticleEvent>()
            .add_message::<AnimationCustomEvent>()
            // Initialize registries for custom trigger/window types
            .init_resource::<AnimationTriggerRegistry>()
            .init_resource::<AnimationWindowRegistry>()
            // Systems
            .add_systems(Update, update_animated_sprites);
    }
}

/// System to update animated sprites and fire animation events
fn update_animated_sprites(
    mut commands: Commands,
    time: Res<Time>,
    sprite_assets: Res<Assets<SpriteData>>,
    trigger_registry: Res<AnimationTriggerRegistry>,
    window_registry: Res<AnimationWindowRegistry>,
    mut query: Query<(
        Entity,
        &mut AnimatedSprite,
        &mut Sprite,
        Option<&mut WindowTracker>,
    )>,
    mut trigger_events: MessageWriter<AnimationTriggerEvent>,
    mut window_events: MessageWriter<AnimationWindowEvent>,
    mut sound_events: MessageWriter<AnimationSoundEvent>,
    mut particle_events: MessageWriter<AnimationParticleEvent>,
    mut custom_events: MessageWriter<AnimationCustomEvent>,
) {
    for (entity, mut animated, mut sprite, tracker_opt) in query.iter_mut() {
        if !animated.playing {
            continue;
        }

        // Get the sprite data
        let Some(sprite_data) = sprite_assets.get(&animated.sprite_data) else {
            continue;
        };

        // Get the current animation
        let Some(animation_name) = animated.current_animation.clone() else {
            continue;
        };
        let Some(animation) = sprite_data.get_animation(&animation_name) else {
            continue;
        };

        // Store previous time for trigger/window detection
        let prev_elapsed = animated.elapsed_ms;

        // Update elapsed time
        animated.elapsed_ms += (time.delta_secs() * 1000.0) as u32;
        let current_elapsed = animated.elapsed_ms;

        let total_duration = animation.total_duration_ms();

        // Handle trigger detection based on loop mode
        if total_duration > 0 {
            let (check_start, check_end, wrapped) = match animation.loop_mode {
                LoopMode::Loop => {
                    if current_elapsed >= total_duration && prev_elapsed < total_duration {
                        // Animation wrapped - check from prev to end
                        (prev_elapsed, total_duration, true)
                    } else {
                        (
                            prev_elapsed % total_duration,
                            current_elapsed % total_duration,
                            false,
                        )
                    }
                }
                LoopMode::PingPong => {
                    // PingPong: triggers fire on forward pass only
                    let double_duration = total_duration * 2;
                    let t = current_elapsed % double_duration;
                    if t < total_duration {
                        let prev_t = prev_elapsed % double_duration;
                        if prev_t < total_duration {
                            (prev_t, t, false)
                        } else {
                            // Transitioned from backward to forward
                            (0, t, false)
                        }
                    } else {
                        // Backward pass - skip triggers
                        (0, 0, false)
                    }
                }
                LoopMode::Once => (prev_elapsed, current_elapsed.min(total_duration), false),
            };

            // Fire one-shot triggers
            for trigger in animation.triggers_in_range(check_start, check_end) {
                fire_trigger(
                    &mut commands,
                    entity,
                    &animation_name,
                    trigger,
                    &trigger_registry,
                    &mut trigger_events,
                    &mut sound_events,
                    &mut particle_events,
                    &mut custom_events,
                );
            }

            // Handle wrapped case (fire triggers from start of loop)
            if wrapped {
                let wrapped_time = current_elapsed % total_duration;
                for trigger in animation.triggers_in_range(0, wrapped_time) {
                    fire_trigger(
                        &mut commands,
                        entity,
                        &animation_name,
                        trigger,
                        &trigger_registry,
                        &mut trigger_events,
                        &mut sound_events,
                        &mut particle_events,
                        &mut custom_events,
                    );
                }
            }

            // Handle window events (if tracker component exists)
            if let Some(mut tracker) = tracker_opt {
                let current_time = match animation.loop_mode {
                    LoopMode::Loop => current_elapsed % total_duration,
                    LoopMode::PingPong => {
                        let double_duration = total_duration * 2;
                        let t = current_elapsed % double_duration;
                        if t < total_duration {
                            t
                        } else {
                            double_duration - t
                        }
                    }
                    LoopMode::Once => current_elapsed.min(total_duration),
                };

                process_windows(
                    &mut commands,
                    entity,
                    &animation_name,
                    animation,
                    current_time,
                    &mut tracker,
                    &window_registry,
                    &mut window_events,
                    &mut sound_events,
                    &mut particle_events,
                    &mut custom_events,
                );
                tracker.prev_elapsed_ms = current_elapsed;
            }
        }

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
            if animated.elapsed_ms >= total_duration {
                animated.playing = false;
            }
        }
    }
}

/// Helper to fire a trigger event and convenience events
fn fire_trigger(
    commands: &mut Commands,
    entity: Entity,
    animation_name: &str,
    trigger: &AnimationTrigger,
    trigger_registry: &AnimationTriggerRegistry,
    trigger_events: &mut MessageWriter<AnimationTriggerEvent>,
    sound_events: &mut MessageWriter<AnimationSoundEvent>,
    particle_events: &mut MessageWriter<AnimationParticleEvent>,
    custom_events: &mut MessageWriter<AnimationCustomEvent>,
) {
    // Fire global Message event (for systems that listen to all entities)
    trigger_events.write(AnimationTriggerEvent {
        entity,
        animation: animation_name.to_string(),
        trigger_id: trigger.id,
        trigger_name: trigger.name.clone(),
        payload: trigger.payload.clone(),
    });

    // Fire entity-scoped Observer event (for .observe() handlers)
    commands.trigger(AnimationTriggered {
        entity,
        name: trigger.name.clone(),
        trigger_id: trigger.id,
        animation: animation_name.to_string(),
        time_ms: trigger.time_ms,
        payload: trigger.payload.clone(),
    });

    // Fire typed convenience events and dispatch to registered custom types
    fire_payload_events(
        commands,
        entity,
        animation_name,
        &trigger.payload,
        trigger_registry,
        sound_events,
        particle_events,
        custom_events,
    );
}

/// Helper to process window state changes
fn process_windows(
    commands: &mut Commands,
    entity: Entity,
    animation_name: &str,
    animation: &AnimationDef,
    current_time: u32,
    tracker: &mut WindowTracker,
    window_registry: &AnimationWindowRegistry,
    window_events: &mut MessageWriter<AnimationWindowEvent>,
    sound_events: &mut MessageWriter<AnimationSoundEvent>,
    particle_events: &mut MessageWriter<AnimationParticleEvent>,
    custom_events: &mut MessageWriter<AnimationCustomEvent>,
) {
    for window in &animation.windows {
        let was_active = tracker.active_windows.contains(&window.id);
        let is_active = window.is_active_at(current_time);

        let progress = if is_active && window.duration_ms() > 0 {
            (current_time.saturating_sub(window.start_ms)) as f32 / window.duration_ms() as f32
        } else {
            0.0
        };

        if !was_active && is_active {
            // Begin phase
            tracker.active_windows.insert(window.id);
            fire_window_event(
                commands,
                entity,
                animation_name,
                window,
                WindowPhase::Begin,
                0.0,
                window_registry,
                window_events,
                sound_events,
                particle_events,
                custom_events,
            );
        } else if was_active && is_active {
            // Tick phase
            fire_window_event(
                commands,
                entity,
                animation_name,
                window,
                WindowPhase::Tick,
                progress,
                window_registry,
                window_events,
                sound_events,
                particle_events,
                custom_events,
            );
        } else if was_active && !is_active {
            // End phase
            tracker.active_windows.remove(&window.id);
            fire_window_event(
                commands,
                entity,
                animation_name,
                window,
                WindowPhase::End,
                1.0,
                window_registry,
                window_events,
                sound_events,
                particle_events,
                custom_events,
            );
        }
    }
}

/// Helper to fire a window event
fn fire_window_event(
    commands: &mut Commands,
    entity: Entity,
    animation_name: &str,
    window: &AnimationWindow,
    phase: WindowPhase,
    progress: f32,
    window_registry: &AnimationWindowRegistry,
    window_events: &mut MessageWriter<AnimationWindowEvent>,
    sound_events: &mut MessageWriter<AnimationSoundEvent>,
    particle_events: &mut MessageWriter<AnimationParticleEvent>,
    custom_events: &mut MessageWriter<AnimationCustomEvent>,
) {
    // Fire global Message event (for systems that listen to all entities)
    window_events.write(AnimationWindowEvent {
        entity,
        animation: animation_name.to_string(),
        window_id: window.id,
        window_name: window.name.clone(),
        phase,
        payload: window.payload.clone(),
        progress,
    });

    // Fire entity-scoped Observer event (for .observe() handlers)
    commands.trigger(AnimationWindowChanged {
        entity,
        name: window.name.clone(),
        window_id: window.id,
        animation: animation_name.to_string(),
        phase,
        progress,
        payload: window.payload.clone(),
    });

    // Only fire typed events on Begin (not every tick)
    if phase == WindowPhase::Begin {
        fire_window_payload_events(
            commands,
            entity,
            animation_name,
            phase,
            progress,
            &window.payload,
            window_registry,
            sound_events,
            particle_events,
            custom_events,
        );
    }
}

/// Helper to fire convenience events based on payload type (for triggers)
fn fire_payload_events(
    commands: &mut Commands,
    entity: Entity,
    animation_name: &str,
    payload: &TriggerPayload,
    trigger_registry: &AnimationTriggerRegistry,
    sound_events: &mut MessageWriter<AnimationSoundEvent>,
    particle_events: &mut MessageWriter<AnimationParticleEvent>,
    custom_events: &mut MessageWriter<AnimationCustomEvent>,
) {
    match payload {
        TriggerPayload::Sound { path, volume } => {
            sound_events.write(AnimationSoundEvent {
                entity,
                path: path.clone(),
                volume: *volume,
            });
        }
        TriggerPayload::Particle { effect, offset } => {
            particle_events.write(AnimationParticleEvent {
                entity,
                effect: effect.clone(),
                offset: *offset,
            });
        }
        TriggerPayload::Custom { event_name, params } => {
            // Fire the generic custom event
            custom_events.write(AnimationCustomEvent {
                entity,
                event_name: event_name.clone(),
                params: params.clone(),
            });
            // Also dispatch to registered typed handlers via Bevy Observers
            trigger_registry.dispatch(commands, entity, animation_name, event_name, params);
        }
        TriggerPayload::None => {}
    }
}

/// Helper to fire convenience events based on payload type (for windows)
fn fire_window_payload_events(
    commands: &mut Commands,
    entity: Entity,
    animation_name: &str,
    phase: WindowPhase,
    progress: f32,
    payload: &TriggerPayload,
    window_registry: &AnimationWindowRegistry,
    sound_events: &mut MessageWriter<AnimationSoundEvent>,
    particle_events: &mut MessageWriter<AnimationParticleEvent>,
    custom_events: &mut MessageWriter<AnimationCustomEvent>,
) {
    match payload {
        TriggerPayload::Sound { path, volume } => {
            sound_events.write(AnimationSoundEvent {
                entity,
                path: path.clone(),
                volume: *volume,
            });
        }
        TriggerPayload::Particle { effect, offset } => {
            particle_events.write(AnimationParticleEvent {
                entity,
                effect: effect.clone(),
                offset: *offset,
            });
        }
        TriggerPayload::Custom { event_name, params } => {
            // Fire the generic custom event
            custom_events.write(AnimationCustomEvent {
                entity,
                event_name: event_name.clone(),
                params: params.clone(),
            });
            // Also dispatch to registered typed handlers via Bevy Observers
            window_registry.dispatch(
                commands,
                entity,
                animation_name,
                phase,
                progress,
                event_name,
                params,
            );
        }
        TriggerPayload::None => {}
    }
}
