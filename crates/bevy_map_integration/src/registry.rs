use crate::plugin_meta::{PluginInfo, PluginMeta, PropertyDef};

#[cfg(feature = "editor")]
use crate::editor::EditorExtension;

/// Registry of all loaded integration plugins and their contributions.
///
/// When the `editor` feature is enabled, this type derives `bevy::prelude::Resource`
/// so it can be inserted into a Bevy `App`.
#[derive(Debug, Default)]
#[cfg_attr(feature = "editor", derive(bevy::prelude::Resource))]
pub struct IntegrationRegistry {
    plugins: Vec<PluginMeta>,
    #[cfg(feature = "editor")]
    contributions: Vec<EditorExtension>,
}

impl IntegrationRegistry {
    /// Register a plugin, indexing its properties and UI contributions.
    pub fn register_plugin(&mut self, meta: PluginMeta) {
        #[cfg(feature = "editor")]
        {
            for def in meta.editor.contributions.clone() {
                if let Some(ext) = EditorExtension::from_def(def) {
                    self.contributions.push(ext);
                }
            }
        }
        self.plugins.push(meta);
    }

    /// Return all properties that apply to the given entity type name.
    pub fn properties_for_entity(&self, type_name: &str) -> Vec<(&PluginInfo, &PropertyDef)> {
        let mut result = Vec::new();
        for meta in &self.plugins {
            let applies = meta.plugin.applies_to.is_empty()
                || meta
                    .plugin
                    .applies_to
                    .iter()
                    .any(|t| t.eq_ignore_ascii_case(type_name));
            if applies {
                for prop in &meta.properties {
                    result.push((&meta.plugin, prop));
                }
            }
        }
        result
    }

    /// Return the inspector section name for a plugin, if configured.
    pub fn inspector_section(&self, plugin_name: &str) -> Option<&str> {
        self.plugins
            .iter()
            .find(|m| m.plugin.name == plugin_name)
            .and_then(|m| m.editor.inspector_section.as_deref())
    }

    /// Return all file extensions contributed by all plugins.
    pub fn all_file_extensions(&self) -> Vec<String> {
        let mut exts = Vec::new();
        for meta in &self.plugins {
            for ext in &meta.editor.file_extensions {
                if !exts.contains(ext) {
                    exts.push(ext.clone());
                }
            }
        }
        exts
    }

    /// Return all UI contributions (panels, menu items, toolbar buttons, etc.).
    #[cfg(feature = "editor")]
    pub fn ui_contributions(&self) -> &[EditorExtension] {
        &self.contributions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_meta() -> PluginMeta {
        toml::from_str(
            r#"
[plugin]
name = "test_plugin"
applies_to = ["npc", "enemy"]

[[properties]]
name = "health"
prop_type = "int"
min = 0.0

[[properties]]
name = "speed"
prop_type = "float"

[editor]
inspector_section = "Test Section"
file_extensions = ["tst", "test"]
"#,
        )
        .unwrap()
    }

    #[test]
    fn register_and_query_properties() {
        let mut registry = IntegrationRegistry::default();
        registry.register_plugin(sample_meta());

        let props = registry.properties_for_entity("npc");
        assert_eq!(props.len(), 2);
        assert_eq!(props[0].1.name, "health");
        assert_eq!(props[1].1.name, "speed");

        // Non-matching type returns empty
        let props = registry.properties_for_entity("chest");
        assert!(props.is_empty());
    }

    #[test]
    fn wildcard_applies_to() {
        let meta: PluginMeta = toml::from_str(
            r#"
[plugin]
name = "universal"

[[properties]]
name = "tag"
"#,
        )
        .unwrap();

        let mut registry = IntegrationRegistry::default();
        registry.register_plugin(meta);

        // Empty applies_to matches everything
        let props = registry.properties_for_entity("anything");
        assert_eq!(props.len(), 1);
    }

    #[test]
    fn inspector_section_lookup() {
        let mut registry = IntegrationRegistry::default();
        registry.register_plugin(sample_meta());

        assert_eq!(
            registry.inspector_section("test_plugin"),
            Some("Test Section")
        );
        assert_eq!(registry.inspector_section("unknown"), None);
    }

    #[test]
    fn file_extensions() {
        let mut registry = IntegrationRegistry::default();
        registry.register_plugin(sample_meta());

        let exts = registry.all_file_extensions();
        assert_eq!(exts, vec!["tst", "test"]);
    }

    #[test]
    fn no_duplicate_extensions() {
        let mut registry = IntegrationRegistry::default();
        registry.register_plugin(sample_meta());
        // Register a second plugin with overlapping extension
        let meta2: PluginMeta = toml::from_str(
            r#"
[plugin]
name = "other"

[editor]
file_extensions = ["tst", "other"]
"#,
        )
        .unwrap();
        registry.register_plugin(meta2);

        let exts = registry.all_file_extensions();
        assert_eq!(exts, vec!["tst", "test", "other"]);
    }
}
