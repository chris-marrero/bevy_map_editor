//! Schema editor UI - Full data type and enum management
//!
//! Provides a comprehensive editor for:
//! - Creating, editing, and deleting enums
//! - Creating, editing, and deleting data types
//! - Managing properties on data types with all 13 property types

use bevy_egui::egui;
use bevy_map_schema::{PropType, PropertyDef, TypeDef};

/// State for the schema editor
#[derive(Default)]
pub struct SchemaEditorState {
    /// Currently active tab
    pub active_tab: SchemaTab,

    // Enum editing state
    pub selected_enum: Option<String>,
    pub new_enum_name: String,
    pub new_enum_value: String,
    pub editing_enum_name: Option<String>,
    pub enum_rename_buffer: String,

    // Data type editing state
    pub selected_type: Option<String>,
    pub new_type_name: String,
    pub editing_type_name: Option<String>,
    pub type_rename_buffer: String,

    // Property editing state
    pub selected_property_idx: Option<usize>,
    pub show_add_property_dialog: bool,
    pub show_edit_property_dialog: bool,
    pub property_edit_state: PropertyEditState,

    // Color picker state
    pub color_picker_buffer: [f32; 3],
}

/// Active tab in the schema editor
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum SchemaTab {
    #[default]
    Enums,
    DataTypes,
}

/// State for editing a property
#[derive(Clone)]
pub struct PropertyEditState {
    pub name: String,
    pub prop_type: PropType,
    pub required: bool,
    pub min: String,
    pub max: String,
    pub enum_type: Option<String>,
    pub ref_type: Option<String>,
    pub item_type: Option<String>,
    pub embedded_type: Option<String>,
    pub show_if: String,
}

impl Default for PropertyEditState {
    fn default() -> Self {
        Self {
            name: String::new(),
            prop_type: PropType::String,
            required: false,
            min: String::new(),
            max: String::new(),
            enum_type: None,
            ref_type: None,
            item_type: None,
            embedded_type: None,
            show_if: String::new(),
        }
    }
}

impl PropertyEditState {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            prop_type: PropType::String,
            required: false,
            min: String::new(),
            max: String::new(),
            enum_type: None,
            ref_type: None,
            item_type: None,
            embedded_type: None,
            show_if: String::new(),
        }
    }

    pub fn from_property(prop: &PropertyDef) -> Self {
        Self {
            name: prop.name.clone(),
            prop_type: prop.prop_type,
            required: prop.required,
            min: prop.min.map(|v| v.to_string()).unwrap_or_default(),
            max: prop.max.map(|v| v.to_string()).unwrap_or_default(),
            enum_type: prop.enum_type.clone(),
            ref_type: prop.ref_type.clone(),
            item_type: prop.item_type.clone(),
            embedded_type: prop.embedded_type.clone(),
            show_if: prop.show_if.clone().unwrap_or_default(),
        }
    }

    pub fn to_property(&self) -> PropertyDef {
        PropertyDef {
            name: self.name.clone(),
            prop_type: self.prop_type,
            required: self.required,
            default: None,
            min: self.min.parse().ok(),
            max: self.max.parse().ok(),
            show_if: if self.show_if.is_empty() {
                None
            } else {
                Some(self.show_if.clone())
            },
            enum_type: self.enum_type.clone(),
            ref_type: self.ref_type.clone(),
            item_type: self.item_type.clone(),
            embedded_type: self.embedded_type.clone(),
        }
    }
}

