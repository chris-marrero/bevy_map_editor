//! Command pattern for undo/redo

use bevy::prelude::*;
use bevy_map_core::LayerData;
use std::collections::HashMap;
use uuid::Uuid;

use crate::project::Project;
use crate::render::RenderState;

/// A command that can be undone/redone
pub trait Command: Send + Sync {
    /// Execute the command (do/redo)
    fn execute(&self, project: &mut Project, render_state: &mut RenderState);
    /// Undo the command
    fn undo(&self, project: &mut Project, render_state: &mut RenderState);
    /// Get a description of the command
    fn description(&self) -> &str;
}

/// Command for batch tile changes (painting strokes, fills, etc.)
pub struct BatchTileCommand {
    pub level_id: Uuid,
    pub layer_idx: usize,
    /// Changes: (x, y) -> (old_tile, new_tile)
    pub changes: HashMap<(u32, u32), (Option<u32>, Option<u32>)>,
    description: String,
}

impl BatchTileCommand {
    /// Create a new batch tile command
    pub fn new(
        level_id: Uuid,
        layer_idx: usize,
        changes: HashMap<(u32, u32), (Option<u32>, Option<u32>)>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            level_id,
            layer_idx,
            changes,
            description: description.into(),
        }
    }

    /// Create from before/after tile snapshots
    pub fn from_diff(
        level_id: Uuid,
        layer_idx: usize,
        before: HashMap<(u32, u32), Option<u32>>,
        after: HashMap<(u32, u32), Option<u32>>,
        description: impl Into<String>,
    ) -> Self {
        let mut changes = HashMap::new();
        for ((x, y), old_tile) in before {
            let new_tile = after.get(&(x, y)).copied().flatten();
            if old_tile != new_tile {
                changes.insert((x, y), (old_tile, Some(new_tile).flatten()));
            }
        }
        Self::new(level_id, layer_idx, changes, description)
    }
}

impl Command for BatchTileCommand {
    fn execute(&self, project: &mut Project, render_state: &mut RenderState) {
        if let Some(level) = project.get_level_mut(self.level_id) {
            if let Some(layer) = level.layers.get_mut(self.layer_idx) {
                if let LayerData::Tiles { tiles, .. } = &mut layer.data {
                    for ((x, y), (_, new_tile)) in &self.changes {
                        let idx = (*y * level.width + *x) as usize;
                        if idx < tiles.len() {
                            tiles[idx] = *new_tile;
                        }
                    }
                }
            }
        }
        render_state.needs_rebuild = true;
    }

    fn undo(&self, project: &mut Project, render_state: &mut RenderState) {
        if let Some(level) = project.get_level_mut(self.level_id) {
            if let Some(layer) = level.layers.get_mut(self.layer_idx) {
                if let LayerData::Tiles { tiles, .. } = &mut layer.data {
                    for ((x, y), (old_tile, _)) in &self.changes {
                        let idx = (*y * level.width + *x) as usize;
                        if idx < tiles.len() {
                            tiles[idx] = *old_tile;
                        }
                    }
                }
            }
        }
        render_state.needs_rebuild = true;
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Collect tiles in a rectangular region for undo tracking
pub fn collect_tiles_in_region(
    project: &Project,
    level_id: Uuid,
    layer_idx: usize,
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
) -> HashMap<(u32, u32), Option<u32>> {
    let mut tiles = HashMap::new();

    if let Some(level) = project.levels.iter().find(|l| l.id == level_id) {
        if let Some(layer) = level.layers.get(layer_idx) {
            if let LayerData::Tiles {
                tiles: tile_data, ..
            } = &layer.data
            {
                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        if x >= 0 && y >= 0 && x < level.width as i32 && y < level.height as i32 {
                            let idx = (y as u32 * level.width + x as u32) as usize;
                            let tile = tile_data.get(idx).copied().flatten();
                            tiles.insert((x as u32, y as u32), tile);
                        }
                    }
                }
            }
        }
    }

    tiles
}

/// Command for tile changes produced by running automap rules.
///
/// Covers potentially many layers in one undoable operation. Each entry in
/// `layer_changes` maps a layer index to a per-cell change map:
/// `(x, y) -> (old_tile, new_tile)`.
///
/// # Invariants
///
/// - `execute` and `undo` are pure: they operate only on `&mut Project` and
///   `&mut RenderState`, with no Bevy API calls.
/// - A layer index that does not exist on the level is silently skipped.
///   This matches `BatchTileCommand`'s defensive behaviour.
/// - Cells whose coordinates are out of bounds for the level are silently
///   skipped. This guards against stale commands after a level resize.
pub struct AutomapCommand {
    pub level_id: Uuid,
    /// Per-layer cell changes: layer_index -> { (x, y) -> (old_tile, new_tile) }
    pub layer_changes: HashMap<usize, HashMap<(u32, u32), (Option<u32>, Option<u32>)>>,
    description: String,
}

