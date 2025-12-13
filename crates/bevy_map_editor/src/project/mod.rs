//! Project management for the map editor
//!
//! This module handles project file save/load and the Project resource.

mod file;

pub use file::*;

use bevy::prelude::Resource;
use bevy_map_animation::SpriteData;
use bevy_map_autotile::AutotileConfig;
use bevy_map_core::{Level, Tileset};
use bevy_map_dialogue::DialogueTree;
use bevy_map_schema::Schema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// The entire editor project
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct Project {
    pub version: u32,
    #[serde(skip)]
    pub path: Option<PathBuf>,
    #[serde(skip)]
    pub schema_path: Option<PathBuf>,
    pub schema: Schema,
    pub tilesets: Vec<Tileset>,
    pub data: DataStore,
    pub levels: Vec<Level>,
    /// Autotile terrain configuration
    #[serde(default)]
    pub autotile_config: AutotileConfig,
    /// Sprite sheet assets (reusable sprite/animation definitions)
    #[serde(default, alias = "animations")]
    pub sprite_sheets: Vec<SpriteData>,
    /// Dialogue tree assets
    #[serde(default)]
    pub dialogues: Vec<DialogueTree>,
    #[serde(skip)]
    pub dirty: bool,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: 1,
            path: None,
            schema_path: None,
            schema: Schema::default(),
            tilesets: Vec::new(),
            data: DataStore::default(),
            levels: Vec::new(),
            autotile_config: AutotileConfig::default(),
            sprite_sheets: Vec::new(),
            dialogues: Vec::new(),
            dirty: false,
        }
    }
}

impl Project {
    pub fn new(schema: Schema) -> Self {
        Self {
            version: 1,
            path: None,
            schema_path: None,
            schema,
            tilesets: Vec::new(),
            data: DataStore::default(),
            levels: Vec::new(),
            autotile_config: AutotileConfig::default(),
            sprite_sheets: Vec::new(),
            dialogues: Vec::new(),
            dirty: false,
        }
    }