/// Render the schema editor window
pub fn render_schema_editor(
    ctx: &egui::Context,
    editor_state: &mut crate::EditorState,
    project: &mut crate::project::Project,
) {
    if !editor_state.show_schema_editor {
        return;
    }

    let mut open = true;
    egui::Window::new("Schema Editor")
        .open(&mut open)
        .collapsible(true)
        .resizable(true)
        .default_size([800.0, 600.0])
        .show(ctx, |ui| {
            // Tab bar
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        editor_state.schema_editor_state.active_tab == SchemaTab::Enums,
                        "Enums",
                    )
                    .clicked()
                {
                    editor_state.schema_editor_state.active_tab = SchemaTab::Enums;
                }
                if ui
                    .selectable_label(
                        editor_state.schema_editor_state.active_tab == SchemaTab::DataTypes,
                        "Data Types",
                    )
                    .clicked()
                {
                    editor_state.schema_editor_state.active_tab = SchemaTab::DataTypes;
                }
            });
            ui.separator();

            match editor_state.schema_editor_state.active_tab {
                SchemaTab::Enums => {
                    render_enums_tab(ui, &mut editor_state.schema_editor_state, project)
                }
                SchemaTab::DataTypes => {
                    render_data_types_tab(ui, &mut editor_state.schema_editor_state, project)
                }
            }
        });

    if !open {
        editor_state.show_schema_editor = false;
    }

    // Property dialogs
    render_add_property_dialog(ctx, editor_state, project);
    render_edit_property_dialog(ctx, editor_state, project);
}

/// Render the Enums tab
fn render_enums_tab(
    ui: &mut egui::Ui,
    state: &mut SchemaEditorState,
    project: &mut crate::project::Project,
) {
    // Left panel - Enum list
    egui::SidePanel::left("enum_list")
        .resizable(true)
        .default_width(200.0)
        .show_inside(ui, |ui| {
            ui.heading("Enums");
            ui.separator();

            // Add new enum
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut state.new_enum_name);
                if ui.button("+").clicked() && !state.new_enum_name.is_empty() {
                    let name = state.new_enum_name.clone();
                    if !project.schema.enums.contains_key(&name) {
                        project.schema.enums.insert(name.clone(), Vec::new());
                        state.selected_enum = Some(name);
                        project.mark_dirty();
                    }
                    state.new_enum_name.clear();
                }
            });
            ui.separator();

            // Enum list
            egui::ScrollArea::vertical()
                .id_salt("enum_list_scroll")
                .show(ui, |ui| {
                    let mut enum_names: Vec<_> = project.schema.enums.keys().cloned().collect();
                    enum_names.sort();

                    let mut to_delete = None;
                    for enum_name in &enum_names {
                        let selected = state.selected_enum.as_ref() == Some(enum_name);
                        ui.horizontal(|ui| {
                            if ui.selectable_label(selected, enum_name).clicked() {
                                state.selected_enum = Some(enum_name.clone());
                            }
                            if ui.small_button("X").clicked() {
                                to_delete = Some(enum_name.clone());
                            }
                        });
                    }

                    if let Some(name) = to_delete {
                        project.schema.enums.remove(&name);
                        if state.selected_enum.as_ref() == Some(&name) {
                            state.selected_enum = None;
                        }
                        project.mark_dirty();
                    }
                });
        });

    // Right panel - Enum values
    egui::CentralPanel::default().show_inside(ui, |ui| {
        if let Some(enum_name) = &state.selected_enum.clone() {
            ui.heading(format!("Enum: {}", enum_name));
            ui.separator();

            // Add new value
            ui.horizontal(|ui| {
                ui.label("New value:");
                ui.text_edit_singleline(&mut state.new_enum_value);
                if ui.button("Add").clicked() && !state.new_enum_value.is_empty() {
                    if let Some(values) = project.schema.enums.get_mut(enum_name) {
                        if !values.contains(&state.new_enum_value) {
                            values.push(state.new_enum_value.clone());
                            project.mark_dirty();
                        }
                    }
                    state.new_enum_value.clear();
                }
            });
            ui.separator();

            // Values list
            if let Some(values) = project.schema.enums.get(enum_name).cloned() {
                let mut to_delete = None;
                let mut to_move_up = None;
                let mut to_move_down = None;

                for (idx, value) in values.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}.", idx + 1));
                        ui.label(value);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("X").clicked() {
                                to_delete = Some(idx);
                            }
                            if idx + 1 < values.len() && ui.small_button("v").clicked() {
                                to_move_down = Some(idx);
                            }
                            if idx > 0 && ui.small_button("^").clicked() {
                                to_move_up = Some(idx);
                            }
                        });
                    });
                }

                // Apply changes
                if let Some(idx) = to_delete {
                    if let Some(values) = project.schema.enums.get_mut(enum_name) {
                        values.remove(idx);
                        project.mark_dirty();
                    }
                }
                if let Some(idx) = to_move_up {
                    if let Some(values) = project.schema.enums.get_mut(enum_name) {
                        if idx > 0 {
                            values.swap(idx, idx - 1);
                            project.mark_dirty();
                        }
                    }
                }
                if let Some(idx) = to_move_down {
                    if let Some(values) = project.schema.enums.get_mut(enum_name) {
                        if idx + 1 < values.len() {
                            values.swap(idx, idx + 1);
                            project.mark_dirty();
                        }
                    }
                }
            }
        } else {
            ui.label("Select an enum to edit its values");
        }
    });
}

