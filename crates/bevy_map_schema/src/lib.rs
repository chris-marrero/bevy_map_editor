//! Schema validation for bevy_map_editor
//!
//! This crate provides schema definitions and validation for entity types
//! used in bevy_map_editor. It allows defining data types with properties
//! that can be validated at load time.
//!
//! # Example
//!
//! ```rust,ignore
//! use bevy_map_schema::{Schema, load_schema, validate_instance};
//! use bevy_map_core::EntityInstance;
//!
//! // Load schema from JSON
//! let schema = load_schema("schema.json")?;
//!
//! // Validate an entity instance against the schema
//! let entity = EntityInstance::new("NPC".to_string(), [100.0, 200.0]);
//! schema.validate_entity(&entity)?;
//! ```

mod types;
mod validate;

pub use types::*;
pub use validate::*;

use std::path::Path;
use thiserror::Error;

/// Errors that can occur when loading or validating schemas
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Load a schema from a JSON file
pub fn load_schema(path: &Path) -> Result<Schema, SchemaError> {
    let content = std::fs::read_to_string(path).map_err(|e| SchemaError::IoError(e.to_string()))?;

    parse_schema(&content)
}

/// Parse a schema from a JSON string
pub fn parse_schema(json: &str) -> Result<Schema, SchemaError> {
    let schema: Schema =
        serde_json::from_str(json).map_err(|e| SchemaError::ParseError(e.to_string()))?;

    validate_schema(&schema)?;

    Ok(schema)
}

/// Save a schema to a JSON file
pub fn save_schema(schema: &Schema, path: &Path) -> Result<(), SchemaError> {
    let content =
        serde_json::to_string_pretty(schema).map_err(|e| SchemaError::ParseError(e.to_string()))?;

    std::fs::write(path, content).map_err(|e| SchemaError::IoError(e.to_string()))?;

    Ok(())
}

/// Load a schema from bytes
pub fn load_schema_from_bytes(bytes: &[u8]) -> Result<Schema, SchemaError> {
    let schema: Schema =
        serde_json::from_slice(bytes).map_err(|e| SchemaError::ParseError(e.to_string()))?;

    validate_schema(&schema)?;

    Ok(schema)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_schema() {
        let json = r#"{
            "version": 1,
            "project": { "name": "Test" },
            "enums": {},
            "data_types": {},
            "embedded_types": {}
        }"#;

        let schema = parse_schema(json).unwrap();
        assert_eq!(schema.version, 1);
        assert_eq!(schema.project.name, "Test");
    }

    #[test]
    fn test_parse_schema_with_types() {
        let json = r##"{
            "version": 1,
            "project": { "name": "Test" },
            "enums": {
                "ItemType": ["Weapon", "Armor", "Consumable"]
            },
            "data_types": {
                "Item": {
                    "color": "#4CAF50",
                    "properties": [
                        { "name": "name", "type": "string", "required": true },
                        { "name": "itemType", "type": "enum", "enumType": "ItemType" }
                    ]
                }
            },
            "embedded_types": {}
        }"##;

        let schema = parse_schema(json).unwrap();
        assert!(schema.enums.contains_key("ItemType"));
        assert!(schema.data_types.contains_key("Item"));

        let item_type = schema.get_type("Item").unwrap();
        assert_eq!(item_type.properties.len(), 2);
    }

    #[test]
    fn test_invalid_enum_reference() {
        let json = r#"{
            "version": 1,
            "project": { "name": "Test" },
            "enums": {},
            "data_types": {
                "Item": {
                    "properties": [
                        { "name": "itemType", "type": "enum", "enumType": "NonExistent" }
                    ]
                }
            },
            "embedded_types": {}
        }"#;

        let result = parse_schema(json);
        assert!(result.is_err());
        if let Err(SchemaError::ValidationError(msg)) = result {
            assert!(msg.contains("NonExistent"));
        }
    }
}
