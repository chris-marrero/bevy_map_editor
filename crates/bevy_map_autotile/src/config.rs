//! Autotile configuration and legacy support
//!
//! This module contains configuration types and legacy 47-tile blob support.

use crate::terrain::TerrainSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Configuration for autotiling in a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutotileConfig {
    /// All terrain sets defined in the project
    pub terrain_sets: Vec<TerrainSet>,
    /// Legacy terrain types (for backward compatibility - will be migrated)
    #[serde(default)]
    pub terrains: Vec<LegacyTerrainType>,
}

impl AutotileConfig {
    pub fn new() -> Self {
        Self {
            terrain_sets: Vec::new(),
            terrains: Vec::new(),
        }
    }

    /// Add a terrain set
    pub fn add_terrain_set(&mut self, terrain_set: TerrainSet) {
        self.terrain_sets.push(terrain_set);
    }

    /// Get terrain set by ID
    pub fn get_terrain_set(&self, id: Uuid) -> Option<&TerrainSet> {
        self.terrain_sets.iter().find(|ts| ts.id == id)
    }

    /// Get mutable terrain set by ID
    pub fn get_terrain_set_mut(&mut self, id: Uuid) -> Option<&mut TerrainSet> {
        self.terrain_sets.iter_mut().find(|ts| ts.id == id)
    }

    /// Remove terrain set by ID
    pub fn remove_terrain_set(&mut self, id: Uuid) -> Option<TerrainSet> {
        if let Some(pos) = self.terrain_sets.iter().position(|ts| ts.id == id) {
            Some(self.terrain_sets.remove(pos))
        } else {
            None
        }
    }

    /// Get all terrain sets for a specific tileset
    pub fn get_terrain_sets_for_tileset(&self, tileset_id: Uuid) -> Vec<&TerrainSet> {
        self.terrain_sets
            .iter()
            .filter(|ts| ts.tileset_id == tileset_id)
            .collect()
    }

    // Legacy compatibility methods

    /// Add a legacy terrain type (for backward compatibility)
    pub fn add_terrain(&mut self, terrain: LegacyTerrainType) {
        self.terrains.push(terrain);
    }

    /// Get legacy terrain by ID
    pub fn get_terrain(&self, id: Uuid) -> Option<&LegacyTerrainType> {
        self.terrains.iter().find(|t| t.id == id)
    }

    /// Remove legacy terrain by ID
    pub fn remove_terrain(&mut self, id: Uuid) -> Option<LegacyTerrainType> {
        if let Some(pos) = self.terrains.iter().position(|t| t.id == id) {
            Some(self.terrains.remove(pos))
        } else {
            None
        }
    }
}

/// Legacy terrain type for backward compatibility with old 47-tile blob format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyTerrainType {
    pub id: Uuid,
    pub name: String,
    pub base_tile: u32,
    pub tileset_id: Uuid,
    #[serde(default)]
    pub tile_mapping: HashMap<u8, u32>,
}

/// Type alias for backward compatibility
pub type TerrainType = LegacyTerrainType;

impl LegacyTerrainType {
    /// Create a new legacy terrain type with standard 47-tile blob mapping
    pub fn new(name: String, tileset_id: Uuid, first_tile_index: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            base_tile: first_tile_index + 46,
            tileset_id,
            tile_mapping: Self::create_47_tile_mapping(first_tile_index),
        }
    }

    /// Create the standard 47-tile blob mapping (for backward compatibility)
    fn create_47_tile_mapping(first_tile: u32) -> HashMap<u8, u32> {
        let mut mapping = HashMap::new();
        for i in 0..47 {
            mapping.insert(i as u8, first_tile + i);
        }
        mapping
    }

    /// Get the tile index for a given neighbor bitmask
    pub fn get_tile(&self, bitmask: u8) -> u32 {
        self.tile_mapping
            .get(&bitmask)
            .copied()
            .unwrap_or(self.base_tile)
    }
}

/// Terrain brush state for painting with automatic tile selection
#[derive(Debug, Clone, Default)]
pub struct TerrainBrush {
    /// Currently selected terrain set ID
    pub selected_terrain_set: Option<Uuid>,
    /// Currently selected terrain index within the set
    pub selected_terrain_index: Option<usize>,
    /// Whether terrain painting mode is active
    pub active: bool,
}

impl TerrainBrush {
    pub fn new() -> Self {
        Self {
            selected_terrain_set: None,
            selected_terrain_index: None,
            active: false,
        }
    }

    pub fn select(&mut self, terrain_set_id: Uuid, terrain_index: usize) {
        self.selected_terrain_set = Some(terrain_set_id);
        self.selected_terrain_index = Some(terrain_index);
        self.active = true;
    }

    pub fn deselect(&mut self) {
        self.selected_terrain_set = None;
        self.selected_terrain_index = None;
        self.active = false;
    }
}