/// Render the Data Types tab
fn render_data_types_tab(
    ui: &mut egui::Ui,
    state: &mut SchemaEditorState,
    project: &mut crate::project::Project,
) {
    // Left panel - Type list
    egui::SidePanel::left("type_list")
        .resizable(true)
        .default_width(200.0)
        .show_inside(ui, |ui| {
            ui.heading("Data Types");
            ui.separator();

            // Add new type
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut state.new_type_name);
                if ui.button("+").clicked() && !state.new_type_name.is_empty() {
                    let name = state.new_type_name.clone();
                    if !project.schema.data_types.contains_key(&name) {
                        project
                            .schema
                            .data_types
                            .insert(name.clone(), TypeDef::default());
                        state.selected_type = Some(name);
                        project.mark_dirty();
                    }
                    state.new_type_name.clear();
                }
            });
            ui.separator();

            // Type list
            egui::ScrollArea::vertical()
                .id_salt("type_list_scroll")
                .show(ui, |ui| {
                    let mut type_names: Vec<_> =
                        project.schema.data_types.keys().cloned().collect();
                    type_names.sort();

                    let mut to_delete = None;
                    for type_name in &type_names {
                        let selected = state.selected_type.as_ref() == Some(type_name);

                        // Get color for indicator
                        let color = project
                            .schema
                            .data_types
                            .get(type_name)
                            .map(|t| parse_color(&t.color))
                            .unwrap_or(egui::Color32::GRAY);

                        ui.horizontal(|ui| {
                            // Color indicator
                            let (rect, _) = ui
                                .allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 2.0, color);

                            if ui.selectable_label(selected, type_name).clicked() {
                                state.selected_type = Some(type_name.clone());
                                state.selected_property_idx = None;
                            }
                            if ui.small_button("X").clicked() {
                                to_delete = Some(type_name.clone());
                            }
                        });
                    }

                    if let Some(name) = to_delete {
                        project.schema.data_types.remove(&name);
                        if state.selected_type.as_ref() == Some(&name) {
                            state.selected_type = None;
                        }
                        project.mark_dirty();
                    }
                });
        });

    // Right panel - Type details
    egui::CentralPanel::default().show_inside(ui, |ui| {
        if let Some(type_name) = state.selected_type.clone() {
            render_type_editor(ui, state, project, &type_name);
        } else {
            ui.label("Select a data type to edit");
        }
    });
}