impl AutomapCommand {
    /// Create a new automap command.
    ///
    /// `description` is the human-readable undo label shown in the Edit menu.
    pub fn new(
        level_id: Uuid,
        layer_changes: HashMap<usize, HashMap<(u32, u32), (Option<u32>, Option<u32>)>>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            level_id,
            layer_changes,
            description: description.into(),
        }
    }

    /// Returns `true` if there are no cell changes recorded.
    ///
    /// A command with no changes is still valid to push to history but will
    /// have no visible effect. Callers may wish to skip pushing it.
    pub fn is_empty(&self) -> bool {
        self.layer_changes.values().all(|cells| cells.is_empty())
    }
}

impl Command for AutomapCommand {
    fn execute(&self, project: &mut Project, render_state: &mut RenderState) {
        if let Some(level) = project.get_level_mut(self.level_id) {
            let level_width = level.width;
            for (layer_idx, cell_changes) in &self.layer_changes {
                if let Some(layer) = level.layers.get_mut(*layer_idx) {
                    if let LayerData::Tiles { tiles, .. } = &mut layer.data {
                        for ((x, y), (_, new_tile)) in cell_changes {
                            let idx = (*y * level_width + *x) as usize;
                            if idx < tiles.len() {
                                tiles[idx] = *new_tile;
                            }
                        }
                    }
                }
            }
        }
        render_state.needs_rebuild = true;
    }

    fn undo(&self, project: &mut Project, render_state: &mut RenderState) {
        if let Some(level) = project.get_level_mut(self.level_id) {
            let level_width = level.width;
            for (layer_idx, cell_changes) in &self.layer_changes {
                if let Some(layer) = level.layers.get_mut(*layer_idx) {
                    if let LayerData::Tiles { tiles, .. } = &mut layer.data {
                        for ((x, y), (old_tile, _)) in cell_changes {
                            let idx = (*y * level_width + *x) as usize;
                            if idx < tiles.len() {
                                tiles[idx] = *old_tile;
                            }
                        }
                    }
                }
            }
        }
        render_state.needs_rebuild = true;
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Command for moving an entity to a new position
pub struct MoveEntityCommand {
    pub level_id: Uuid,
    pub entity_id: Uuid,
    pub old_position: [f32; 2],
    pub new_position: [f32; 2],
}

impl MoveEntityCommand {
    pub fn new(
        level_id: Uuid,
        entity_id: Uuid,
        old_position: [f32; 2],
        new_position: [f32; 2],
    ) -> Self {
        Self {
            level_id,
            entity_id,
            old_position,
            new_position,
        }
    }
}

impl Command for MoveEntityCommand {
    fn execute(&self, project: &mut Project, _render_state: &mut RenderState) {
        if let Some(level) = project.get_level_mut(self.level_id) {
            if let Some(entity) = level.entities.iter_mut().find(|e| e.id == self.entity_id) {
                entity.position = self.new_position;
            }
        }
    }

    fn undo(&self, project: &mut Project, _render_state: &mut RenderState) {
        if let Some(level) = project.get_level_mut(self.level_id) {
            if let Some(entity) = level.entities.iter_mut().find(|e| e.id == self.entity_id) {
                entity.position = self.old_position;
            }
        }
    }

    fn description(&self) -> &str {
        "Move Entity"
    }
}

/// Stores command history for undo/redo
#[derive(Resource, Default)]
pub struct CommandHistory {
    /// Stack of commands that have been executed
    undo_stack: Vec<Box<dyn Command>>,
    /// Stack of commands that have been undone
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandHistory {
    /// Execute a command and add it to history
    pub fn execute(
        &mut self,
        command: Box<dyn Command>,
        project: &mut Project,
        render_state: &mut RenderState,
    ) {
        command.execute(project, render_state);
        self.undo_stack.push(command);
        self.redo_stack.clear(); // Clear redo stack on new command
        project.mark_dirty();
    }

    /// Undo the last command
    pub fn undo(&mut self, project: &mut Project, render_state: &mut RenderState) {
        if let Some(command) = self.undo_stack.pop() {
            command.undo(project, render_state);
            self.redo_stack.push(command);
            project.mark_dirty();
        }
    }

    /// Redo the last undone command
    pub fn redo(&mut self, project: &mut Project, render_state: &mut RenderState) {
        if let Some(command) = self.redo_stack.pop() {
            command.execute(project, render_state);
            self.undo_stack.push(command);
            project.mark_dirty();
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get description of command to undo
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|c| c.description())
    }

    /// Get description of command to redo
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|c| c.description())
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Push a command directly onto the undo stack without executing it.
    /// Use this when the changes have already been applied (e.g., during painting).
    pub fn push_undo(&mut self, command: Box<dyn Command>) {
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }
}
