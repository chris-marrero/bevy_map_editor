use std::collections::HashMap;
use std::path::PathBuf;

use crate::plugin_meta::PluginMeta;
use crate::IntegrationError;

/// Discovers and loads plugin TOML metadata files.
#[derive(Debug, Default)]
pub struct PluginManager {
    plugins_dir: Option<PathBuf>,
    plugins: HashMap<String, PluginMeta>,
}

impl PluginManager {
    /// Create a manager using the platform default config directory.
    ///
    /// Searches in order:
    /// 1. `$XDG_CONFIG_HOME/bevy_map_editor/plugins/` (Linux)
    /// 2. Platform-specific config directories
    /// 3. `.bevy_map/plugins/` fallback in the current directory
    pub fn from_default_config() -> Result<Self, IntegrationError> {
        let plugins_dir = Self::find_plugins_dir();
        Ok(PluginManager {
            plugins_dir,
            plugins: HashMap::new(),
        })
    }

    /// Ensure the plugins directory exists.
    pub fn sync_plugins(&mut self) -> Result<(), IntegrationError> {
        if let Some(ref dir) = self.plugins_dir {
            if !dir.exists() {
                std::fs::create_dir_all(dir)?;
            }
        }
        Ok(())
    }

    /// Load all `.toml` plugin metadata files from the plugins directory.
    pub fn load_metadata(&mut self) -> Result<(), IntegrationError> {
        self.plugins.clear();

        let Some(ref dir) = self.plugins_dir else {
            return Ok(());
        };

        if !dir.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                let content = std::fs::read_to_string(&path)?;
                let meta: PluginMeta = toml::from_str(&content)?;
                self.plugins.insert(meta.plugin.name.clone(), meta);
            }
        }

        Ok(())
    }

    /// Iterate over all loaded plugins as `(name, metadata)` pairs.
    pub fn plugins(&self) -> impl Iterator<Item = (&String, &PluginMeta)> {
        self.plugins.iter()
    }

    fn find_plugins_dir() -> Option<PathBuf> {
        // Try XDG_CONFIG_HOME first (Linux)
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let dir = PathBuf::from(xdg).join("bevy_map_editor/plugins");
            return Some(dir);
        }

        // Try platform-specific config directories
        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return Some(PathBuf::from(home).join(".config/bevy_map_editor/plugins"));
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return Some(
                    PathBuf::from(home).join("Library/Application Support/bevy_map_editor/plugins"),
                );
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return Some(PathBuf::from(appdata).join("bevy_map_editor/plugins"));
            }
        }

        // Fallback to local directory
        Some(PathBuf::from(".bevy_map/plugins"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_manager_has_no_plugins() {
        let manager = PluginManager::default();
        assert_eq!(manager.plugins().count(), 0);
    }

    #[test]
    fn load_from_temp_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugins_dir = tmp.path().join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();

        // Write a test plugin file
        std::fs::write(
            plugins_dir.join("test.toml"),
            r#"
[plugin]
name = "temp_test"
version = "0.1.0"

[[properties]]
name = "score"
prop_type = "int"
"#,
        )
        .unwrap();

        let mut manager = PluginManager {
            plugins_dir: Some(plugins_dir),
            plugins: HashMap::new(),
        };

        manager.load_metadata().unwrap();
        assert_eq!(manager.plugins().count(), 1);

        let (name, meta) = manager.plugins().next().unwrap();
        assert_eq!(name, "temp_test");
        assert_eq!(meta.properties.len(), 1);
    }

    #[test]
    fn sync_creates_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let plugins_dir = tmp.path().join("nonexistent/plugins");

        let mut manager = PluginManager {
            plugins_dir: Some(plugins_dir.clone()),
            plugins: HashMap::new(),
        };

        assert!(!plugins_dir.exists());
        manager.sync_plugins().unwrap();
        assert!(plugins_dir.exists());
    }

    #[test]
    fn load_with_no_dir_is_ok() {
        let mut manager = PluginManager {
            plugins_dir: None,
            plugins: HashMap::new(),
        };
        assert!(manager.load_metadata().is_ok());
    }
}