/// Render the editor for a single data type
fn render_type_editor(
    ui: &mut egui::Ui,
    state: &mut SchemaEditorState,
    project: &mut crate::project::Project,
    type_name: &str,
) {
    // First check if type exists
    if !project.schema.data_types.contains_key(type_name) {
        return;
    }

    ui.heading(type_name);
    ui.separator();

    // Read current values for display
    let (current_placeable, current_color, current_icon, current_marker_size) = {
        let type_def = project.schema.data_types.get(type_name).unwrap();
        (
            type_def.placeable,
            type_def.color.clone(),
            type_def.icon.clone(),
            type_def.marker_size,
        )
    };

    // Type settings
    let mut new_placeable = current_placeable;
    let mut new_color = parse_color_rgb(&current_color);
    let mut new_icon = current_icon.clone().unwrap_or_default();
    let mut new_marker_size = current_marker_size.unwrap_or(16) as i32;
    let mut settings_changed = false;

    egui::CollapsingHeader::new("Settings")
        .default_open(true)
        .id_salt(format!("settings_{}", type_name))
        .show(ui, |ui| {
            egui::Grid::new(format!("type_settings_{}", type_name))
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    // Placeable checkbox
                    ui.label("Placeable:");
                    if ui
                        .checkbox(&mut new_placeable, "Can be placed in levels")
                        .changed()
                    {
                        settings_changed = true;
                    }
                    ui.end_row();

                    // Marker size (only shown when placeable)
                    if new_placeable {
                        ui.label("Marker Size:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut new_marker_size)
                                    .range(8..=64)
                                    .suffix(" px"),
                            )
                            .changed()
                        {
                            settings_changed = true;
                        }
                        ui.end_row();
                    }

                    // Color picker
                    ui.label("Color:");
                    if ui.color_edit_button_rgb(&mut new_color).changed() {
                        settings_changed = true;
                    }
                    ui.end_row();

                    // Icon (optional) - file browser
                    ui.label("Icon:");
                    ui.horizontal(|ui| {
                        // Show current icon path or "(none)"
                        let display_text = if new_icon.is_empty() {
                            "(none)".to_string()
                        } else {
                            // Show just filename for cleaner display
                            std::path::Path::new(&new_icon)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| new_icon.clone())
                        };
                        ui.label(&display_text);

                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = open_icon_dialog() {
                                new_icon = path;
                                settings_changed = true;
                            }
                        }

                        if !new_icon.is_empty() && ui.button("Clear").clicked() {
                            new_icon.clear();
                            settings_changed = true;
                        }
                    });
                    ui.end_row();
                });
        });

    // Apply changes after UI rendering
    if settings_changed {
        if let Some(type_def) = project.schema.data_types.get_mut(type_name) {
            type_def.placeable = new_placeable;
            type_def.marker_size = if new_placeable {
                Some(new_marker_size as u32)
            } else {
                None
            };
            type_def.color = format!(
                "#{:02x}{:02x}{:02x}",
                (new_color[0] * 255.0) as u8,
                (new_color[1] * 255.0) as u8,
                (new_color[2] * 255.0) as u8
            );
            type_def.icon = if new_icon.is_empty() {
                None
            } else {
                Some(new_icon)
            };
            project.mark_dirty();
        }
    }

    ui.separator();

    // Properties section
    ui.horizontal(|ui| {
        ui.heading("Properties");
        // Use unique ID for button based on type name to avoid ID conflicts
        if ui.add(egui::Button::new("+ Add Property")).clicked() {
            state.show_add_property_dialog = true;
            state.property_edit_state = PropertyEditState::new();
        }
    });
    ui.separator();

    // Property list
    egui::ScrollArea::vertical()
        .id_salt(format!("property_list_scroll_{}", type_name))
        .show(ui, |ui| {
            let type_def = project.schema.data_types.get(type_name).cloned();
            if let Some(type_def) = type_def {
                let mut to_delete = None;
                let mut to_edit = None;
                let mut to_move_up = None;
                let mut to_move_down = None;

                for (idx, prop) in type_def.properties.iter().enumerate() {
                    let selected = state.selected_property_idx == Some(idx);

                    let frame_response = egui::Frame::new()
                        .fill(if selected {
                            ui.style().visuals.selection.bg_fill
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Property info (clickable area for selection)
                                let info_response = ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.strong(&prop.name);
                                        ui.label(format!("({})", prop.prop_type.display_name()));
                                        if prop.required {
                                            ui.colored_label(egui::Color32::RED, "*");
                                        }
                                    });

                                    // Show type-specific info
                                    let mut details = Vec::new();
                                    if let Some(ref enum_type) = prop.enum_type {
                                        details.push(format!("enum: {}", enum_type));
                                    }
                                    if let Some(ref ref_type) = prop.ref_type {
                                        details.push(format!("ref: {}", ref_type));
                                    }
                                    if let Some(ref item_type) = prop.item_type {
                                        details.push(format!("items: {}", item_type));
                                    }
                                    if let Some(ref embedded_type) = prop.embedded_type {
                                        details.push(format!("embedded: {}", embedded_type));
                                    }
                                    if let Some(min) = prop.min {
                                        details.push(format!("min: {}", min));
                                    }
                                    if let Some(max) = prop.max {
                                        details.push(format!("max: {}", max));
                                    }
                                    if !details.is_empty() {
                                        ui.label(details.join(", "));
                                    }
                                });

                                // Make the info area clickable for selection
                                if info_response
                                    .response
                                    .interact(egui::Sense::click())
                                    .clicked()
                                {
                                    state.selected_property_idx = Some(idx);
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("X").clicked() {
                                            to_delete = Some(idx);
                                        }
                                        if ui.small_button("Edit").clicked() {
                                            to_edit = Some(idx);
                                        }
                                        if idx + 1 < type_def.properties.len()
                                            && ui.small_button("v").clicked()
                                        {
                                            to_move_down = Some(idx);
                                        }
                                        if idx > 0 && ui.small_button("^").clicked() {
                                            to_move_up = Some(idx);
                                        }
                                    },
                                );
                            });
                        });

                    // Allow clicking on the frame background to select (but buttons will take priority)
                    let _ = frame_response;
                }

                // Apply changes
                if let Some(idx) = to_delete {
                    if let Some(type_def) = project.schema.data_types.get_mut(type_name) {
                        type_def.properties.remove(idx);
                        state.selected_property_idx = None;
                        project.mark_dirty();
                    }
                }
                if let Some(idx) = to_edit {
                    state.selected_property_idx = Some(idx);
                    state.property_edit_state =
                        PropertyEditState::from_property(&type_def.properties[idx]);
                    state.show_edit_property_dialog = true;
                }
                if let Some(idx) = to_move_up {
                    if let Some(type_def) = project.schema.data_types.get_mut(type_name) {
                        if idx > 0 {
                            type_def.properties.swap(idx, idx - 1);
                            state.selected_property_idx = Some(idx - 1);
                            project.mark_dirty();
                        }
                    }
                }
                if let Some(idx) = to_move_down {
                    if let Some(type_def) = project.schema.data_types.get_mut(type_name) {
                        if idx + 1 < type_def.properties.len() {
                            type_def.properties.swap(idx, idx + 1);
                            state.selected_property_idx = Some(idx + 1);
                            project.mark_dirty();
                        }
                    }
                }
            }
        });
}

