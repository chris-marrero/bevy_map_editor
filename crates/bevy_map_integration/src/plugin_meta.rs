use serde::Deserialize;

/// Top-level plugin metadata, deserialized from a TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginMeta {
    pub plugin: PluginInfo,
    #[serde(default)]
    pub properties: Vec<PropertyDef>,
    #[serde(default)]
    pub editor: EditorMeta,
}

/// Basic information about a plugin.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub applies_to: Vec<String>,
}

/// A single property definition contributed by a plugin.
#[derive(Debug, Clone, Deserialize)]
pub struct PropertyDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default = "default_property_type")]
    pub prop_type: PropertyType,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub extensions: Option<Vec<String>>,
    #[serde(default)]
    pub variants: Option<Vec<String>>,
    #[serde(default)]
    pub default: Option<toml::Value>,
}

fn default_property_type() -> PropertyType {
    PropertyType::String
}

/// The type of a plugin property.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyType {
    String,
    Int,
    Float,
    Bool,
    FilePath,
    Enum,
    Point,
    Color,
}

/// Editor-specific metadata for a plugin.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EditorMeta {
    #[serde(default)]
    pub inspector_section: Option<String>,
    #[serde(default)]
    pub file_extensions: Vec<String>,
    #[serde(default)]
    pub contributions: Vec<ContributionDef>,
}

/// A UI contribution declared in plugin metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContributionDef {
    Panel { name: String },
    MenuItem { path: String },
    InspectorSection { name: String },
    ToolbarButton { name: String },
    ContextMenu { target: String, name: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_plugin() {
        let toml_str = r#"
[plugin]
name = "test_plugin"

[[properties]]
name = "speed"
prop_type = "float"
min = 0.0
max = 100.0
"#;
        let meta: PluginMeta = toml::from_str(toml_str).unwrap();
        assert_eq!(meta.plugin.name, "test_plugin");
        assert_eq!(meta.properties.len(), 1);
        assert_eq!(meta.properties[0].name, "speed");
        assert_eq!(meta.properties[0].prop_type, PropertyType::Float);
        assert_eq!(meta.properties[0].min, Some(0.0));
        assert_eq!(meta.properties[0].max, Some(100.0));
    }

    #[test]
    fn parse_full_plugin() {
        let toml_str = r#"
[plugin]
name = "rpg_plugin"
version = "1.0.0"
description = "RPG entity properties"
applies_to = ["npc", "enemy"]

[[properties]]
name = "health"
description = "Max health points"
required = true
prop_type = "int"
min = 1.0
max = 9999.0
default = 100

[[properties]]
name = "faction"
prop_type = "enum"
variants = ["friendly", "hostile", "neutral"]
default = "neutral"

[[properties]]
name = "sprite_path"
prop_type = "filepath"
extensions = ["png", "jpg"]

[editor]
inspector_section = "RPG Properties"
file_extensions = ["rpg", "npcdef"]

[[editor.contributions]]
type = "panel"
name = "RPG Overview"

[[editor.contributions]]
type = "menu_item"
path = "Plugins/RPG/Configure"

[[editor.contributions]]
type = "toolbar_button"
name = "RPG Tools"

[[editor.contributions]]
type = "context_menu"
target = "entity"
name = "Set RPG Type"
"#;
        let meta: PluginMeta = toml::from_str(toml_str).unwrap();
        assert_eq!(meta.plugin.name, "rpg_plugin");
        assert_eq!(meta.plugin.applies_to, vec!["npc", "enemy"]);
        assert_eq!(meta.properties.len(), 3);
        assert!(meta.properties[0].required);
        assert_eq!(
            meta.properties[1].variants,
            Some(vec![
                "friendly".to_string(),
                "hostile".to_string(),
                "neutral".to_string()
            ])
        );
        assert_eq!(meta.editor.inspector_section, Some("RPG Properties".into()));
        assert_eq!(meta.editor.file_extensions, vec!["rpg", "npcdef"]);
        assert_eq!(meta.editor.contributions.len(), 4);
    }

    #[test]
    fn parse_property_types() {
        for (input, expected) in [
            ("string", PropertyType::String),
            ("int", PropertyType::Int),
            ("float", PropertyType::Float),
            ("bool", PropertyType::Bool),
            ("filepath", PropertyType::FilePath),
            ("enum", PropertyType::Enum),
            ("point", PropertyType::Point),
            ("color", PropertyType::Color),
        ] {
            let toml_str = format!(
                r#"
[plugin]
name = "test"

[[properties]]
name = "prop"
prop_type = "{input}"
"#
            );
            let meta: PluginMeta = toml::from_str(&toml_str).unwrap();
            assert_eq!(meta.properties[0].prop_type, expected, "Failed for {input}");
        }
    }
}
