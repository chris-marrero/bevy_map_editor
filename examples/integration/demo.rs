//! Integration API Demo - bevy_aseprite_ultra
//!
//! Demonstrates how a third-party plugin author would use the `bevy_map_integration`
//! API to register plugin metadata and bridge editor properties to components.
//!
//! This example does NOT depend on `bevy_aseprite_ultra` itself — it loads the
//! TOML metadata, registers it in an `IntegrationRegistry`, and shows the
//! property query and component-bridging patterns via logging.
//!
//! Controls:
//! - Space: Re-print registry summary to console
//!
//! Run with: cargo run --example integration_demo -p bevy_map_editor_examples

use bevy::prelude::*;
use bevy_map_integration::prelude::*;
use std::path::PathBuf;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Integration API Demo - bevy_aseprite_ultra".to_string(),
                resolution: (720, 480).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .run();
}

/// Locate the TOML metadata file relative to the project.
fn find_toml_path() -> PathBuf {
    // When run via `cargo run`, CARGO_MANIFEST_DIR points to examples/
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(&manifest_dir).join("integration/aseprite_ultra.toml");
        if path.exists() {
            return path;
        }
    }

    // Fallback: running from workspace root
    if let Ok(cwd) = std::env::current_dir() {
        let path = cwd.join("examples/integration/aseprite_ultra.toml");
        if path.exists() {
            return path;
        }
    }

    // Last resort
    PathBuf::from("examples/integration/aseprite_ultra.toml")
}

/// Load the TOML, build the registry, and demonstrate the API.
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // --- Step 1: Load TOML from disk ---
    let toml_path = find_toml_path();
    info!("Loading plugin metadata from: {}", toml_path.display());

    let toml_str = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", toml_path.display()));

    // --- Step 2: Parse into PluginMeta ---
    let meta: PluginMeta = toml::from_str(&toml_str).expect("Failed to parse aseprite_ultra.toml");

    info!(
        "Parsed plugin: {} v{} — \"{}\"",
        meta.plugin.name, meta.plugin.version, meta.plugin.description
    );
    info!("  Applies to: {:?}", meta.plugin.applies_to);
    info!("  Properties: {}", meta.properties.len());

    // --- Step 3: Register in IntegrationRegistry ---
    let mut registry = IntegrationRegistry::default();
    registry.register_plugin(meta);

    // --- Step 4: Query properties for various entity types ---
    print_registry_summary(&registry);

    // --- Step 5: Demonstrate the component-bridging pattern ---
    demonstrate_component_bridge(&registry);

    // Store the registry as a resource so we can query it on SPACE
    commands.insert_resource(RegistryRes(registry));

    // Spawn on-screen info text
    commands.spawn((
        Text::new(
            "Integration API Demo\n\n\
             Plugin: bevy_aseprite_ultra v0.8.1\n\n\
             This example loads TOML metadata and\n\
             registers it via IntegrationRegistry.\n\n\
             Check the console for property queries\n\
             and component bridge output.\n\n\
             SPACE: Re-print registry summary",
        ),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            top: Val::Px(24.0),
            ..default()
        },
    ));
}

/// Wrapper resource so we can store the registry in Bevy's ECS.
#[derive(Resource)]
struct RegistryRes(IntegrationRegistry);

fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, registry: Res<RegistryRes>) {
    if keyboard.just_pressed(KeyCode::Space) {
        info!("--- SPACE pressed: re-printing registry summary ---");
        print_registry_summary(&registry.0);
    }
}

/// Print a summary of the registry contents to the console.
fn print_registry_summary(registry: &IntegrationRegistry) {
    info!("=== Integration Registry Summary ===");

    // Query entity types that should match
    for entity_type in ["animated_sprite", "sprite", "aseprite_entity"] {
        let props = registry.properties_for_entity(entity_type);
        info!(
            "  Entity type '{}': {} properties",
            entity_type,
            props.len()
        );
        for (plugin_info, prop) in &props {
            let required = if prop.required { " [required]" } else { "" };
            info!(
                "    - {}: {:?}{} (from '{}')",
                prop.name, prop.prop_type, required, plugin_info.name
            );
        }
    }

    // Query entity types that should NOT match
    for entity_type in ["chest", "npc"] {
        let props = registry.properties_for_entity(entity_type);
        info!(
            "  Entity type '{}': {} properties (not in applies_to)",
            entity_type,
            props.len()
        );
    }

    // File extensions
    let exts = registry.all_file_extensions();
    info!("  File extensions: {:?}", exts);

    // Inspector section
    if let Some(section) = registry.inspector_section("bevy_aseprite_ultra") {
        info!("  Inspector section: \"{}\"", section);
    }
}

/// Show what a real bridge crate would do: read property values and map them
/// to bevy_aseprite_ultra components. We simulate this with mock data.
fn demonstrate_component_bridge(registry: &IntegrationRegistry) {
    info!("=== Component Bridge Simulation ===");
    info!("A companion crate would translate editor properties into components:");

    let props = registry.properties_for_entity("animated_sprite");

    // Simulate mock property values as an editor entity might have
    let mock_values: Vec<(&str, &str)> = vec![
        ("aseprite_file", "sprites/hero.aseprite"),
        ("animation_tag", "idle"),
        ("animation_speed", "1.5"),
        ("animation_direction", "ping_pong"),
        ("animation_repeat", "loop"),
        ("repeat_count", "1"),
        ("slice_name", ""),
    ];

    for (_, prop) in &props {
        let value = mock_values
            .iter()
            .find(|(name, _)| *name == prop.name)
            .map(|(_, v)| *v)
            .unwrap_or("<unset>");

        if value.is_empty() {
            continue;
        }

        let component_hint = match prop.name.as_str() {
            "aseprite_file" => format!(
                "AsepriteAnimationBundle {{ aseprite: asset_server.load(\"{value}\"), .. }}"
            ),
            "animation_tag" => format!("Animation {{ tag: \"{value}\".into(), .. }}"),
            "animation_speed" => format!("AnimationControl {{ speed: {value}, .. }}"),
            "animation_direction" => {
                format!("AnimationControl {{ direction: Direction::{value}, .. }}")
            }
            "animation_repeat" => format!("AnimationControl {{ repeat: Repeat::{value}, .. }}"),
            "repeat_count" => format!("AnimationControl {{ repeat_count: {value}, .. }}"),
            "slice_name" => format!("AsepriteSliceBundle {{ name: \"{value}\".into(), .. }}"),
            _ => format!("<unknown mapping for '{}'>", prop.name),
        };

        info!("  {} = \"{}\"  →  {}", prop.name, value, component_hint);
    }

    info!("=== End of bridge simulation ===");
}