/// Render the Add Property dialog
fn render_add_property_dialog(
    ctx: &egui::Context,
    editor_state: &mut crate::EditorState,
    project: &mut crate::project::Project,
) {
    if !editor_state.schema_editor_state.show_add_property_dialog {
        return;
    }

    let mut close = false;
    let mut add = false;

    egui::Window::new("Add Property")
        .id(egui::Id::new("add_property_dialog"))
        .collapsible(false)
        .resizable(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            render_property_form(
                ui,
                &mut editor_state.schema_editor_state.property_edit_state,
                project,
                "add",
            );

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    close = true;
                }
                let can_add = !editor_state
                    .schema_editor_state
                    .property_edit_state
                    .name
                    .is_empty();
                if ui.add_enabled(can_add, egui::Button::new("Add")).clicked() {
                    add = true;
                    close = true;
                }
            });
        });

    if add {
        if let Some(type_name) = &editor_state.schema_editor_state.selected_type.clone() {
            if let Some(type_def) = project.schema.data_types.get_mut(type_name) {
                let prop = editor_state
                    .schema_editor_state
                    .property_edit_state
                    .to_property();
                type_def.properties.push(prop);
                project.mark_dirty();
            }
        }
    }

    if close {
        editor_state.schema_editor_state.show_add_property_dialog = false;
    }
}

