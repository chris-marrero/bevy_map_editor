//! Schema validation logic

use crate::{Schema, SchemaError};

/// Validate that the schema is internally consistent
pub fn validate_schema(schema: &Schema) -> Result<(), SchemaError> {
    // Check that all enum references point to valid enums
    for (type_name, type_def) in schema.data_types.iter().chain(schema.embedded_types.iter()) {
        for prop in &type_def.properties {
            if let Some(enum_type) = &prop.enum_type {
                if !schema.enums.contains_key(enum_type) {
                    return Err(SchemaError::ValidationError(format!(
                        "Type '{}' property '{}' references unknown enum '{}'",
                        type_name, prop.name, enum_type
                    )));
                }
            }

            if let Some(ref_type) = &prop.ref_type {
                if !schema.data_types.contains_key(ref_type) {
                    return Err(SchemaError::ValidationError(format!(
                        "Type '{}' property '{}' references unknown type '{}'",
                        type_name, prop.name, ref_type
                    )));
                }
            }

            if let Some(embedded_type) = &prop.embedded_type {
                if !schema.embedded_types.contains_key(embedded_type) {
                    return Err(SchemaError::ValidationError(format!(
                        "Type '{}' property '{}' references unknown embedded type '{}'",
                        type_name, prop.name, embedded_type
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Validate an entity instance against the schema
pub fn validate_instance(
    schema: &Schema,
    type_name: &str,
    properties: &std::collections::HashMap<String, serde_json::Value>,
) -> Result<(), SchemaError> {
    let type_def = schema
        .get_type(type_name)
        .ok_or_else(|| SchemaError::ValidationError(format!("Unknown type: {}", type_name)))?;

    // Check required properties are present
    for prop_def in &type_def.properties {
        if prop_def.required && !properties.contains_key(&prop_def.name) {
            return Err(SchemaError::ValidationError(format!(
                "Missing required property '{}' for type '{}'",
                prop_def.name, type_name
            )));
        }
    }

    // Validate property values
    for (prop_name, value) in properties {
        if let Some(prop_def) = type_def.properties.iter().find(|p| &p.name == prop_name) {
            validate_property_value(schema, prop_def, value)?;
        }
    }

    Ok(())
}

/// Validate a single property value against its definition
fn validate_property_value(
    schema: &Schema,
    prop_def: &crate::PropertyDef,
    value: &serde_json::Value,
) -> Result<(), SchemaError> {
    use crate::PropType;

    match prop_def.prop_type {
        PropType::String | PropType::Multiline => {
            if !value.is_string() && !value.is_null() {
                return Err(SchemaError::ValidationError(format!(
                    "Property '{}' must be a string",
                    prop_def.name
                )));
            }
        }
        PropType::Int => {
            if let Some(n) = value.as_i64() {
                if let Some(min) = prop_def.min {
                    if (n as f64) < min {
                        return Err(SchemaError::ValidationError(format!(
                            "Property '{}' must be >= {}",
                            prop_def.name, min
                        )));
                    }
                }
                if let Some(max) = prop_def.max {
                    if (n as f64) > max {
                        return Err(SchemaError::ValidationError(format!(
                            "Property '{}' must be <= {}",
                            prop_def.name, max
                        )));
                    }
                }
            } else if !value.is_null() {
                return Err(SchemaError::ValidationError(format!(
                    "Property '{}' must be an integer",
                    prop_def.name
                )));
            }
        }
        PropType::Float => {
            if let Some(n) = value.as_f64() {
                if let Some(min) = prop_def.min {
                    if n < min {
                        return Err(SchemaError::ValidationError(format!(
                            "Property '{}' must be >= {}",
                            prop_def.name, min
                        )));
                    }
                }
                if let Some(max) = prop_def.max {
                    if n > max {
                        return Err(SchemaError::ValidationError(format!(
                            "Property '{}' must be <= {}",
                            prop_def.name, max
                        )));
                    }
                }
            } else if !value.is_null() {
                return Err(SchemaError::ValidationError(format!(
                    "Property '{}' must be a number",
                    prop_def.name
                )));
            }
        }
        PropType::Bool => {
            if !value.is_boolean() && !value.is_null() {
                return Err(SchemaError::ValidationError(format!(
                    "Property '{}' must be a boolean",
                    prop_def.name
                )));
            }
        }
        PropType::Enum => {
            if let Some(s) = value.as_str() {
                if let Some(enum_type) = &prop_def.enum_type {
                    if let Some(enum_values) = schema.get_enum(enum_type) {
                        if !enum_values.contains(&s.to_string()) {
                            return Err(SchemaError::ValidationError(format!(
                                "Property '{}' must be one of: {:?}",
                                prop_def.name, enum_values
                            )));
                        }
                    }
                }
            } else if !value.is_null() {
                return Err(SchemaError::ValidationError(format!(
                    "Property '{}' must be a string enum value",
                    prop_def.name
                )));
            }
        }
        PropType::Array => {
            if !value.is_array() && !value.is_null() {
                return Err(SchemaError::ValidationError(format!(
                    "Property '{}' must be an array",
                    prop_def.name
                )));
            }
        }
        PropType::Ref => {
            if !value.is_string() && !value.is_null() {
                return Err(SchemaError::ValidationError(format!(
                    "Property '{}' must be a reference string",
                    prop_def.name
                )));
            }
        }
        // Other types (Point, Color, Sprite, Dialogue, Embedded) are more complex
        // and validation is deferred to runtime
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_schema;

    #[test]
    fn test_validate_required_property() {
        let schema = parse_schema(
            r#"{
            "version": 1,
            "project": { "name": "Test" },
            "enums": {},
            "data_types": {
                "Item": {
                    "properties": [
                        { "name": "name", "type": "string", "required": true }
                    ]
                }
            },
            "embedded_types": {}
        }"#,
        )
        .unwrap();

        // Missing required property
        let props = std::collections::HashMap::new();
        let result = validate_instance(&schema, "Item", &props);
        assert!(result.is_err());

        // With required property
        let mut props = std::collections::HashMap::new();
        props.insert("name".to_string(), serde_json::json!("Sword"));
        let result = validate_instance(&schema, "Item", &props);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_min_max() {
        let schema = parse_schema(
            r#"{
            "version": 1,
            "project": { "name": "Test" },
            "enums": {},
            "data_types": {
                "Item": {
                    "properties": [
                        { "name": "value", "type": "int", "min": 0, "max": 100 }
                    ]
                }
            },
            "embedded_types": {}
        }"#,
        )
        .unwrap();

        // Below min
        let mut props = std::collections::HashMap::new();
        props.insert("value".to_string(), serde_json::json!(-1));
        let result = validate_instance(&schema, "Item", &props);
        assert!(result.is_err());

        // Above max
        props.insert("value".to_string(), serde_json::json!(101));
        let result = validate_instance(&schema, "Item", &props);
        assert!(result.is_err());

        // Within range
        props.insert("value".to_string(), serde_json::json!(50));
        let result = validate_instance(&schema, "Item", &props);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_enum() {
        let schema = parse_schema(
            r#"{
            "version": 1,
            "project": { "name": "Test" },
            "enums": {
                "ItemType": ["Weapon", "Armor", "Consumable"]
            },
            "data_types": {
                "Item": {
                    "properties": [
                        { "name": "itemType", "type": "enum", "enumType": "ItemType" }
                    ]
                }
            },
            "embedded_types": {}
        }"#,
        )
        .unwrap();

        // Invalid enum value
        let mut props = std::collections::HashMap::new();
        props.insert("itemType".to_string(), serde_json::json!("Invalid"));
        let result = validate_instance(&schema, "Item", &props);
        assert!(result.is_err());

        // Valid enum value
        props.insert("itemType".to_string(), serde_json::json!("Weapon"));
        let result = validate_instance(&schema, "Item", &props);
        assert!(result.is_ok());
    }
}
