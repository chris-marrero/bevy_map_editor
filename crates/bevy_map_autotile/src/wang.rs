//! WangFiller Algorithm for Automatic Terrain Tile Selection
//!
//! This module implements the Wang tile filling algorithm for automatic
//! terrain tile selection, matching Tiled's behavior exactly.
//!
//! # Algorithm Overview
//!
//! The WangFiller uses a 3-phase approach:
//! 1. **Build Constraints**: Gather soft preferences from existing tiles and neighbors
//! 2. **Place Tiles + Propagate**: Select tiles and propagate hard constraints to neighbors
//! 3. **Corrections**: Fix edge neighbors that violate constraints (single pass)

use crate::terrain::{TerrainSet, TerrainSetType, TileTerrainData};
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::collections::HashMap;

// =============================================================================
// TerrainId Type
// =============================================================================

/// Terrain color ID (0 = empty/no terrain, 1+ = terrain index + 1)
pub type TerrainId = u8;

// =============================================================================
// WangId - Array-based terrain color storage (Tiled-compatible)
// =============================================================================

/// Position indices for WangId (clockwise from top)
///   7|0|1
///   6|X|2
///   5|4|3
/// - Even indices (0,2,4,6) = Edges (Top, Right, Bottom, Left)
/// - Odd indices (1,3,5,7) = Corners (TopRight, BottomRight, BottomLeft, TopLeft)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WangPosition {
    Top = 0,
    TopRight = 1,
    Right = 2,
    BottomRight = 3,
    Bottom = 4,
    BottomLeft = 5,
    Left = 6,
    TopLeft = 7,
}

impl WangPosition {
    /// Create from index (0-7)
    #[inline]
    pub fn from_index(i: usize) -> Self {
        match i % 8 {
            0 => WangPosition::Top,
            1 => WangPosition::TopRight,
            2 => WangPosition::Right,
            3 => WangPosition::BottomRight,
            4 => WangPosition::Bottom,
            5 => WangPosition::BottomLeft,
            6 => WangPosition::Left,
            7 => WangPosition::TopLeft,
            _ => unreachable!(),
        }
    }

    /// Get the opposite position (across the tile)
    #[inline]
    pub fn opposite(self) -> Self {
        Self::from_index((self as usize + 4) % 8)
    }

    /// Check if this is a corner position (odd indices)
    #[inline]
    pub fn is_corner(self) -> bool {
        (self as u8) % 2 == 1
    }

    /// Get next position clockwise
    #[inline]
    pub fn next(self) -> Self {
        Self::from_index((self as usize + 1) % 8)
    }

    /// Get previous position counter-clockwise
    #[inline]
    pub fn prev(self) -> Self {
        Self::from_index((self as usize + 7) % 8)
    }
}

/// Wang ID using array storage (Tiled-compatible)
/// Color 0 = empty/no terrain
/// Layout: [Top, TopRight, Right, BottomRight, Bottom, BottomLeft, Left, TopLeft]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct WangId {
    pub colors: [TerrainId; 8],
}

impl WangId {
    /// Wildcard WangId (all zeros = matches anything)
    pub const WILDCARD: Self = WangId { colors: [0; 8] };

    /// Create a WangId with all positions set to one terrain
    pub fn filled(terrain: TerrainId) -> Self {
        WangId {
            colors: [terrain; 8],
        }
    }

    /// Get color at position (0 = empty)
    #[inline]
    pub fn color_at(&self, pos: WangPosition) -> TerrainId {
        self.colors[pos as usize]
    }

    /// Set color at position
    #[inline]
    pub fn set_color(&mut self, pos: WangPosition, color: TerrainId) {
        self.colors[pos as usize] = color;
    }

    /// Get color at index (0 = empty)
    #[inline]
    pub fn color_at_index(&self, i: usize) -> TerrainId {
        self.colors[i % 8]
    }

    /// Set color at index
    #[inline]
    pub fn set_color_at_index(&mut self, i: usize, color: TerrainId) {
        self.colors[i % 8] = color;
    }

    /// Get opposite index (position on neighbor that faces us)
    #[inline]
    pub fn opposite_index(i: usize) -> usize {
        (i + 4) % 8
    }

    /// Check if index is a corner (odd indices: 1,3,5,7)
    #[inline]
    pub fn is_corner(i: usize) -> bool {
        i % 2 == 1
    }

    /// Get next index clockwise
    #[inline]
    pub fn next_index(i: usize) -> usize {
        (i + 1) % 8
    }