/// Render the Edit Property dialog
fn render_edit_property_dialog(
    ctx: &egui::Context,
    editor_state: &mut crate::EditorState,
    project: &mut crate::project::Project,
) {
    if !editor_state.schema_editor_state.show_edit_property_dialog {
        return;
    }

    let mut close = false;
    let mut save = false;

    egui::Window::new("Edit Property")
        .id(egui::Id::new("edit_property_dialog"))
        .collapsible(false)
        .resizable(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            render_property_form(
                ui,
                &mut editor_state.schema_editor_state.property_edit_state,
                project,
                "edit",
            );

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    close = true;
                }
                let can_save = !editor_state
                    .schema_editor_state
                    .property_edit_state
                    .name
                    .is_empty();
                if ui
                    .add_enabled(can_save, egui::Button::new("Save"))
                    .clicked()
                {
                    save = true;
                    close = true;
                }
            });
        });

    if save {
        if let Some(type_name) = &editor_state.schema_editor_state.selected_type.clone() {
            if let Some(prop_idx) = editor_state.schema_editor_state.selected_property_idx {
                if let Some(type_def) = project.schema.data_types.get_mut(type_name) {
                    if prop_idx < type_def.properties.len() {
                        let prop = editor_state
                            .schema_editor_state
                            .property_edit_state
                            .to_property();
                        type_def.properties[prop_idx] = prop;
                        project.mark_dirty();
                    }
                }
            }
        }
    }

    if close {
        editor_state.schema_editor_state.show_edit_property_dialog = false;
    }
}

