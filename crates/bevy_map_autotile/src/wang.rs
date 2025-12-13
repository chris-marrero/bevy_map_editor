//! Tiled-Style WangFiller Algorithm
//!
//! This module implements the Wang tile filling algorithm for automatic
//! terrain tile selection, compatible with Tiled's terrain system.

use crate::terrain::{TerrainSet, TerrainSetType, TileTerrainData};
use std::collections::HashMap;

/// Wang ID representing terrain colors at all 8 positions
/// Uses Tiled's position indexing:
///   7|0|1
///   6|X|2
///   5|4|3
/// - Even indices (0,2,4,6) = Edges (Top, Right, Bottom, Left)
/// - Odd indices (1,3,5,7) = Corners (TopRight, BottomRight, BottomLeft, TopLeft)
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct WangId {
    /// 8 positions: Top=0, TopRight=1, Right=2, BottomRight=3, Bottom=4, BottomLeft=5, Left=6, TopLeft=7
    /// None = wildcard (any terrain matches)
    pub colors: [Option<usize>; 8],
}

impl WangId {
    pub const WILDCARD: Self = WangId { colors: [None; 8] };

    /// Create a WangId with all positions set to one terrain
    pub fn filled(terrain: usize) -> Self {
        WangId {
            colors: [Some(terrain); 8],
        }
    }

    /// Get opposite index (position on neighbor that faces us)
    pub fn opposite_index(i: usize) -> usize {
        (i + 4) % 8
    }

    /// Check if index is a corner (odd indices: 1,3,5,7)
    pub fn is_corner(i: usize) -> bool {
        i % 2 == 1
    }

    /// Get next index clockwise
    pub fn next_index(i: usize) -> usize {
        (i + 1) % 8
    }

    /// Get previous index counter-clockwise
    pub fn prev_index(i: usize) -> usize {
        (i + 7) % 8
    }
}

/// Information about constraints for a single cell
#[derive(Clone, Default, Debug)]
pub struct CellInfo {
    /// Desired terrain colors at each position
    pub desired: WangId,
    /// Which positions are hard-constrained (must match exactly)
    pub mask: [bool; 8],
}

/// Fills a region with Wang tiles based on constraints
/// This is a port of Tiled's WangFiller algorithm
pub struct WangFiller<'a> {
    terrain_set: &'a TerrainSet,
    /// Grid of cell constraints for the fill region
    cells: HashMap<(i32, i32), CellInfo>,
    /// Corrections queue for tiles that need re-evaluation
    corrections: Vec<(i32, i32)>,
    /// Whether corrections are enabled (starts false, enabled after initial pass)
    corrections_enabled: bool,
}

impl<'a> WangFiller<'a> {
    pub fn new(terrain_set: &'a TerrainSet) -> Self {
        Self {
            terrain_set,
            cells: HashMap::new(),
            corrections: Vec::new(),
            corrections_enabled: false,
        }
    }

    /// Get or create cell info at position
    pub fn get_cell_mut(&mut self, x: i32, y: i32) -> &mut CellInfo {
        self.cells.entry((x, y)).or_default()
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

        // Neighbor offsets in clockwise order matching WangId positions
        // Top, TopRight, Right, BottomRight, Bottom, BottomLeft, Left, TopLeft
        // Y-UP coordinate system: +Y is up (above), -Y is down (below)
        let offsets: [(i32, i32); 8] = [
            (0, 1),    // 0 = Top (neighbor above)
            (1, 1),    // 1 = TopRight
            (1, 0),    // 2 = Right
            (1, -1),   // 3 = BottomRight
            (0, -1),   // 4 = Bottom (neighbor below)
            (-1, -1),  // 5 = BottomLeft
            (-1, 0),   // 6 = Left
            (-1, 1),   // 7 = TopLeft
        ];

        let mut neighbor_wangids: [WangId; 8] = [WangId::WILDCARD; 8];

        // Get WangId of each neighbor
        for (i, (dx, dy)) in offsets.iter().enumerate() {
            let nx = x + dx;
            let ny = y + dy;

            if nx >= 0 && ny >= 0 && nx < width as i32 && ny < height as i32 {
                let nidx = (ny as u32 * width + nx as u32) as usize;
                if let Some(tile) = tiles.get(nidx).copied().flatten() {
                    if let Some(terrain_data) = self.terrain_set.get_tile_terrain(tile) {
                        // Convert from our internal format to WangId
                        neighbor_wangids[i] = self.tile_terrain_to_wang_id(terrain_data);
                    }
                }
            }
        }

        // Get edge colors from opposite sides of neighbors
        for i in [0, 2, 4, 6] {
            // Top, Right, Bottom, Left edges
            let opp = WangId::opposite_index(i);
            result.colors[i] = neighbor_wangids[i].colors[opp];
        }

        // Get corner colors with fallback logic (like Tiled)
        for i in [1, 3, 5, 7] {
            // TopRight, BottomRight, BottomLeft, TopLeft corners
            let opp = WangId::opposite_index(i);
            let mut color = neighbor_wangids[i].colors[opp];

            // Fallback 1: Get from left neighbor's corner
            if color.is_none() {
                let left_idx = WangId::prev_index(i);
                let left_corner = (i + 2) % 8;
                color = neighbor_wangids[left_idx].colors[left_corner];
            }

            // Fallback 2: Get from right neighbor's corner
            if color.is_none() {
                let right_idx = WangId::next_index(i);
                let right_corner = (i + 6) % 8;
                color = neighbor_wangids[right_idx].colors[right_corner];
            }

            result.colors[i] = color;
        }

        result
    }

