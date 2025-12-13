//! Terrain types and data structures
//!
//! This module contains the core terrain types used for autotiling.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Simple RGBA color for terrain visualization (no Bevy dependency)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

/// Type of terrain set - determines how tiles are matched
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TerrainSetType {
    /// 4 corners per tile (TL, TR, BL, BR)
    /// Good for basic terrain transitions
    #[default]
    Corner,
    /// 4 edges per tile (Top, Right, Bottom, Left)
    /// Good for roads, platforms, paths
    Edge,
    /// 4 corners + 4 edges per tile
    /// Most flexible, requires more tiles
    Mixed,
}

impl TerrainSetType {
    /// Get the number of positions used by this terrain set type
    pub fn position_count(&self) -> usize {
        match self {
            TerrainSetType::Corner => 4,
            TerrainSetType::Edge => 4,
            TerrainSetType::Mixed => 8,
        }
    }

    /// Get the name for a position index
    pub fn position_name(&self, index: usize) -> &'static str {
        match self {
            TerrainSetType::Corner => match index {
                0 => "Top-Left",
                1 => "Top-Right",
                2 => "Bottom-Left",
                3 => "Bottom-Right",
                _ => "Unknown",
            },
            TerrainSetType::Edge => match index {
                0 => "Top",
                1 => "Right",
                2 => "Bottom",
                3 => "Left",
                _ => "Unknown",
            },
            TerrainSetType::Mixed => match index {
                0 => "Top-Left Corner",
                1 => "Top Edge",
                2 => "Top-Right Corner",
                3 => "Right Edge",
                4 => "Bottom-Right Corner",
                5 => "Bottom Edge",
                6 => "Bottom-Left Corner",
                7 => "Left Edge",
                _ => "Unknown",
            },
        }
    }
}

/// A terrain type within a set (e.g., "Grass", "Dirt", "Water")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terrain {
    pub id: Uuid,
    pub name: String,
    /// Display color for UI visualization
    pub color: Color,
    /// Representative tile for this terrain (shown in UI)
    pub icon_tile: Option<u32>,
}

impl Terrain {
    pub fn new(name: String, color: Color) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            color,
            icon_tile: None,
        }
    }
}

/// Terrain assignments for a single tile's corners/edges
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TileTerrainData {
    /// Terrain index at each position (None = no terrain assigned)
    /// For Corner: indices 0-3 (TL, TR, BL, BR)
    /// For Edge: indices 0-3 (Top, Right, Bottom, Left)
    /// For Mixed: indices 0-7 (TL, T, TR, R, BR, B, BL, L - clockwise from top-left)
    pub terrains: [Option<usize>; 8],
}

impl TileTerrainData {
    pub fn new() -> Self {
        Self {
            terrains: [None; 8],
        }
    }

    /// Set terrain at a specific position
    pub fn set(&mut self, position: usize, terrain_index: Option<usize>) {
        if position < 8 {
            self.terrains[position] = terrain_index;
        }
    }

    /// Get terrain at a specific position
    pub fn get(&self, position: usize) -> Option<usize> {
        self.terrains.get(position).copied().flatten()
    }

    /// Check if this tile has any terrain assigned
    pub fn has_any_terrain(&self) -> bool {
        self.terrains.iter().any(|t| t.is_some())
    }

    /// Check if all positions have the same terrain (useful for fill tiles)
    pub fn is_uniform(&self, position_count: usize) -> Option<usize> {
        let first = self.terrains[0]?;
        for i in 1..position_count {
            if self.terrains[i] != Some(first) {
                return None;
            }
        }
        Some(first)
    }
}

/// Constraints for finding a matching tile (Tiled-style with masks)
#[derive(Debug, Clone, Default)]
pub struct TileConstraints {
    /// Desired terrain at each position
    pub desired: [Option<usize>; 8],
    /// Mask indicating which positions are constrained (true = must match)
    pub mask: [bool; 8],
}

impl TileConstraints {
    pub fn new() -> Self {
        Self {
            desired: [None; 8],
            mask: [false; 8],
        }
    }

    /// Set a constrained terrain at a position
    pub fn set(&mut self, position: usize, terrain_index: usize) {
        if position < 8 {
            self.desired[position] = Some(terrain_index);
            self.mask[position] = true;
        }
    }

    /// Set desired terrain without constraining (soft preference)
    pub fn set_desired(&mut self, position: usize, terrain_index: usize) {
        if position < 8 {
            self.desired[position] = Some(terrain_index);
        }
    }

    /// Check if a position is constrained
    pub fn is_constrained(&self, position: usize) -> bool {
        position < 8 && self.mask[position]
    }

    /// Get the required terrains array (legacy compatibility)
    #[allow(dead_code)]
    pub fn required(&self) -> &[Option<usize>; 8] {
        &self.desired
    }
}

/// A terrain set attached to a tileset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainSet {
    pub id: Uuid,
    pub name: String,
    /// Which tileset this terrain set belongs to
    pub tileset_id: Uuid,
    /// Type of terrain matching (Corner, Edge, or Mixed)
    pub set_type: TerrainSetType,
    /// List of terrains in this set (e.g., ["Grass", "Dirt", "Water"])
    pub terrains: Vec<Terrain>,
    /// Terrain assignments for each tile (tile_index -> TileTerrainData)
    pub tile_terrains: HashMap<u32, TileTerrainData>,
}