    /// Mark project as modified
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if project has unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get project name (from path or schema)
    pub fn name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.schema.project.name.clone())
    }

    /// Add a new data instance
    pub fn add_data_instance(&mut self, instance: DataInstance) {
        self.data.add(instance);
        self.dirty = true;
    }

    /// Remove a data instance by ID
    pub fn remove_data_instance(&mut self, id: Uuid) -> Option<DataInstance> {
        let result = self.data.remove(id);
        if result.is_some() {
            self.dirty = true;
        }
        result
    }

    /// Get data instance by ID
    pub fn get_data_instance(&self, id: Uuid) -> Option<&DataInstance> {
        self.data.get(id)
    }

    /// Get mutable data instance by ID
    pub fn get_data_instance_mut(&mut self, id: Uuid) -> Option<&mut DataInstance> {
        let result = self.data.get_mut(id);
        if result.is_some() {
            self.dirty = true;
        }
        result
    }

    /// Count entities of a given type across all levels
    pub fn count_entities_of_type(&self, type_name: &str) -> usize {
        self.levels
            .iter()
            .map(|level| {
                level
                    .entities
                    .iter()
                    .filter(|e| e.type_name == type_name)
                    .count()
            })
            .sum()
    }

    /// Add a new level
    pub fn add_level(&mut self, level: Level) {
        self.levels.push(level);
        self.dirty = true;
    }

    /// Get level by ID
    pub fn get_level(&self, id: Uuid) -> Option<&Level> {
        self.levels.iter().find(|l| l.id == id)
    }

    /// Get mutable level by ID
    pub fn get_level_mut(&mut self, id: Uuid) -> Option<&mut Level> {
        self.dirty = true;
        self.levels.iter_mut().find(|l| l.id == id)
    }

    /// Duplicate a data instance by ID, returns the new instance's ID
    pub fn duplicate_data_instance(&mut self, id: Uuid) -> Option<Uuid> {
        let original = self.data.get(id)?.clone();
        let mut duplicate = original;
        duplicate.id = Uuid::new_v4();

        // Append " (Copy)" to the name if there's a name property
        if let Some(bevy_map_core::Value::String(name)) = duplicate.properties.get_mut("name") {
            name.push_str(" (Copy)");
        }

        let new_id = duplicate.id;
        self.data.add(duplicate);
        self.dirty = true;
        Some(new_id)
    }

    /// Remove a level by ID
    pub fn remove_level(&mut self, id: Uuid) -> Option<Level> {
        if let Some(pos) = self.levels.iter().position(|l| l.id == id) {
            self.dirty = true;
            Some(self.levels.remove(pos))
        } else {
            None
        }
    }

    /// Duplicate a level by ID, returns the new level's ID
    pub fn duplicate_level(&mut self, id: Uuid) -> Option<Uuid> {
        let original = self.get_level(id)?.clone();
        let mut duplicate = original;
        duplicate.id = Uuid::new_v4();
        duplicate.name = format!("{} (Copy)", duplicate.name);

        // Also assign new IDs to all entities
        for entity in &mut duplicate.entities {
            entity.id = Uuid::new_v4();
        }

        let new_id = duplicate.id;
        self.levels.push(duplicate);
        self.dirty = true;
        Some(new_id)
    }

    // Sprite sheet methods

    /// Add a new sprite sheet asset
    pub fn add_sprite_sheet(&mut self, sprite_sheet: SpriteData) {
        self.sprite_sheets.push(sprite_sheet);
        self.dirty = true;
    }

    /// Get a sprite sheet by ID
    pub fn get_sprite_sheet(&self, id: Uuid) -> Option<&SpriteData> {
        self.sprite_sheets.iter().find(|a| a.id == id)
    }

    /// Get mutable sprite sheet by ID
    pub fn get_sprite_sheet_mut(&mut self, id: Uuid) -> Option<&mut SpriteData> {
        self.dirty = true;
        self.sprite_sheets.iter_mut().find(|a| a.id == id)
    }

    /// Remove a sprite sheet by ID
    pub fn remove_sprite_sheet(&mut self, id: Uuid) -> Option<SpriteData> {
        if let Some(pos) = self.sprite_sheets.iter().position(|a| a.id == id) {
            self.dirty = true;
            Some(self.sprite_sheets.remove(pos))
        } else {
            None
        }
    }

    // Dialogue methods

    /// Add a new dialogue tree
    pub fn add_dialogue(&mut self, dialogue: DialogueTree) {
        self.dialogues.push(dialogue);
        self.dirty = true;
    }

    /// Get a dialogue by ID
    pub fn get_dialogue(&self, id: &str) -> Option<&DialogueTree> {
        self.dialogues.iter().find(|d| d.id == id)
    }

    /// Get mutable dialogue by ID
    pub fn get_dialogue_mut(&mut self, id: &str) -> Option<&mut DialogueTree> {
        self.dirty = true;
        self.dialogues.iter_mut().find(|d| d.id == id)
    }

    /// Remove a dialogue by ID
    pub fn remove_dialogue(&mut self, id: &str) -> Option<DialogueTree> {
        if let Some(pos) = self.dialogues.iter().position(|d| d.id == id) {
            self.dirty = true;
            Some(self.dialogues.remove(pos))
        } else {
            None
        }
    }
}

/// A data instance (non-placeable thing like an Item, Quest, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataInstance {
    pub id: Uuid,
    pub type_name: String,
    pub properties: HashMap<String, bevy_map_core::Value>,
}

impl DataInstance {
    pub fn new(type_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            type_name,
            properties: HashMap::new(),
        }
    }
}

/// Stores all data_type instances (non-placeable things like Items, Quests)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataStore {
    /// Key: type name (e.g., "Item", "Quest")
    /// Value: list of instances of that type
    pub instances: HashMap<String, Vec<DataInstance>>,
}

impl DataStore {
    pub fn add(&mut self, instance: DataInstance) {
        self.instances
            .entry(instance.type_name.clone())
            .or_default()
            .push(instance);
    }

    pub fn remove(&mut self, id: Uuid) -> Option<DataInstance> {
        for instances in self.instances.values_mut() {
            if let Some(pos) = instances.iter().position(|i| i.id == id) {
                return Some(instances.remove(pos));
            }
        }
        None
    }

    pub fn get(&self, id: Uuid) -> Option<&DataInstance> {
        for instances in self.instances.values() {
            if let Some(instance) = instances.iter().find(|i| i.id == id) {
                return Some(instance);
            }
        }
        None
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut DataInstance> {
        for instances in self.instances.values_mut() {
            if let Some(instance) = instances.iter_mut().find(|i| i.id == id) {
                return Some(instance);
            }
        }
        None
    }

    pub fn get_by_type(&self, type_name: &str) -> &[DataInstance] {
        self.instances
            .get(type_name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn all_instances(&self) -> impl Iterator<Item = &DataInstance> {
        self.instances.values().flatten()
    }
}