    /// Get previous index counter-clockwise
    #[inline]
    pub fn prev_index(i: usize) -> usize {
        (i + 7) % 8
    }

    /// Check if this WangId has any terrain set
    pub fn has_any_terrain(&self) -> bool {
        self.colors.iter().any(|&c| c != 0)
    }
}

// =============================================================================
// CellInfo - Constraint information for a single cell (Tiled-compatible)
// =============================================================================

/// Information about constraints for a single cell
#[derive(Clone, Default, Debug)]
pub struct CellInfo {
    /// Desired terrain colors at each position
    pub desired: WangId,
    /// Hard constraint mask - if mask[i] is true, desired.colors[i] MUST match
    pub mask: [bool; 8],
}

impl CellInfo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a hard constraint at a position (must match exactly)
    #[inline]
    pub fn set_constraint(&mut self, pos: WangPosition, color: TerrainId) {
        let idx = pos as usize;
        self.desired.colors[idx] = color;
        self.mask[idx] = true;
    }

    /// Set a hard constraint at an index
    #[inline]
    pub fn set_constraint_at_index(&mut self, i: usize, color: TerrainId) {
        self.desired.colors[i % 8] = color;
        self.mask[i % 8] = true;
    }

    /// Set a soft preference at a position (preferred but not required)
    #[inline]
    pub fn set_preference(&mut self, pos: WangPosition, color: TerrainId) {
        let idx = pos as usize;
        // Only set if not already hard-constrained
        if !self.mask[idx] {
            self.desired.colors[idx] = color;
        }
    }

    /// Set a soft preference at an index
    #[inline]
    pub fn set_preference_at_index(&mut self, i: usize, color: TerrainId) {
        let idx = i % 8;
        if !self.mask[idx] {
            self.desired.colors[idx] = color;
        }
    }

    /// Check if a position has a hard constraint
    #[inline]
    pub fn is_constrained(&self, pos: WangPosition) -> bool {
        self.mask[pos as usize]
    }

    /// Check if an index has a hard constraint
    #[inline]
    pub fn is_constrained_at_index(&self, i: usize) -> bool {
        self.mask[i % 8]
    }
}

// =============================================================================
// Neighbor Offsets
// =============================================================================

/// Neighbor offsets in Y-UP coordinate system, indexed by WangPosition
const NEIGHBOR_OFFSETS: [(i32, i32); 8] = [
    (0, 1),   // 0 = Top
    (1, 1),   // 1 = TopRight
    (1, 0),   // 2 = Right
    (1, -1),  // 3 = BottomRight
    (0, -1),  // 4 = Bottom
    (-1, -1), // 5 = BottomLeft
    (-1, 0),  // 6 = Left
    (-1, 1),  // 7 = TopLeft
];

// =============================================================================
// WangFiller - Main fill algorithm (Tiled-compatible)
// =============================================================================

/// Fills a region with Wang tiles based on constraints
///
/// Uses a 3-phase approach matching Tiled's wangfiller.cpp:
/// 1. Build constraints from existing tiles and neighbors
/// 2. Place tiles and propagate constraints to edge neighbors
/// 3. Single-pass corrections for violated constraints
pub struct WangFiller<'a> {
    terrain_set: &'a TerrainSet,
    /// Grid of cell constraints for the fill region
    cells: HashMap<(i32, i32), CellInfo>,
    /// Queue of cells that need correction (outside the paint region)
    corrections: Vec<(i32, i32)>,
    /// Whether corrections are enabled (set during Phase 2)
    corrections_enabled: bool,
    /// Random number generator for probability-weighted selection
    rng: SmallRng,
}

impl<'a> WangFiller<'a> {
    pub fn new(terrain_set: &'a TerrainSet) -> Self {
        Self {
            terrain_set,
            cells: HashMap::new(),
            corrections: Vec::new(),
            corrections_enabled: false,
            rng: SmallRng::seed_from_u64(0),
        }
    }

