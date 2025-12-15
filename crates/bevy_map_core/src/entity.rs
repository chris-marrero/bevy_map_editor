//! Entity instance for placed objects in the world

use crate::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// An entity placed in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInstance {
    /// Unique identifier for this instance
    pub id: Uuid,
    /// Type name (e.g., "NPC", "Enemy", "Chest")
    pub type_name: String,
    /// Position in world coordinates [x, y]
    pub position: [f32; 2],
    /// If this is an instance of a template, the template ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<Uuid>,
    /// Property overrides (for template instances) or direct properties
    #[serde(default)]
    pub properties: HashMap<String, Value>,
}

impl EntityInstance {
    /// Create a new entity instance
    pub fn new(type_name: String, position: [f32; 2]) -> Self {
        Self {
            id: Uuid::new_v4(),
            type_name,
            position,
            template_id: None,
            properties: HashMap::new(),
        }
    }

    /// Create an entity instance from a template
    pub fn from_template(template_id: Uuid, type_name: String, position: [f32; 2]) -> Self {
        Self {
            id: Uuid::new_v4(),
            type_name,
            position,
            template_id: Some(template_id),
            properties: HashMap::new(),
        }
    }

    /// Get a display name for this entity
    pub fn get_display_name(&self) -> String {
        self.properties
            .get("name")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{} ({})", self.type_name, &self.id.to_string()[..8]))
    }

    /// Get a string property
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.properties.get(key).and_then(|v| v.as_string())
    }

    /// Set a string property
    pub fn set_string(&mut self, key: &str, value: String) {
        self.properties
            .insert(key.to_string(), Value::String(value));
    }

    /// Get an integer property
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.properties.get(key).and_then(|v| v.as_int())
    }

    /// Set an integer property
    pub fn set_int(&mut self, key: &str, value: i64) {
        self.properties.insert(key.to_string(), Value::Int(value));
    }

    /// Get a float property
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.properties.get(key).and_then(|v| v.as_float())
    }

    /// Set a float property
    pub fn set_float(&mut self, key: &str, value: f64) {
        self.properties.insert(key.to_string(), Value::Float(value));
    }

    /// Get a boolean property
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.properties.get(key).and_then(|v| v.as_bool())
    }

    /// Set a boolean property
    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.properties.insert(key.to_string(), Value::Bool(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_instance() {
        let mut entity = EntityInstance::new("NPC".to_string(), [100.0, 200.0]);
        entity.set_string("name", "Guard".to_string());
        entity.set_int("health", 100);

        assert_eq!(entity.get_string("name"), Some("Guard"));
        assert_eq!(entity.get_int("health"), Some(100));
        assert_eq!(entity.get_display_name(), "Guard");
    }

    #[test]
    fn test_entity_from_template() {
        let template_id = Uuid::new_v4();
        let entity = EntityInstance::from_template(template_id, "Enemy".to_string(), [50.0, 50.0]);

        assert_eq!(entity.template_id, Some(template_id));
        assert_eq!(entity.type_name, "Enemy");
    }
}
