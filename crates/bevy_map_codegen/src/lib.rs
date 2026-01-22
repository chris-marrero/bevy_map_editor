//! bevy_map_codegen - Code generation for bevy_map_editor game projects
//!
//! This crate provides automatic Rust code generation from schema definitions,
//! including:
//!
//! - **Project scaffolding** - Create new game projects with proper structure
//! - **Entity structs** - Auto-generate `#[derive(MapEntity)]` structs from schema types
//! - **Behavior stubs** - Generate empty system function signatures per entity type
//! - **Behavior systems** - Pre-built systems for common 2D patterns (movement, combat, AI)
//!
//! # Example
//!
//! ```rust,ignore
//! use bevy_map_codegen::{create_project, generate_all, ProjectConfig, CodegenConfig};
//!
//! // Create a new game project
//! let project_config = ProjectConfig {
//!     name: "my_game".to_string(),
//!     path: PathBuf::from("./my_game"),
//!     bevy_version: "0.15".to_string(),
//! };
//! create_project(&project_config)?;
//!
//! // Generate code from schema
//! let codegen_config = CodegenConfig::new(PathBuf::from("./my_game/src/generated"));
//! generate_all(&schema, &entity_configs, &codegen_config)?;
//! ```

pub mod behaviors;
pub mod entities;
pub mod enums;
pub mod generator;
pub mod scaffold;
pub mod stubs;

pub use generator::{generate_all, CodegenConfig, CodegenResult};
pub use scaffold::{create_project, ProjectConfig};

use thiserror::Error;

/// Errors that can occur during code generation
#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse generated code: {0}")]
    ParseError(String),

    #[error("Failed to format code: {0}")]
    FormatError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Template error: {0}")]
    TemplateError(String),
}

/// Convert a PascalCase name to snake_case
pub fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert a snake_case name to PascalCase
pub fn to_pascal_case(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Format Rust code using prettyplease
pub fn format_code(code: &str) -> Result<String, CodegenError> {
    let syntax_tree = syn::parse_file(code)
        .map_err(|e| CodegenError::ParseError(format!("Failed to parse: {}", e)))?;

    Ok(prettyplease::unparse(&syntax_tree))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("Player"), "player");
        assert_eq!(to_snake_case("PlayerCharacter"), "player_character");
        assert_eq!(to_snake_case("NPCController"), "n_p_c_controller");
        assert_eq!(to_snake_case("MyHTTPHandler"), "my_h_t_t_p_handler");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("player"), "Player");
        assert_eq!(to_pascal_case("player_character"), "PlayerCharacter");
        assert_eq!(to_pascal_case("my_type"), "MyType");
    }
}