    /// Create filler with deterministic seed based on paint position
    pub fn with_seed(terrain_set: &'a TerrainSet, seed: u64) -> Self {
        Self {
            terrain_set,
            cells: HashMap::new(),
            corrections: Vec::new(),
            corrections_enabled: false,
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    /// Get or create cell info at position
    #[inline]
    pub fn get_cell_mut(&mut self, x: i32, y: i32) -> &mut CellInfo {
        self.cells.entry((x, y)).or_default()
    }

    /// Convert TileTerrainData to WangId
    fn tile_terrain_to_wang_id(&self, data: &TileTerrainData) -> WangId {
        let mut wang = WangId::WILDCARD;

        match self.terrain_set.set_type {
            TerrainSetType::Corner => {
                // TileTerrainData for Corner: 0=TL, 1=TR, 2=BL, 3=BR
                // WangId: 7=TopLeft, 1=TopRight, 5=BottomLeft, 3=BottomRight
                if let Some(t) = data.get(0) {
                    wang.colors[7] = (t + 1) as u8; // TL
                }
                if let Some(t) = data.get(1) {
                    wang.colors[1] = (t + 1) as u8; // TR
                }
                if let Some(t) = data.get(2) {
                    wang.colors[5] = (t + 1) as u8; // BL
                }
                if let Some(t) = data.get(3) {
                    wang.colors[3] = (t + 1) as u8; // BR
                }
            }
            TerrainSetType::Edge => {
                // TileTerrainData for Edge: 0=Top, 1=Right, 2=Bottom, 3=Left
                // WangId: 0=Top, 2=Right, 4=Bottom, 6=Left
                if let Some(t) = data.get(0) {
                    wang.colors[0] = (t + 1) as u8; // Top
                }
                if let Some(t) = data.get(1) {
                    wang.colors[2] = (t + 1) as u8; // Right
                }
                if let Some(t) = data.get(2) {
                    wang.colors[4] = (t + 1) as u8; // Bottom
                }
                if let Some(t) = data.get(3) {
                    wang.colors[6] = (t + 1) as u8; // Left
                }
            }
            TerrainSetType::Mixed => {
                // TileTerrainData for Mixed: 0=TL, 1=Top, 2=TR, 3=Right, 4=BR, 5=Bottom, 6=BL, 7=Left
                // WangId: 0=Top, 1=TR, 2=Right, 3=BR, 4=Bottom, 5=BL, 6=Left, 7=TL
                if let Some(t) = data.get(0) {
                    wang.colors[7] = (t + 1) as u8; // TL
                }
                if let Some(t) = data.get(1) {
                    wang.colors[0] = (t + 1) as u8; // Top
                }
                if let Some(t) = data.get(2) {
                    wang.colors[1] = (t + 1) as u8; // TR
                }
                if let Some(t) = data.get(3) {
                    wang.colors[2] = (t + 1) as u8; // Right
                }
                if let Some(t) = data.get(4) {
                    wang.colors[3] = (t + 1) as u8; // BR
                }
                if let Some(t) = data.get(5) {
                    wang.colors[4] = (t + 1) as u8; // Bottom
                }
                if let Some(t) = data.get(6) {
                    wang.colors[5] = (t + 1) as u8; // BL
                }
                if let Some(t) = data.get(7) {
                    wang.colors[6] = (t + 1) as u8; // Left
                }
            }
        }

        wang
    }

    /// Build constraints from the 8 surrounding tiles
    fn wang_id_from_surroundings(
        &self,
        tiles: &[Option<u32>],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) -> WangId {
        let mut result = WangId::WILDCARD;

        for (i, &(dx, dy)) in NEIGHBOR_OFFSETS.iter().enumerate() {
            let nx = x + dx;
            let ny = y + dy;

            if nx >= 0 && ny >= 0 && nx < width as i32 && ny < height as i32 {
                let nidx = (ny as u32 * width + nx as u32) as usize;
                if let Some(tile) = tiles.get(nidx).copied().flatten() {
                    if let Some(terrain_data) = self.terrain_set.get_tile_terrain(tile) {
                        let neighbor_wang = self.tile_terrain_to_wang_id(terrain_data);
                        // Get the opposite position's color from the neighbor
                        let opp_idx = WangId::opposite_index(i);
                        let color = neighbor_wang.colors[opp_idx];
                        if color != 0 {
                            result.colors[i] = color;
                        }
                    }
                }
            }
        }

        result
    }

    /// Score a tile against cell constraints using penalty scoring
    ///
    /// Returns None if the tile violates a hard constraint,
    /// otherwise returns the penalty score (lower is better)
    fn score_tile(&self, cell: &CellInfo, tile_wang: &WangId) -> Option<f32> {
        let mut penalty = 0.0f32;

        for i in 0..8 {
            let want = cell.desired.colors[i];
            let have = tile_wang.colors[i];

            if cell.mask[i] {
                // Hard constraint - must match exactly (0 matches 0)
                if want != have {
                    return None; // Reject tile
                }
            } else if want != 0 && want != have {
                // Soft preference - add penalty for mismatch
                penalty += 1.0;
            }
        }

        Some(penalty)
    }

    /// Find the best matching tile using penalty scoring
    fn find_best_match(&mut self, cell: &CellInfo) -> Option<u32> {
        let mut candidates: Vec<(u32, f32)> = Vec::new();
        let mut best_penalty = f32::MAX;

        for (&tile_id, tile_terrain) in &self.terrain_set.tile_terrains {
            if !tile_terrain.has_any_terrain() {
                continue;
            }

            let tile_wang = self.tile_terrain_to_wang_id(tile_terrain);

            if let Some(penalty) = self.score_tile(cell, &tile_wang) {
                if penalty < best_penalty {
                    best_penalty = penalty;
                    candidates.clear();
                }
                if (penalty - best_penalty).abs() < f32::EPSILON {
                    // Use inverse penalty as probability weight
                    let weight = 1.0 / (1.0 + penalty);
                    candidates.push((tile_id, weight));
                }
            }
        }

        self.random_pick(&candidates)
    }

    /// Pick a random tile from candidates weighted by probability
    fn random_pick(&mut self, candidates: &[(u32, f32)]) -> Option<u32> {
        if candidates.is_empty() {
            return None;
        }

        if candidates.len() == 1 {
            return Some(candidates[0].0);
        }

        let total: f32 = candidates.iter().map(|(_, p)| p).sum();
        if total <= 0.0 {
            return candidates.first().map(|(id, _)| *id);
        }

        let mut random_val = self.rng.gen::<f32>() * total;
        for (tile_id, prob) in candidates {
            random_val -= prob;
            if random_val <= 0.0 {
                return Some(*tile_id);
            }
        }

        candidates.last().map(|(id, _)| *id)
    }

    /// Update adjacent cell's constraints after placing a tile
    fn update_adjacent(&mut self, placed_wang: &WangId, nx: i32, ny: i32, dir_idx: usize) {
        let cell = self.get_cell_mut(nx, ny);
        let opposite_idx = WangId::opposite_index(dir_idx);

        // Set hard constraint on neighbor's opposite position
        cell.mask[opposite_idx] = true;
        cell.desired.colors[opposite_idx] = placed_wang.colors[dir_idx];
    }

    /// Check if a cell violates its hard constraints given a tile's WangId
    fn cell_violates_constraints(&self, cell: &CellInfo, tile_wang: &WangId) -> bool {
        for i in 0..8 {
            if cell.mask[i] {
                let want = cell.desired.colors[i];
                let have = tile_wang.colors[i];
                if want != 0 && want != have {
                    return true;
                }
            }
        }
        false
    }

    /// Apply the filler to a tile layer using 3-phase algorithm
    pub fn apply(
        &mut self,
        tiles: &mut [Option<u32>],
        width: u32,
        height: u32,
        region: &[(i32, i32)],
    ) {
        // Convert region to a set for O(1) lookup
        let region_set: std::collections::HashSet<(i32, i32)> = region.iter().copied().collect();

        // =========================================================================
        // Phase 1: Build Constraints
        // =========================================================================
        for &(x, y) in region {
            let idx = (y as u32 * width + x as u32) as usize;

            // Preserve existing tile attributes as soft preferences
            if let Some(tile_id) = tiles.get(idx).copied().flatten() {
                if let Some(terrain_data) = self.terrain_set.get_tile_terrain(tile_id) {
                    let existing = self.tile_terrain_to_wang_id(terrain_data);
                    let cell = self.get_cell_mut(x, y);

                    for i in 0..8 {
                        if !cell.mask[i] && existing.colors[i] != 0 {
                            cell.desired.colors[i] = existing.colors[i];
                        }
                    }
                }
            }

            // Merge constraints from neighbors
            let around = self.wang_id_from_surroundings(tiles, width, height, x, y);
            let cell = self.get_cell_mut(x, y);

            for i in 0..8 {
                if !cell.mask[i] && around.colors[i] != 0 {
                    cell.desired.colors[i] = around.colors[i];
                }
            }
        }

        // =========================================================================
        // Phase 2: Place Tiles + Propagate
        // =========================================================================
        self.corrections_enabled = true;

        for &(x, y) in region {
            let cell_info = self.cells.get(&(x, y)).cloned().unwrap_or_default();

            if let Some(chosen_tile) = self.find_best_match(&cell_info) {
                let idx = (y as u32 * width + x as u32) as usize;
                tiles[idx] = Some(chosen_tile);

                // Get the WangId of the chosen tile
                let chosen_wang =
                    if let Some(terrain_data) = self.terrain_set.get_tile_terrain(chosen_tile) {
                        self.tile_terrain_to_wang_id(terrain_data)
                    } else {
                        continue;
                    };

                // Propagate to neighbors
                for (dir_idx, &(dx, dy)) in NEIGHBOR_OFFSETS.iter().enumerate() {
                    let nx = x + dx;
                    let ny = y + dy;

                    // Skip if out of bounds
                    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                        continue;
                    }

                    let nidx = (ny as u32 * width + nx as u32) as usize;

                    // Skip if neighbor is empty
                    if tiles.get(nidx).copied().flatten().is_none() {
                        continue;
                    }

                    // Update neighbor constraints
                    self.update_adjacent(&chosen_wang, nx, ny, dir_idx);

                    // Only add for correction if:
                    // - corrections enabled
                    // - edge (not corner) - this matches Tiled's behavior
                    // - outside region
                    // - tile exists and violates constraints
                    if self.corrections_enabled && !WangId::is_corner(dir_idx) {
                        let outside = !region_set.contains(&(nx, ny));
                        if outside {
                            // Check if neighbor violates the new constraint
                            if let Some(neighbor_tile) = tiles.get(nidx).copied().flatten() {
                                if let Some(neighbor_terrain) =
                                    self.terrain_set.get_tile_terrain(neighbor_tile)
                                {
                                    let neighbor_wang =
                                        self.tile_terrain_to_wang_id(neighbor_terrain);
                                    if let Some(cell) = self.cells.get(&(nx, ny)) {
                                        if self.cell_violates_constraints(cell, &neighbor_wang) {
                                            if !self.corrections.contains(&(nx, ny)) {
                                                self.corrections.push((nx, ny));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // =========================================================================
        // Phase 3: Single-Pass Corrections
        // =========================================================================
        let correction_list: Vec<_> = std::mem::take(&mut self.corrections);

        for (x, y) in correction_list {
            // Skip if somehow in region
            if region_set.contains(&(x, y)) {
                continue;
            }

            let idx = (y as u32 * width + x as u32) as usize;

            if let Some(orig_tile) = tiles.get(idx).copied().flatten() {
                if let Some(tile_terrain) = self.terrain_set.get_tile_terrain(orig_tile) {
                    let current_wang = self.tile_terrain_to_wang_id(tile_terrain);

                    if let Some(cell) = self.cells.get(&(x, y)).cloned() {
                        // Check if actually violates constraints
                        if self.cell_violates_constraints(&cell, &current_wang) {
                            // Try to find a better tile
                            if let Some(fix_tile) = self.find_best_match(&cell) {
                                tiles[idx] = Some(fix_tile);
                            }
                        }
                    }
                }
            }
        }
    }
}

// =============================================================================
// Paint Target - Where to paint terrain
// =============================================================================

/// Represents what the terrain brush is painting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaintTarget {
    /// Paint at a corner intersection (affects 4 tiles)
    Corner { corner_x: u32, corner_y: u32 },
    /// Paint at a horizontal edge (between tile rows)
    HorizontalEdge { tile_x: u32, edge_y: u32 },
    /// Paint at a vertical edge (between tile columns)
    VerticalEdge { edge_x: u32, tile_y: u32 },
}

/// Determine the paint target based on mouse position within a tile
pub fn get_paint_target(
    world_x: f32,
    world_y: f32,
    tile_size: f32,
    set_type: TerrainSetType,
) -> PaintTarget {
    let tile_x = (world_x / tile_size).floor() as i32;
    let tile_y = (world_y / tile_size).floor() as i32;

    let local_x = (world_x / tile_size).fract();
    let local_y = (world_y / tile_size).fract();

    // Handle negative fractional parts
    let local_x = if local_x < 0.0 {
        local_x + 1.0
    } else {
        local_x
    };
    let local_y = if local_y < 0.0 {
        local_y + 1.0
    } else {
        local_y
    };

    // Corner-only: always paint corners
    if set_type == TerrainSetType::Corner {
        let corner_x = if local_x < 0.5 { tile_x } else { tile_x + 1 };
        let corner_y = if local_y < 0.5 { tile_y } else { tile_y + 1 };
        return PaintTarget::Corner {
            corner_x: corner_x.max(0) as u32,
            corner_y: corner_y.max(0) as u32,
        };
    }

    // Edge-only: always paint edges
    if set_type == TerrainSetType::Edge {
        let dist_h = (local_y - 0.5).abs();
        let dist_v = (local_x - 0.5).abs();

        if dist_h < dist_v {
            let edge_y = if local_y < 0.5 { tile_y } else { tile_y + 1 };
            return PaintTarget::HorizontalEdge {
                tile_x: tile_x.max(0) as u32,
                edge_y: edge_y.max(0) as u32,
            };
        } else {
            let edge_x = if local_x < 0.5 { tile_x } else { tile_x + 1 };
            return PaintTarget::VerticalEdge {
                edge_x: edge_x.max(0) as u32,
                tile_y: tile_y.max(0) as u32,
            };
        }
    }

    // Mixed: divide tile into 3x3 grid
    let zone_x = if local_x < 0.33 {
        0
    } else if local_x < 0.67 {
        1
    } else {
        2
    };
    let zone_y = if local_y < 0.33 {
        0
    } else if local_y < 0.67 {
        1
    } else {
        2
    };

    match (zone_x, zone_y) {
        (0, 0) => PaintTarget::Corner {
            corner_x: tile_x.max(0) as u32,
            corner_y: tile_y.max(0) as u32,
        },
        (2, 0) => PaintTarget::Corner {
            corner_x: (tile_x + 1).max(0) as u32,
            corner_y: tile_y.max(0) as u32,
        },
        (0, 2) => PaintTarget::Corner {
            corner_x: tile_x.max(0) as u32,
            corner_y: (tile_y + 1).max(0) as u32,
        },
        (2, 2) => PaintTarget::Corner {
            corner_x: (tile_x + 1).max(0) as u32,
            corner_y: (tile_y + 1).max(0) as u32,
        },
        (1, 0) => PaintTarget::HorizontalEdge {
            tile_x: tile_x.max(0) as u32,
            edge_y: tile_y.max(0) as u32,
        },
        (1, 2) => PaintTarget::HorizontalEdge {
            tile_x: tile_x.max(0) as u32,
            edge_y: (tile_y + 1).max(0) as u32,
        },
        (0, 1) => PaintTarget::VerticalEdge {
            edge_x: tile_x.max(0) as u32,
            tile_y: tile_y.max(0) as u32,
        },
        (2, 1) => PaintTarget::VerticalEdge {
            edge_x: (tile_x + 1).max(0) as u32,
            tile_y: tile_y.max(0) as u32,
        },
        // Center defaults to corner
        (1, 1) => PaintTarget::Corner {
            corner_x: tile_x.max(0) as u32,
            corner_y: tile_y.max(0) as u32,
        },
        _ => unreachable!(),
    }
}

// =============================================================================
// Paint Functions
// =============================================================================

/// Paint terrain at a corner intersection
///
/// A corner is at the intersection of 4 tiles. In Y-UP coordinates:
/// - (cx-1, cy-1): Tile below-left, we paint its TopRight corner (index 1)
/// - (cx,   cy-1): Tile below-right, we paint its TopLeft corner (index 7)
/// - (cx-1, cy  ): Tile above-left, we paint its BottomRight corner (index 3)
/// - (cx,   cy  ): Tile above-right, we paint its BottomLeft corner (index 5)
pub fn paint_terrain(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    corner_x: u32,
    corner_y: u32,
    terrain_set: &TerrainSet,
    terrain_index: usize,
) {
    let cx = corner_x as i32;
    let cy = corner_y as i32;
    let color = (terrain_index + 1) as u8;
    let is_mixed = terrain_set.set_type == TerrainSetType::Mixed;

    // Corner affects 4 tiles - map to their specific corner indices
    let affected: [(i32, i32, usize); 4] = [
        (cx - 1, cy - 1, 1), // Tile below-left, TopRight corner
        (cx, cy - 1, 7),     // Tile below-right, TopLeft corner
        (cx - 1, cy, 3),     // Tile above-left, BottomRight corner
        (cx, cy, 5),         // Tile above-right, BottomLeft corner
    ];

    // Seed based on corner position for deterministic results
    let seed = (corner_x as u64) << 32 | (corner_y as u64);
    let mut filler = WangFiller::with_seed(terrain_set, seed);
    let mut region = Vec::new();

    for &(tx, ty, corner_idx) in &affected {
        if tx >= 0 && ty >= 0 && tx < width as i32 && ty < height as i32 {
            let cell = filler.get_cell_mut(tx, ty);

            // Set HARD constraint for the painted corner
            cell.mask[corner_idx] = true;
            cell.desired.colors[corner_idx] = color;

            // For Mixed terrain, also constrain adjacent edges
            if is_mixed {
                let adjacent_edges: [usize; 2] = match corner_idx {
                    1 => [0, 2], // TopRight: Top + Right
                    3 => [2, 4], // BottomRight: Right + Bottom
                    5 => [4, 6], // BottomLeft: Bottom + Left
                    7 => [6, 0], // TopLeft: Left + Top
                    _ => continue,
                };
                for &edge_idx in &adjacent_edges {
                    cell.mask[edge_idx] = true;
                    cell.desired.colors[edge_idx] = color;
                }
            }

            region.push((tx, ty));
        }
    }

    filler.apply(tiles, width, height, &region);
}

/// Paint terrain at a horizontal edge
pub fn paint_terrain_horizontal_edge(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    tile_x: u32,
    edge_y: u32,
    terrain_set: &TerrainSet,
    terrain_index: usize,
) {
    let tx = tile_x as i32;
    let ey = edge_y as i32;
    let color = (terrain_index + 1) as u8;
    let is_mixed = terrain_set.set_type == TerrainSetType::Mixed;

    // Seed based on edge position for deterministic results
    let seed = (tile_x as u64) << 32 | (edge_y as u64) | 0x1000_0000_0000_0000;
    let mut filler = WangFiller::with_seed(terrain_set, seed);
    let mut region = Vec::new();

    // Edge below: set Top (index 0), edge above: set Bottom (index 4)
    let affected: [(i32, i32, usize); 2] = [
        (tx, ey - 1, 0), // Tile below edge, Top
        (tx, ey, 4),     // Tile above edge, Bottom
    ];

    for &(x, y, edge_idx) in &affected {
        if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
            let cell = filler.get_cell_mut(x, y);
            cell.mask[edge_idx] = true;
            cell.desired.colors[edge_idx] = color;

            // For Mixed terrain, also set the adjacent corners
            if is_mixed {
                let adjacent_corners: [usize; 2] = match edge_idx {
                    0 => [7, 1], // Top: TopLeft + TopRight
                    4 => [5, 3], // Bottom: BottomLeft + BottomRight
                    _ => continue,
                };
                for &corner_idx in &adjacent_corners {
                    cell.mask[corner_idx] = true;
                    cell.desired.colors[corner_idx] = color;
                }
            }

            if !region.contains(&(x, y)) {
                region.push((x, y));
            }
        }
    }

    filler.apply(tiles, width, height, &region);
}

/// Paint terrain at a vertical edge
pub fn paint_terrain_vertical_edge(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    edge_x: u32,
    tile_y: u32,
    terrain_set: &TerrainSet,
    terrain_index: usize,
) {
    let ex = edge_x as i32;
    let ty = tile_y as i32;
    let color = (terrain_index + 1) as u8;
    let is_mixed = terrain_set.set_type == TerrainSetType::Mixed;

    // Seed based on edge position for deterministic results
    let seed = (edge_x as u64) << 32 | (tile_y as u64) | 0x2000_0000_0000_0000;
    let mut filler = WangFiller::with_seed(terrain_set, seed);
    let mut region = Vec::new();

    // Left tile: set Right (index 2), right tile: set Left (index 6)
    let affected: [(i32, i32, usize); 2] = [
        (ex - 1, ty, 2), // Tile left of edge, Right
        (ex, ty, 6),     // Tile right of edge, Left
    ];

    for &(x, y, edge_idx) in &affected {
        if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
            let cell = filler.get_cell_mut(x, y);
            cell.mask[edge_idx] = true;
            cell.desired.colors[edge_idx] = color;

            // For Mixed terrain, also set the adjacent corners
            if is_mixed {
                let adjacent_corners: [usize; 2] = match edge_idx {
                    2 => [1, 3], // Right: TopRight + BottomRight
                    6 => [7, 5], // Left: TopLeft + BottomLeft
                    _ => continue,
                };
                for &corner_idx in &adjacent_corners {
                    cell.mask[corner_idx] = true;
                    cell.desired.colors[corner_idx] = color;
                }
            }

            if !region.contains(&(x, y)) {
                region.push((x, y));
            }
        }
    }

    filler.apply(tiles, width, height, &region);
}

/// Unified terrain painting function that handles corners and edges
pub fn paint_terrain_at_target(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    target: PaintTarget,
    terrain_set: &TerrainSet,
    terrain_index: usize,
) {
    match target {
        PaintTarget::Corner { corner_x, corner_y } => {
            paint_terrain(
                tiles,
                width,
                height,
                corner_x,
                corner_y,
                terrain_set,
                terrain_index,
            );
        }
        PaintTarget::HorizontalEdge { tile_x, edge_y } => {
            paint_terrain_horizontal_edge(
                tiles,
                width,
                height,
                tile_x,
                edge_y,
                terrain_set,
                terrain_index,
            );
        }
        PaintTarget::VerticalEdge { edge_x, tile_y } => {
            paint_terrain_vertical_edge(
                tiles,
                width,
                height,
                edge_x,
                tile_y,
                terrain_set,
                terrain_index,
            );
        }
    }
}

/// Update a single tile based on its neighbors
pub fn update_tile_with_neighbors(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    terrain_set: &TerrainSet,
    primary_terrain: usize,
) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }

    let color = (primary_terrain + 1) as u8;
    let mut filler = WangFiller::new(terrain_set);
    let cell = filler.get_cell_mut(x, y);

    // Set all positions as soft preferences
    for i in 0..8 {
        cell.desired.colors[i] = color;
    }

    filler.apply(tiles, width, height, &[(x, y)]);
}

// =============================================================================
// Preview Function
// =============================================================================

/// Get affected region for a paint target
fn get_affected_region(
    target: PaintTarget,
    width: u32,
    height: u32,
    _set_type: TerrainSetType,
) -> Vec<(i32, i32)> {
    let mut tiles = Vec::new();

    match target {
        PaintTarget::Corner { corner_x, corner_y } => {
            let cx = corner_x as i32;
            let cy = corner_y as i32;
            // 4 tiles share this corner (in Y-UP coordinates)
            for (dx, dy) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
                let x = cx + dx;
                let y = cy + dy;
                if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                    tiles.push((x, y));
                }
            }
        }
        PaintTarget::HorizontalEdge { tile_x, edge_y } => {
            let tx = tile_x as i32;
            let ey = edge_y as i32;
            // 2 tiles share this horizontal edge
            if ey > 0 && tx >= 0 && tx < width as i32 && (ey - 1) < height as i32 {
                tiles.push((tx, ey - 1));
            }
            if tx >= 0 && tx < width as i32 && ey >= 0 && ey < height as i32 {
                tiles.push((tx, ey));
            }
        }
        PaintTarget::VerticalEdge { edge_x, tile_y } => {
            let ex = edge_x as i32;
            let ty = tile_y as i32;
            // 2 tiles share this vertical edge
            if ex > 0 && ty >= 0 && ty < height as i32 && (ex - 1) < width as i32 {
                tiles.push((ex - 1, ty));
            }
            if ex >= 0 && ex < width as i32 && ty >= 0 && ty < height as i32 {
                tiles.push((ex, ty));
            }
        }
    }

    tiles
}

/// Calculate preview tiles without modifying actual tile data
pub fn preview_terrain_at_target(
    tiles: &[Option<u32>],
    width: u32,
    height: u32,
    target: PaintTarget,
    terrain_set: &TerrainSet,
    terrain_index: usize,
) -> Vec<((i32, i32), u32)> {
    let affected_region = get_affected_region(target, width, height, terrain_set.set_type);

    if affected_region.is_empty() {
        return Vec::new();
    }

    // Snapshot original tiles in affected region
    let original: HashMap<(i32, i32), Option<u32>> = affected_region
        .iter()
        .map(|&(x, y)| {
            let idx = (y as u32 * width + x as u32) as usize;
            ((x, y), tiles.get(idx).copied().flatten())
        })
        .collect();

    // Clone and apply
    let mut preview_tiles = tiles.to_vec();
    paint_terrain_at_target(
        &mut preview_tiles,
        width,
        height,
        target,
        terrain_set,
        terrain_index,
    );

    // Find changed tiles
    let mut result = Vec::new();
    for (x, y) in affected_region {
        let idx = (y as u32 * width + x as u32) as usize;
        let old = original.get(&(x, y)).copied().flatten();
        let new = preview_tiles.get(idx).copied().flatten();

        if new != old {
            if let Some(tile_id) = new {
                result.push(((x, y), tile_id));
            }
        }
    }

    result
}