impl TerrainSet {
    pub fn new(name: String, tileset_id: Uuid, set_type: TerrainSetType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            tileset_id,
            set_type,
            terrains: Vec::new(),
            tile_terrains: HashMap::new(),
        }
    }

    /// Add a new terrain to this set
    pub fn add_terrain(&mut self, name: String, color: Color) -> usize {
        let terrain = Terrain::new(name, color);
        self.terrains.push(terrain);
        self.terrains.len() - 1
    }

    /// Remove a terrain by index
    pub fn remove_terrain(&mut self, index: usize) -> Option<Terrain> {
        if index < self.terrains.len() {
            // Update all tile terrain data to remove references to this terrain
            for tile_data in self.tile_terrains.values_mut() {
                for pos in tile_data.terrains.iter_mut() {
                    if let Some(terrain_idx) = pos {
                        if *terrain_idx == index {
                            *pos = None;
                        } else if *terrain_idx > index {
                            *terrain_idx -= 1;
                        }
                    }
                }
            }
            Some(self.terrains.remove(index))
        } else {
            None
        }
    }

    /// Get terrain index by name
    pub fn get_terrain_index(&self, name: &str) -> Option<usize> {
        self.terrains.iter().position(|t| t.name == name)
    }

    /// Set terrain for a tile position
    pub fn set_tile_terrain(
        &mut self,
        tile_index: u32,
        position: usize,
        terrain_index: Option<usize>,
    ) {
        let data = self.tile_terrains.entry(tile_index).or_default();
        data.set(position, terrain_index);
    }

    /// Get tile terrain data
    pub fn get_tile_terrain(&self, tile_index: u32) -> Option<&TileTerrainData> {
        self.tile_terrains.get(&tile_index)
    }

    /// Find a tile that matches the given constraints (Tiled-style penalty scoring)
    /// Returns the best matching tile, even if not perfect
    pub fn find_matching_tile(&self, constraints: &TileConstraints) -> Option<u32> {
        self.find_best_tile(constraints).map(|(tile, _score)| tile)
    }

    /// Find the best tile match using Tiled-style penalty scoring
    /// Returns (tile_index, penalty_score) where lower score = better match
    /// Returns None only if no tiles have terrain data
    pub fn find_best_tile(&self, constraints: &TileConstraints) -> Option<(u32, i32)> {
        let position_count = self.set_type.position_count();
        let mut best_tile: Option<(u32, i32)> = None;

        for (&tile_index, tile_data) in &self.tile_terrains {
            if !tile_data.has_any_terrain() {
                continue;
            }

            let mut penalty = 0i32;
            let mut impossible = false;

            for i in 0..position_count {
                let desired = constraints.desired[i];
                let actual = tile_data.terrains[i];
                let is_constrained = constraints.mask[i];

                match (desired, actual, is_constrained) {
                    // Constrained position: must match exactly
                    (Some(d), Some(a), true) if d != a => {
                        // Hard constraint violation - reject this tile
                        impossible = true;
                        break;
                    }
                    // Constrained position: matches
                    (Some(_), Some(_), true) => {
                        // Perfect match, no penalty
                    }
                    // Constrained but tile has no terrain here
                    (Some(_), None, true) => {
                        impossible = true;
                        break;
                    }
                    // Unconstrained position with preference: score by match
                    (Some(d), Some(a), false) if d != a => {
                        // Soft mismatch - add transition penalty
                        penalty += self.transition_penalty(d, a);
                    }
                    // Unconstrained, no preference or matches
                    _ => {
                        // No penalty
                    }
                }
            }

            if impossible {
                continue;
            }

            // Track the best (lowest penalty) tile
            match best_tile {
                None => best_tile = Some((tile_index, penalty)),
                Some((_, best_penalty)) if penalty < best_penalty => {
                    best_tile = Some((tile_index, penalty));
                }
                _ => {}
            }
        }

        best_tile
    }

    /// Calculate transition penalty between two terrain types
    /// Returns 0 for same terrain, positive for different terrains
    fn transition_penalty(&self, from: usize, to: usize) -> i32 {
        if from == to {
            0
        } else {
            // Simple penalty: 1 per mismatch
            1
        }
    }

    /// Find all tiles that have a specific terrain (useful for finding "fill" tiles)
    pub fn find_uniform_tiles(&self, terrain_index: usize) -> Vec<u32> {
        let position_count = self.set_type.position_count();

        self.tile_terrains
            .iter()
            .filter_map(|(&tile_index, tile_data)| {
                if tile_data.is_uniform(position_count) == Some(terrain_index) {
                    Some(tile_index)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_set_type_position_count() {
        assert_eq!(TerrainSetType::Corner.position_count(), 4);
        assert_eq!(TerrainSetType::Edge.position_count(), 4);
        assert_eq!(TerrainSetType::Mixed.position_count(), 8);
    }

    #[test]
    fn test_tile_terrain_data() {
        let mut data = TileTerrainData::new();
        assert!(!data.has_any_terrain());

        data.set(0, Some(0));
        assert!(data.has_any_terrain());
        assert_eq!(data.get(0), Some(0));
        assert_eq!(data.get(1), None);
    }

    #[test]
    fn test_terrain_set_find_uniform() {
        let mut set = TerrainSet::new(
            "Test".to_string(),
            Uuid::new_v4(),
            TerrainSetType::Corner,
        );

        set.add_terrain("Grass".to_string(), Color::GREEN);

        // Add a uniform tile (all corners = grass)
        let mut tile_data = TileTerrainData::new();
        for i in 0..4 {
            tile_data.set(i, Some(0));
        }
        set.tile_terrains.insert(42, tile_data);

        let uniform_tiles = set.find_uniform_tiles(0);
        assert!(uniform_tiles.contains(&42));
    }
}
