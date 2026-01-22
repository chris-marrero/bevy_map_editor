//! Project scaffolding for creating new game projects
//!
//! This module generates complete game project structures that integrate with
//! bevy_map_editor's runtime system.

use crate::CodegenError;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for creating a new game project
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    /// Project name (used for Cargo package name)
    pub name: String,
    /// Path where the project will be created
    pub path: PathBuf,
    /// Bevy version to use (e.g., "0.15")
    pub bevy_version: String,
    /// bevy_map_runtime version (e.g., "0.3")
    pub runtime_version: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "my_game".to_string(),
            path: PathBuf::from("./my_game"),
            bevy_version: "0.15".to_string(),
            runtime_version: "0.3".to_string(),
        }
    }
}

impl ProjectConfig {
    /// Create a new project config with the given name and path
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            ..Default::default()
        }
    }

    /// Set the Bevy version
    pub fn with_bevy_version(mut self, version: impl Into<String>) -> Self {
        self.bevy_version = version.into();
        self
    }

    /// Set the runtime version
    pub fn with_runtime_version(mut self, version: impl Into<String>) -> Self {
        self.runtime_version = version.into();
        self
    }
}

/// Create a new game project with the given configuration
///
/// This generates:
/// - `Cargo.toml` with proper dependencies
/// - `src/main.rs` with basic game setup
/// - `src/generated/mod.rs` placeholder
/// - `assets/maps/` directory for map files
pub fn create_project(config: &ProjectConfig) -> Result<PathBuf, CodegenError> {
    let project_path = &config.path;

    // Create directory structure
    fs::create_dir_all(project_path)?;
    fs::create_dir_all(project_path.join("src/generated"))?;
    fs::create_dir_all(project_path.join("assets/maps"))?;
    fs::create_dir_all(project_path.join("assets/sprites"))?;

    // Generate files
    write_cargo_toml(config)?;
    write_main_rs(config)?;
    write_generated_mod(config)?;
    write_gitignore(config)?;

    Ok(project_path.clone())
}

/// Write the Cargo.toml file
fn write_cargo_toml(config: &ProjectConfig) -> Result<(), CodegenError> {
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "{bevy_version}"
bevy_map_runtime = "{runtime_version}"

# Faster compile times for debug builds
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
"#,
        name = config.name,
        bevy_version = config.bevy_version,
        runtime_version = config.runtime_version,
    );

    fs::write(config.path.join("Cargo.toml"), cargo_toml)?;
    Ok(())
}

/// Write the main.rs file
fn write_main_rs(config: &ProjectConfig) -> Result<(), CodegenError> {
    let main_rs = format!(
        r#"//! {} - A game created with bevy_map_editor
//!
//! This is the main entry point for your game.
//! The `generated` module contains auto-generated code from your map project.

use bevy::prelude::*;
use bevy_map_runtime::prelude::*;

mod generated;

fn main() {{
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {{
            primary_window: Some(Window {{
                title: "{}".to_string(),
                ..default()
            }}),
            ..default()
        }}))
        .add_plugins(MapRuntimePlugin)
        .add_plugins(generated::GeneratedPlugin)
        .add_systems(Startup, setup)
        .run();
}}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {{
    // Spawn camera
    commands.spawn(Camera2d);

    // Load the starting map
    // Update this path to match your map file
    commands.spawn(MapHandle(asset_server.load("maps/level.map.json")));
}}
"#,
        config.name,
        config.name.replace('_', " "),
    );

    fs::write(config.path.join("src/main.rs"), main_rs)?;
    Ok(())
}

/// Write the generated/mod.rs placeholder
fn write_generated_mod(config: &ProjectConfig) -> Result<(), CodegenError> {
    let generated_mod = r#"//! Auto-generated code from bevy_map_editor
//!
//! This module is regenerated when you save your map project with code generation enabled.
//! Do not edit manually - your changes will be overwritten!

use bevy::prelude::*;

/// Plugin that registers all generated systems and components
pub struct GeneratedPlugin;

impl Plugin for GeneratedPlugin {
    fn build(&self, _app: &mut App) {
        // Generated entity components, stubs, and behaviors will be registered here
        // Enable code generation in your map project's Game Settings to populate this
    }
}
"#;

    fs::write(config.path.join("src/generated/mod.rs"), generated_mod)?;
    Ok(())
}

/// Write .gitignore file
fn write_gitignore(config: &ProjectConfig) -> Result<(), CodegenError> {
    let gitignore = r#"/target
Cargo.lock
"#;

    fs::write(config.path.join(".gitignore"), gitignore)?;
    Ok(())
}

/// Check if a path is a valid game project (has Cargo.toml and src/main.rs)
pub fn is_valid_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists() && path.join("src/main.rs").exists()
}

/// Check if a path has the generated module
pub fn has_generated_module(path: &Path) -> bool {
    path.join("src/generated/mod.rs").exists()
}

/// Ensure the generated module exists, creating it if necessary
pub fn ensure_generated_module(path: &Path) -> Result<(), CodegenError> {
    let gen_dir = path.join("src/generated");
    if !gen_dir.exists() {
        fs::create_dir_all(&gen_dir)?;
    }

    let mod_file = gen_dir.join("mod.rs");
    if !mod_file.exists() {
        let placeholder = r#"//! Auto-generated code from bevy_map_editor
//!
//! This module will be populated when you save your map project
//! with code generation enabled.

use bevy::prelude::*;

pub struct GeneratedPlugin;

impl Plugin for GeneratedPlugin {
    fn build(&self, _app: &mut App) {
        // Code generation not yet run
    }
}
"#;
        fs::write(mod_file, placeholder)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::default();
        assert_eq!(config.name, "my_game");
        assert_eq!(config.bevy_version, "0.15");
    }

    #[test]
    fn test_project_config_builder() {
        let config = ProjectConfig::new("test_game", "/tmp/test")
            .with_bevy_version("0.14")
            .with_runtime_version("0.2");

        assert_eq!(config.name, "test_game");
        assert_eq!(config.bevy_version, "0.14");
        assert_eq!(config.runtime_version, "0.2");
    }

    #[test]
    fn test_create_project() {
        let temp = temp_dir().join("test_bevy_map_codegen_scaffold");
        let _ = fs::remove_dir_all(&temp); // Clean up any previous test

        let config = ProjectConfig::new("test_game", &temp);
        let result = create_project(&config);

        assert!(result.is_ok());
        assert!(temp.join("Cargo.toml").exists());
        assert!(temp.join("src/main.rs").exists());
        assert!(temp.join("src/generated/mod.rs").exists());
        assert!(temp.join("assets/maps").exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp);
    }
}