/// Render the property editing form
fn render_property_form(
    ui: &mut egui::Ui,
    state: &mut PropertyEditState,
    project: &crate::project::Project,
    id_context: &str,
) {
    egui::Grid::new(format!("property_form_{}", id_context))
        .num_columns(2)
        .spacing([10.0, 4.0])
        .show(ui, |ui| {
            // Name
            ui.label("Name:");
            ui.text_edit_singleline(&mut state.name);
            ui.end_row();

            // Type selector
            ui.label("Type:");
            egui::ComboBox::from_id_salt(format!("prop_type_{}", id_context))
                .selected_text(state.prop_type.display_name())
                .show_ui(ui, |ui| {
                    // Base property types (excluding deprecated Embedded)
                    let types = [
                        PropType::String,
                        PropType::Multiline,
                        PropType::Int,
                        PropType::Float,
                        PropType::Bool,
                        PropType::Enum,
                        PropType::Ref,
                        PropType::Array,
                        PropType::Point,
                        PropType::Color,
                        PropType::Sprite,
                        PropType::Dialogue,
                    ];
                    for t in types {
                        ui.selectable_value(&mut state.prop_type, t, t.display_name());
                    }

                    // Add custom data types dynamically
                    if !project.schema.data_types.is_empty() {
                        ui.separator();
                        ui.label("Custom Types:");
                        let mut type_names: Vec<_> =
                            project.schema.data_types.keys().cloned().collect();
                        type_names.sort();
                        for type_name in &type_names {
                            let is_selected = state.prop_type == PropType::Ref
                                && state.ref_type.as_ref() == Some(type_name);
                            if ui.selectable_label(is_selected, type_name).clicked() {
                                state.prop_type = PropType::Ref;
                                state.ref_type = Some(type_name.clone());
                            }
                        }
                    }
                });
            ui.end_row();

            // Required checkbox
            ui.label("Required:");
            ui.checkbox(&mut state.required, "");
            ui.end_row();

            // Type-specific options
            match state.prop_type {
                PropType::Int | PropType::Float => {
                    ui.label("Min:");
                    ui.text_edit_singleline(&mut state.min);
                    ui.end_row();

                    ui.label("Max:");
                    ui.text_edit_singleline(&mut state.max);
                    ui.end_row();
                }
                PropType::Enum => {
                    ui.label("Enum Type:");
                    let enum_names: Vec<_> = project.schema.enums.keys().cloned().collect();
                    let selected = state.enum_type.clone().unwrap_or_default();
                    egui::ComboBox::from_id_salt(format!("enum_type_selector_{}", id_context))
                        .selected_text(&selected)
                        .show_ui(ui, |ui| {
                            for name in &enum_names {
                                if ui
                                    .selectable_label(state.enum_type.as_ref() == Some(name), name)
                                    .clicked()
                                {
                                    state.enum_type = Some(name.clone());
                                }
                            }
                        });
                    ui.end_row();
                }
                PropType::Ref => {
                    ui.label("Reference Type:");
                    let type_names: Vec<_> = project.schema.data_types.keys().cloned().collect();
                    let selected = state.ref_type.clone().unwrap_or_default();
                    egui::ComboBox::from_id_salt(format!("ref_type_selector_{}", id_context))
                        .selected_text(&selected)
                        .show_ui(ui, |ui| {
                            for name in &type_names {
                                if ui
                                    .selectable_label(state.ref_type.as_ref() == Some(name), name)
                                    .clicked()
                                {
                                    state.ref_type = Some(name.clone());
                                }
                            }
                        });
                    ui.end_row();
                }
                PropType::Array => {
                    ui.label("Item Type:");
                    // For arrays, item type can be a basic type or a custom type
                    let item_types = ["String", "Int", "Float", "Bool"];
                    let selected = state.item_type.clone().unwrap_or_default();
                    egui::ComboBox::from_id_salt(format!("item_type_selector_{}", id_context))
                        .selected_text(&selected)
                        .show_ui(ui, |ui| {
                            for name in item_types {
                                if ui
                                    .selectable_label(
                                        state.item_type.as_ref() == Some(&name.to_string()),
                                        name,
                                    )
                                    .clicked()
                                {
                                    state.item_type = Some(name.to_string());
                                }
                            }
                            // Add custom data types
                            if !project.schema.data_types.is_empty() {
                                ui.separator();
                                ui.label("Custom Types:");
                                let mut type_names: Vec<_> =
                                    project.schema.data_types.keys().cloned().collect();
                                type_names.sort();
                                for type_name in &type_names {
                                    if ui
                                        .selectable_label(
                                            state.item_type.as_ref() == Some(type_name),
                                            type_name,
                                        )
                                        .clicked()
                                    {
                                        state.item_type = Some(type_name.clone());
                                    }
                                }
                            }
                        });
                    ui.end_row();
                }
                _ => {}
            }

            // Show If (conditional visibility)
            ui.label("Show If:");
            ui.text_edit_singleline(&mut state.show_if);
            ui.end_row();
        });

    // Help text for show_if
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("'Show If' format: property_name=value (e.g., type=weapon)")
            .small()
            .weak(),
    );
}

/// Parse a hex color string to egui Color32
fn parse_color(color_str: &str) -> egui::Color32 {
    let color_str = color_str.trim_start_matches('#');
    if color_str.len() == 6 {
        if let Ok(r) = u8::from_str_radix(&color_str[0..2], 16) {
            if let Ok(g) = u8::from_str_radix(&color_str[2..4], 16) {
                if let Ok(b) = u8::from_str_radix(&color_str[4..6], 16) {
                    return egui::Color32::from_rgb(r, g, b);
                }
            }
        }
    }
    egui::Color32::GRAY
}

/// Parse a hex color string to RGB float array
fn parse_color_rgb(color_str: &str) -> [f32; 3] {
    let color = parse_color(color_str);
    [
        color.r() as f32 / 255.0,
        color.g() as f32 / 255.0,
        color.b() as f32 / 255.0,
    ]
}

/// Open a file dialog to select an icon image (native only)
#[cfg(not(target_arch = "wasm32"))]
fn open_icon_dialog() -> Option<String> {
    use rfd::FileDialog;

    FileDialog::new()
        .add_filter("Image Files", &["png", "jpg", "jpeg", "bmp", "gif", "svg"])
        .add_filter("All Files", &["*"])
        .set_title("Select Icon Image")
        .pick_file()
        .map(|p| p.display().to_string())
}
