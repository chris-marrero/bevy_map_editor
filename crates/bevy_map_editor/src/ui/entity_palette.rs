//! Entity palette for placing schema-defined placeable types

use bevy_egui::egui;

use crate::project::Project;
use crate::EditorState;

/// State for entity placement
#[derive(Default)]
pub struct EntityPaintState {
    /// Currently selected entity type name
    pub selected_entity_type: Option<String>,
    /// Whether entity placement mode is active
    pub is_entity_mode: bool,
}

impl EntityPaintState {
    pub fn new() -> Self {
        Self {
            selected_entity_type: None,
            is_entity_mode: false,
        }
    }
}

/// Render the entity palette showing placeable types from the schema
pub fn render_entity_palette(ui: &mut egui::Ui, editor_state: &mut EditorState, project: &Project) {
    // Get all placeable types from schema
    let placeable_types = project.schema.placeable_type_names();

    if placeable_types.is_empty() {
        ui.label("No placeable entity types defined.");
        ui.separator();
        ui.label("To create placeable entities:");
        ui.label("1. Open Edit → Schema Editor");
        ui.label("2. Create a new data type");
        ui.label("3. Check 'Placeable' checkbox");
        ui.label("4. Optionally set an icon image");
        return;
    }

    ui.label("Click to select an entity type, then place on canvas with the Entity tool.");
    ui.separator();

    // List all placeable types
    for type_name in placeable_types {
        if let Some(type_def) = project.schema.get_type(type_name) {
            let selected = editor_state.selected_entity_type.as_deref() == Some(type_name);

            ui.horizontal(|ui| {
                // Color swatch from type's color field
                let color = parse_hex_color(&type_def.color);
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, color);

                // Icon indicator if type has one
                if type_def.icon.is_some() {
                    ui.label("img");
                }

                // Selectable label with type name
                if ui.selectable_label(selected, type_name).clicked() {
                    editor_state.selected_entity_type = Some(type_name.to_string());
                    // Don't automatically switch tools - let users manually select Entity tool
                }
            });
        }
    }

    ui.separator();

    // Show currently selected type info
    if let Some(type_name) = &editor_state.selected_entity_type {
        if let Some(type_def) = project.schema.get_type(type_name) {
            ui.heading("Selected Type");
            ui.label(format!("Name: {}", type_name));

            if let Some(icon) = &type_def.icon {
                ui.label(format!("Icon: {}", icon));
            }

            ui.label(format!("Properties: {}", type_def.properties.len()));

            // List property names
            if !type_def.properties.is_empty() {
                ui.collapsing("Properties", |ui| {
                    for prop in &type_def.properties {
                        let required = if prop.required { "*" } else { "" };
                        ui.label(format!("• {}{}: {:?}", prop.name, required, prop.prop_type));
                    }
                });
            }
        }
    }
}

/// Parse a hex color string like "#FF0000" or "FF0000" into egui::Color32
fn parse_hex_color(color_str: &str) -> egui::Color32 {
    let hex = color_str.trim_start_matches('#');

    if hex.len() >= 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return egui::Color32::from_rgb(r, g, b);
        }
    }

    // Default fallback color (green)
    egui::Color32::from_rgb(0, 200, 100)
}