    /// Convert TileTerrainData to WangId using Tiled's position mapping
    fn tile_terrain_to_wang_id(&self, data: &TileTerrainData) -> WangId {
        let mut wang_id = WangId::WILDCARD;

        match self.terrain_set.set_type {
            TerrainSetType::Corner => {
                // Our Corner: 0=TL, 1=TR, 2=BL, 3=BR
                // Tiled corners: 7=TopLeft, 1=TopRight, 5=BottomLeft, 3=BottomRight
                wang_id.colors[7] = data.get(0); // TL
                wang_id.colors[1] = data.get(1); // TR
                wang_id.colors[5] = data.get(2); // BL
                wang_id.colors[3] = data.get(3); // BR
            }
            TerrainSetType::Edge => {
                // Our Edge: 0=Top, 1=Right, 2=Bottom, 3=Left
                // Tiled edges: 0=Top, 2=Right, 4=Bottom, 6=Left
                wang_id.colors[0] = data.get(0); // Top
                wang_id.colors[2] = data.get(1); // Right
                wang_id.colors[4] = data.get(2); // Bottom
                wang_id.colors[6] = data.get(3); // Left
            }
            TerrainSetType::Mixed => {
                // Our Mixed: 0=TL, 1=Top, 2=TR, 3=Right, 4=BR, 5=Bottom, 6=BL, 7=Left
                // Tiled: 0=Top, 1=TR, 2=Right, 3=BR, 4=Bottom, 5=BL, 6=Left, 7=TL
                wang_id.colors[7] = data.get(0); // TL corner
                wang_id.colors[0] = data.get(1); // Top edge
                wang_id.colors[1] = data.get(2); // TR corner
                wang_id.colors[2] = data.get(3); // Right edge
                wang_id.colors[3] = data.get(4); // BR corner
                wang_id.colors[4] = data.get(5); // Bottom edge
                wang_id.colors[5] = data.get(6); // BL corner
                wang_id.colors[6] = data.get(7); // Left edge
            }
        }

        wang_id
    }

    /// Find the best tile matching constraints using penalty scoring
    fn find_best_match(&self, info: &CellInfo) -> Option<u32> {
        let mut best_tile = None;
        let mut lowest_penalty = i32::MAX;

        for (&tile_id, tile_terrain) in &self.terrain_set.tile_terrains {
            if !tile_terrain.has_any_terrain() {
                continue;
            }

            let tile_wang_id = self.tile_terrain_to_wang_id(tile_terrain);

            // Check hard constraints first
            let mut matches_hard = true;
            for i in 0..8 {
                if info.mask[i] {
                    let desired = info.desired.colors[i];
                    let actual = tile_wang_id.colors[i];
                    if desired.is_some() && desired != actual {
                        matches_hard = false;
                        break;
                    }
                }
            }

            if !matches_hard {
                continue;
            }

            // Calculate penalty for soft preferences
            let mut penalty = 0i32;
            for i in 0..8 {
                if !info.mask[i] {
                    if let Some(desired) = info.desired.colors[i] {
                        let actual = tile_wang_id.colors[i];
                        if Some(desired) != actual {
                            penalty += 1;
                        }
                    }
                }
            }

            if penalty < lowest_penalty {
                lowest_penalty = penalty;
                best_tile = Some(tile_id);
            }
        }

        best_tile
    }

    /// Update adjacent cell's constraints based on a placed tile
    fn update_adjacent(
        &mut self,
        wang_id: &WangId,
        adj_x: i32,
        adj_y: i32,
        direction_index: usize,
    ) {
        // CRITICAL FIX: For corner-only terrain sets, do NOT propagate ANY constraints
        // to neighbors. Corner constraints are already fully set by the paint function
        // (which sets the corner on all 4 tiles sharing that corner). Edges don't exist
        // in corner-only sets, so there's nothing to propagate.
        if self.terrain_set.set_type == TerrainSetType::Corner {
            return;
        }

        let opp = WangId::opposite_index(direction_index);
        let cell = self.get_cell_mut(adj_x, adj_y);

        // Set the opposite position on the neighbor
        cell.desired.colors[opp] = wang_id.colors[direction_index];
        cell.mask[opp] = true;

        // If this is an EDGE (not corner), also update adjacent corners as SOFT preferences
        // CRITICAL FIX: Do NOT set mask=true for corners propagated from edges.
        // This prevents correction cascades that cause tiles too far away to be affected.
        // The tile matching algorithm will still prefer these values, but they won't
        // trigger hard constraint violations that force distant tile corrections.
        if !WangId::is_corner(opp) {
            let corner_a = WangId::next_index(opp);
            let corner_b = WangId::prev_index(opp);

            let adj_corner_a = WangId::prev_index(direction_index);
            let adj_corner_b = WangId::next_index(direction_index);

            // Only set if not already constrained (soft preference)
            if cell.desired.colors[corner_a].is_none() {
                cell.desired.colors[corner_a] = wang_id.colors[adj_corner_a];
                // Do NOT set mask[corner_a] = true - keep as soft constraint
            }

            if cell.desired.colors[corner_b].is_none() {
                cell.desired.colors[corner_b] = wang_id.colors[adj_corner_b];
                // Do NOT set mask[corner_b] = true - keep as soft constraint
            }
        }
    }

    /// Apply the filler to a tile layer
    pub fn apply(
        &mut self,
        tiles: &mut [Option<u32>],
        width: u32,
        height: u32,
        region: &[(i32, i32)], // Positions to fill
    ) {
        // Neighbor offsets matching WangId positions
        // Y-UP coordinate system: +Y is up (above), -Y is down (below)
        let offsets: [(i32, i32); 8] = [
            (0, 1),    // 0 = Top (neighbor above)
            (1, 1),    // 1 = TopRight
            (1, 0),    // 2 = Right
            (1, -1),   // 3 = BottomRight
            (0, -1),   // 4 = Bottom (neighbor below)
            (-1, -1),  // 5 = BottomLeft
            (-1, 0),   // 6 = Left
            (-1, 1),   // 7 = TopLeft
        ];

        // Phase 1: Set border constraints from outside tiles AND preserve current tile's terrain
        for &(x, y) in region {
            // Track which positions we preserve from the current tile (soft preferences)
            let mut preserved_positions = [false; 8];

            // First, get the current tile's terrain (if any) to preserve non-painted corners
            let idx = (y as u32 * width + x as u32) as usize;
            if let Some(current_tile) = tiles.get(idx).copied().flatten() {
                if let Some(current_terrain) = self.terrain_set.get_tile_terrain(current_tile) {
                    let current_wang_id = self.tile_terrain_to_wang_id(current_terrain);
                    let cell = self.get_cell_mut(x, y);

                    // Preserve current tile's terrain for positions not explicitly set
                    // Keep as SOFT preferences (don't set mask) but track them
                    for (i, preserved) in preserved_positions.iter_mut().enumerate() {
                        if !cell.mask[i] && current_wang_id.colors[i].is_some() {
                            cell.desired.colors[i] = current_wang_id.colors[i];
                            *preserved = true;
                        }
                    }
                }
            }

            // Then get constraints from neighboring tiles
            let surroundings = self.wang_id_from_surroundings(tiles, width, height, x, y);
            let cell = self.get_cell_mut(x, y);

            // Merge surroundings but don't override hard constraints OR preserved values
            for (i, &preserved) in preserved_positions.iter().enumerate() {
                if !cell.mask[i] && !preserved && surroundings.colors[i].is_some() {
                    cell.desired.colors[i] = surroundings.colors[i];
                }
            }
        }

        // Enable corrections for the resolution phase
        // This ensures neighbor tiles outside the paint region get corrected
        self.corrections_enabled = true;

        // Phase 2: Resolve tiles
        for &(x, y) in region {
            let cell = self.cells.get(&(x, y)).cloned().unwrap_or_default();

            if let Some(new_tile) = self.find_best_match(&cell) {
                let idx = (y as u32 * width + x as u32) as usize;
                tiles[idx] = Some(new_tile);

                // Get WangId of placed tile
                let placed_wang_id = self
                    .terrain_set
                    .get_tile_terrain(new_tile)
                    .map(|t| self.tile_terrain_to_wang_id(t))
                    .unwrap_or(WangId::WILDCARD);

                // Update neighbors
                for (i, (dx, dy)) in offsets.iter().enumerate() {
                    let nx = x + dx;
                    let ny = y + dy;

                    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                        continue;
                    }

                    // Only update neighbors that ALREADY have terrain tiles
                    // This prevents painting tiles on empty cells
                    let nidx = (ny as u32 * width + nx as u32) as usize;
                    if tiles.get(nidx).copied().flatten().is_none() {
                        continue;
                    }

                    self.update_adjacent(&placed_wang_id, nx, ny, i);

                    // Queue for corrections only if:
                    // 1. It's outside the paint region
                    // 2. The current tile doesn't match the new constraints (like Tiled)
                    // CRITICAL FIX: Corner-only terrain sets should NEVER queue corrections
                    // because there are no edge constraints to propagate.
                    // For other sets, only queue for EDGE adjacency (positions 0,2,4,6),
                    // NOT for corner adjacency (positions 1,3,5,7). This matches Tiled's:
                    // `if (!WangId::isCorner(i) && mCorrectionsEnabled ...)`
                    let is_corner_only = self.terrain_set.set_type == TerrainSetType::Corner;
                    let is_edge_adjacency = !WangId::is_corner(i);
                    if self.corrections_enabled && !is_corner_only && is_edge_adjacency {
                        let region_contains = region.contains(&(nx, ny));
                        if !region_contains {
                            // Check if current tile actually needs correction
                            if let Some(neighbor_tile) = tiles.get(nidx).copied().flatten() {
                                if let Some(neighbor_terrain) = self.terrain_set.get_tile_terrain(neighbor_tile) {
                                    let neighbor_wang_id = self.tile_terrain_to_wang_id(neighbor_terrain);
                                    let cell = self.cells.get(&(nx, ny));

                                    // Only queue if constraints don't match
                                    if let Some(cell_info) = cell {
                                        let needs_correction = (0..8).any(|pos| {
                                            cell_info.mask[pos] &&
                                            cell_info.desired.colors[pos] != neighbor_wang_id.colors[pos]
                                        });
                                        if needs_correction {
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

        // Phase 3: Single-pass corrections (no recursive propagation)
        // Only fix tiles that VIOLATE hard constraints, don't propagate further
        let processed: std::collections::HashSet<(i32, i32)> = region.iter().copied().collect();
        let final_corrections: Vec<_> = self.corrections.drain(..).collect();

        for (x, y) in final_corrections {
            if processed.contains(&(x, y)) {
                continue;
            }

            let cell = self.cells.get(&(x, y)).cloned().unwrap_or_default();
            let idx = (y as u32 * width + x as u32) as usize;

            // Only fix if current tile actually VIOLATES hard constraints
            if let Some(current_tile) = tiles.get(idx).copied().flatten() {
                if let Some(current_terrain) = self.terrain_set.get_tile_terrain(current_tile) {
                    let current_wang_id = self.tile_terrain_to_wang_id(current_terrain);

                    // Check if any hard constraint is violated
                    let violates_hard_constraint = (0..8).any(|pos| {
                        cell.mask[pos] &&
                        cell.desired.colors[pos].is_some() &&
                        cell.desired.colors[pos] != current_wang_id.colors[pos]
                    });

                    // Only fix if a hard constraint is violated
                    if violates_hard_constraint {
                        if let Some(new_tile) = self.find_best_match(&cell) {
                            tiles[idx] = Some(new_tile);
                            // DO NOT propagate - single pass only
                        }
                    }
                }
            }
        }
    }
}

/// Represents what the terrain brush is painting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaintTarget {
    /// Paint at a corner intersection (affects 4 tiles)
    /// Coordinates are corner indices (between tiles)
    Corner { corner_x: u32, corner_y: u32 },
    /// Paint at an edge (horizontal edge between rows of tiles)
    /// tile_x is the tile column, edge_y is the edge row (between tile rows)
    HorizontalEdge { tile_x: u32, edge_y: u32 },
    /// Paint at an edge (vertical edge between columns of tiles)
    /// edge_x is the edge column (between tile columns), tile_y is the tile row
    VerticalEdge { edge_x: u32, tile_y: u32 },
}

/// Determine the paint target based on mouse position within a tile
pub fn get_paint_target(
    world_x: f32,
    world_y: f32,
    tile_size: f32,
    set_type: TerrainSetType,
) -> PaintTarget {
    // Calculate tile coordinates (Y-up coordinate system with BottomLeft anchor)
    let tile_x = (world_x / tile_size).floor() as i32;
    let tile_y = (world_y / tile_size).floor() as i32;

    // Calculate position within the tile (0.0 to 1.0)
    let local_x = (world_x / tile_size).fract();
    let local_y = (world_y / tile_size).fract();

    // Handle negative fractional parts
    let local_x = if local_x < 0.0 { local_x + 1.0 } else { local_x };
    let local_y = if local_y < 0.0 { local_y + 1.0 } else { local_y };

    // For Corner-only sets, always paint corners
    if set_type == TerrainSetType::Corner {
        let corner_x = if local_x < 0.5 { tile_x } else { tile_x + 1 };
        let corner_y = if local_y < 0.5 { tile_y } else { tile_y + 1 };
        return PaintTarget::Corner {
            corner_x: corner_x.max(0) as u32,
            corner_y: corner_y.max(0) as u32,
        };
    }

    // For Edge-only sets, always paint edges
    if set_type == TerrainSetType::Edge {
        let dist_to_horizontal = (local_y - 0.5).abs();
        let dist_to_vertical = (local_x - 0.5).abs();

        if dist_to_horizontal < dist_to_vertical {
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

    // For Mixed sets, divide tile into 3x3 grid
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
        // Corners
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
        // Edges
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
        // Center - default to corner
        (1, 1) => PaintTarget::Corner {
            corner_x: tile_x.max(0) as u32,
            corner_y: tile_y.max(0) as u32,
        },
        _ => unreachable!(),
    }
}

/// Paint terrain at a corner intersection using Tiled-style approach
pub fn paint_terrain(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    terrain_set: &TerrainSet,
    terrain_index: usize,
) {
    let cx = x as i32;
    let cy = y as i32;

    // Define affected tiles and which corner position to set on each
    // WangId positions: 0=Top, 1=TR, 2=Right, 3=BR, 4=Bottom, 5=BL, 6=Left, 7=TL
    // Using Y-up coordinates (BottomLeft anchor): cy-1 is below, cy is above
    let affected_tiles: [(i32, i32, usize); 4] = [
        (cx - 1, cy - 1, 1), // Tile below-left: set TR corner (corner is at its top-right)
        (cx, cy - 1, 7),     // Tile below-right: set TL corner (corner is at its top-left)
        (cx - 1, cy, 3),     // Tile above-left: set BR corner (corner is at its bottom-right)
        (cx, cy, 5),         // Tile above-right: set BL corner (corner is at its bottom-left)
    ];

    let mut filler = WangFiller::new(terrain_set);
    let mut region = Vec::new();

    // Set constraints for each affected tile
    for &(tx, ty, corner_pos) in &affected_tiles {
        if tx >= 0 && ty >= 0 && tx < width as i32 && ty < height as i32 {
            let cell = filler.get_cell_mut(tx, ty);
            cell.desired.colors[corner_pos] = Some(terrain_index);
            cell.mask[corner_pos] = true;
            region.push((tx, ty));
        }
    }

    // Apply to all affected tiles
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

    let mut filler = WangFiller::new(terrain_set);
    let mut region = Vec::new();

    // Y-up coordinates: ey-1 is below, ey is above the edge
    // Following Tiled's approach: only set the EDGE positions, not corners
    // Corners will be determined by the tile matching algorithm
    let affected: Vec<(i32, i32, Vec<usize>)> = vec![
        (tx, ey - 1, vec![0]),           // Tile below: set Top edge only
        (tx, ey, vec![4]),               // Tile above: set Bottom edge only
    ];

    for (tile_x, tile_y, positions) in &affected {
        if *tile_x >= 0 && *tile_y >= 0 && *tile_x < width as i32 && *tile_y < height as i32 {
            let cell = filler.get_cell_mut(*tile_x, *tile_y);
            for &pos in positions {
                cell.desired.colors[pos] = Some(terrain_index);
                cell.mask[pos] = true;
            }
            if !region.contains(&(*tile_x, *tile_y)) {
                region.push((*tile_x, *tile_y));
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

    let mut filler = WangFiller::new(terrain_set);
    let mut region = Vec::new();

    // Following Tiled's approach: only set the EDGE positions, not corners
    // Corners will be determined by the tile matching algorithm
    let affected: Vec<(i32, i32, Vec<usize>)> = vec![
        (ex - 1, ty, vec![2]),           // Left tile: set Right edge only
        (ex, ty, vec![6]),               // Right tile: set Left edge only
    ];

    for (tile_x, tile_y, positions) in &affected {
        if *tile_x >= 0 && *tile_y >= 0 && *tile_x < width as i32 && *tile_y < height as i32 {
            let cell = filler.get_cell_mut(*tile_x, *tile_y);
            for &pos in positions {
                cell.desired.colors[pos] = Some(terrain_index);
                cell.mask[pos] = true;
            }
            if !region.contains(&(*tile_x, *tile_y)) {
                region.push((*tile_x, *tile_y));
            }
        }
    }

    filler.apply(tiles, width, height, &region);
}

/// Unified terrain painting function that handles corners and edges based on PaintTarget
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

    let mut filler = WangFiller::new(terrain_set);
    let cell = filler.get_cell_mut(x, y);

    // Set all positions to the primary terrain as soft preference
    for i in 0..8 {
        cell.desired.colors[i] = Some(primary_terrain);
    }

    filler.apply(tiles, width, height, &[(x, y)]);
}

/// Get the tiles potentially affected by a paint target (including neighbors for corrections)
/// Returns positions in a local region around the target
fn get_affected_region(
    target: PaintTarget,
    width: u32,
    height: u32,
    set_type: TerrainSetType,
) -> Vec<(i32, i32)> {
    let (center_x, center_y) = match target {
        PaintTarget::Corner { corner_x, corner_y } => (corner_x as i32, corner_y as i32),
        PaintTarget::HorizontalEdge { tile_x, edge_y } => (tile_x as i32, edge_y as i32),
        PaintTarget::VerticalEdge { edge_x, tile_y } => (edge_x as i32, tile_y as i32),
    };

    // Use smaller radius for corner-only sets (no edge propagation needed)
    // Corner-only: radius 1 = 3x3 (just the 4 tiles around the corner + center)
    // Edge/Mixed: radius 2 = 5x5 (edge propagation may affect more tiles)
    let radius = match set_type {
        TerrainSetType::Corner => 1,
        TerrainSetType::Edge | TerrainSetType::Mixed => 2,
    };

    let mut positions = Vec::new();
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = center_x + dx;
            let y = center_y + dy;
            if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                positions.push((x, y));
            }
        }
    }
    positions
}

/// Calculate preview tiles without modifying actual tile data
/// Returns list of (position, tile_id) pairs showing what would be placed
/// Optimized to only clone and scan a small region around the paint target
pub fn preview_terrain_at_target(
    tiles: &[Option<u32>],
    width: u32,
    height: u32,
    target: PaintTarget,
    terrain_set: &TerrainSet,
    terrain_index: usize,
) -> Vec<((i32, i32), u32)> {
    // Get the region that could be affected by this paint operation
    let affected_region = get_affected_region(target, width, height, terrain_set.set_type);

    if affected_region.is_empty() {
        return Vec::new();
    }

    // Snapshot only the affected tiles (not the entire map)
    let original_tiles: HashMap<(i32, i32), Option<u32>> = affected_region
        .iter()
        .map(|&(x, y)| {
            let idx = (y as u32 * width + x as u32) as usize;
            ((x, y), tiles.get(idx).copied().flatten())
        })
        .collect();

    // Clone only the portion we need to modify
    // Note: We still need the full slice for the algorithm, but we're limiting the comparison
    let mut preview_tiles = tiles.to_vec();

    // Apply paint to the clone
    paint_terrain_at_target(
        &mut preview_tiles,
        width,
        height,
        target,
        terrain_set,
        terrain_index,
    );

    // Only check tiles in the affected region, not the entire map
    let mut result = Vec::new();
    for (x, y) in affected_region {
        let idx = (y as u32 * width + x as u32) as usize;
        let original = original_tiles.get(&(x, y)).copied().flatten();
        let new_tile = preview_tiles.get(idx).copied().flatten();

        // If tile changed or was added, include in preview
        if new_tile != original {
            if let Some(tile_id) = new_tile {
                result.push(((x, y), tile_id));
            }
        }
    }

    result
}
