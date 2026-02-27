//! Project tree view panel

use bevy_egui::egui;
use bevy_map_integration::registry::IntegrationRegistry;
use uuid::Uuid;

use super::Selection;
use crate::project::Project;
use crate::EditorState;
use crate::RenamingItem;

/// Result from rendering the tree view
#[derive(Default)]
pub struct TreeViewResult {
    pub duplicate_data: Option<Uuid>,
    pub delete_data: Option<Uuid>,
    pub duplicate_level: Option<Uuid>,
    pub delete_level: Option<Uuid>,
    pub delete_entity: Option<(Uuid, Uuid)>,
    pub add_tile_layer: Option<Uuid>,
    pub add_object_layer: Option<Uuid>,
    pub delete_layer: Option<(Uuid, usize)>,
    pub move_layer_up: Option<(Uuid, usize)>,
    pub move_layer_down: Option<(Uuid, usize)>,
    pub toggle_layer_visibility: Option<(Uuid, usize)>,
    /// Select entity type for placement (switches to Entity tool)
    pub select_entity_type_for_placement: Option<String>,
    // Sprite sheet actions
    pub create_sprite_sheet: bool,
    /// Edit sprite sheet animations (opens Animation Editor)
    pub edit_sprite_sheet: Option<Uuid>,
    /// Edit sprite sheet grid settings (opens SpriteSheet Editor)
    pub edit_sprite_sheet_settings: Option<Uuid>,
    pub delete_sprite_sheet: Option<Uuid>,
    pub duplicate_sprite_sheet: Option<Uuid>,
    // Dialogue actions
    pub create_dialogue: bool,
    pub edit_dialogue: Option<String>,
    pub delete_dialogue: Option<String>,
    pub duplicate_dialogue: Option<String>,
    // Data instance actions
    /// Select a data instance for editing in inspector
    pub selected_data_instance: Option<Uuid>,
    /// Create a new data instance of the specified type
    pub create_data_instance: Option<String>,
    /// Delete a data instance by UUID
    pub delete_data_instance: Option<Uuid>,
    /// Select an entity (from level) for editing in inspector (level_id, entity_id)
    pub selected_entity: Option<(Uuid, Uuid)>,
    /// Duplicate a data instance
    pub duplicate_data_instance: Option<Uuid>,
    /// Duplicate an entity (level_id, entity_id)
    pub duplicate_entity: Option<(Uuid, Uuid)>,
    /// Toggle data instance selection (Ctrl+Click multi-select)
    pub toggle_data_instance: Option<Uuid>,
    /// Toggle entity selection (Ctrl+Click multi-select)
    pub toggle_entity: Option<(Uuid, Uuid)>,
    /// Delete all selected data instances (bulk delete)
    pub delete_selected_data_instances: bool,
    /// Delete all selected entities (bulk delete)
    pub delete_selected_entities: bool,
    /// Rename a data instance (triggers inline edit mode)
    pub rename_data_instance: Option<Uuid>,
    /// Rename an entity (triggers inline edit mode)
    pub rename_entity: Option<(Uuid, Uuid)>,
    /// Commit the rename with new name
    pub commit_rename: Option<String>,
    /// Cancel the rename operation
    pub cancel_rename: bool,

    // Level rename
    /// Rename a level
    pub rename_level: Option<Uuid>,

    // Layer operations
    /// Rename a layer (level_id, layer_index)
    pub rename_layer: Option<(Uuid, usize)>,
    /// Duplicate a layer (level_id, layer_index)
    pub duplicate_layer: Option<(Uuid, usize)>,

    // Tileset operations
    /// Rename a tileset
    pub rename_tileset: Option<Uuid>,
    /// Duplicate a tileset
    pub duplicate_tileset: Option<Uuid>,
    /// Delete a tileset
    pub delete_tileset: Option<Uuid>,

    // Sprite sheet rename
    /// Rename a sprite sheet
    pub rename_sprite_sheet: Option<Uuid>,

    // Dialogue rename
    /// Rename a dialogue
    pub rename_dialogue: Option<String>,
}

/// Render the project tree view
pub fn render_tree_view(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &mut Project,
    integration_registry: Option<&IntegrationRegistry>,
) -> TreeViewResult {
    let mut result = TreeViewResult::default();

    ui.heading("Tree View");
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("tree_view_scroll")
        .show(ui, |ui| {
            // Levels section
            egui::CollapsingHeader::new("Levels")
                .default_open(true)
                .show(ui, |ui| {
                    render_levels_section(ui, editor_state, project, &mut result, integration_registry);
                });

            // Tilesets section
            egui::CollapsingHeader::new("Tilesets")
                .default_open(true)
                .show(ui, |ui| {
                    let tileset_ids: Vec<_> = project.tilesets.iter().map(|t| (t.id, t.name.clone())).collect();
                    for (tileset_id, tileset_name) in tileset_ids {
                        let selected = matches!(editor_state.selection, Selection::Tileset(id) if id == tileset_id);

                        // Check if this tileset is being renamed
                        let is_renaming = matches!(
                            &editor_state.renaming_item,
                            Some(RenamingItem::Tileset(id)) if *id == tileset_id
                        );

                        if is_renaming {
                            let text_response = ui.text_edit_singleline(&mut editor_state.rename_buffer);
                            if text_response.lost_focus() {
                                if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !editor_state.rename_buffer.is_empty() {
                                    result.commit_rename = Some(editor_state.rename_buffer.clone());
                                }
                                result.cancel_rename = true;
                            }
                            text_response.request_focus();
                        } else {
                            let response = ui.selectable_label(selected, &tileset_name);
                            if response.clicked() {
                                editor_state.selection = Selection::Tileset(tileset_id);
                                editor_state.selected_tileset = Some(tileset_id);
                            }

                            // Context menu for tileset
                            response.context_menu(|ui| {
                                if ui.button("Rename").clicked() {
                                    result.rename_tileset = Some(tileset_id);
                                    ui.close();
                                }
                                if ui.button("Duplicate").clicked() {
                                    result.duplicate_tileset = Some(tileset_id);
                                    ui.close();
                                }
                                ui.separator();
                                if ui.button("Delete").clicked() {
                                    result.delete_tileset = Some(tileset_id);
                                    ui.close();
                                }
                            });
                        }
                    }

                    if ui.button("+ New Tileset").clicked() {
                        editor_state.show_new_tileset_dialog = true;
                    }
                });

            // Data section (type definitions from schema with instances)
            egui::CollapsingHeader::new("Data")
                .default_open(true)
                .show(ui, |ui| {
                    let type_names = project.schema.data_type_names();

                    if type_names.is_empty() {
                        ui.label("(no data types defined)");
                        ui.label("Define types in Edit → Schema Editor");
                    } else {
                        for type_name in type_names {
                            if let Some(type_def) = project.schema.get_type(type_name) {
                                // Parse color for visual indicator
                                let type_color = parse_hex_color(&type_def.color);

                                // Get DataStore instances for this type
                                let data_instances: Vec<_> = project.data.instances
                                    .get(type_name)
                                    .map(|v| v.iter().map(|inst| {
                                        let display_name = inst.properties
                                            .get("name")
                                            .and_then(|v| match v {
                                                bevy_map_core::Value::String(s) => Some(s.clone()),
                                                _ => None,
                                            })
                                            .unwrap_or_else(|| inst.id.to_string()[..8].to_string());
                                        (inst.id, display_name)
                                    }).collect())
                                    .unwrap_or_default();

                                // Get Level entities for this type (from all levels)
                                let level_entities: Vec<_> = project.levels.iter()
                                    .flat_map(|level| {
                                        level.entities.iter()
                                            .filter(|e| e.type_name == *type_name)
                                            .map(|e| {
                                                let display_name = e.properties
                                                    .get("name")
                                                    .and_then(|v| match v {
                                                        bevy_map_core::Value::String(s) => Some(s.clone()),
                                                        _ => None,
                                                    })
                                                    .unwrap_or_else(|| e.id.to_string()[..8].to_string());
                                                // Format: "Name @(x,y) [LevelName]"
                                                let display = format!(
                                                    "{} @({:.0},{:.0}) [{}]",
                                                    display_name,
                                                    e.position[0],
                                                    e.position[1],
                                                    level.name
                                                );
                                                (level.id, e.id, display)
                                            })
                                            .collect::<Vec<_>>()
                                    })
                                    .collect();

                                let total_count = data_instances.len() + level_entities.len();

                                ui.horizontal(|ui| {
                                    // Color swatch
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(12.0, 12.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 2.0, type_color);

                                    // Icon indicator if present
                                    if type_def.icon.is_some() {
                                        ui.label("📄");
                                    }

                                    // Expandable type header with instances nested inside
                                    let header_text = format!("{} ({})", type_name, total_count);
                                    egui::CollapsingHeader::new(&header_text)
                                        .id_salt(format!("data_type_{}", type_name))
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            // Check if type has "name" property for rename functionality
                                            let has_name_property = type_def.properties.iter()
                                                .any(|p| p.name == "name");

                                            // List DataStore instances first
                                            for (inst_id, display_name) in &data_instances {
                                                // Check if this item is being renamed
                                                let is_renaming = matches!(
                                                    &editor_state.renaming_item,
                                                    Some(RenamingItem::DataInstance(id)) if *id == *inst_id
                                                );

                                                // Check if selected (single or multi)
                                                let selected = matches!(
                                                    &editor_state.selection,
                                                    Selection::DataInstance(id) if *id == *inst_id
                                                ) || matches!(
                                                    &editor_state.selection,
                                                    Selection::MultipleDataInstances(ids) if ids.contains(inst_id)
                                                );

                                                if is_renaming {
                                                    // Show inline text edit for rename
                                                    let response = ui.text_edit_singleline(&mut editor_state.rename_buffer);
                                                    if response.lost_focus() {
                                                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                                            result.commit_rename = Some(editor_state.rename_buffer.clone());
                                                        }
                                                        result.cancel_rename = true;
                                                    }
                                                    response.request_focus();
                                                } else {
                                                    let response = ui.selectable_label(selected, display_name);

                                                    if response.clicked() {
                                                        let modifiers = ui.input(|i| i.modifiers);
                                                        if modifiers.ctrl {
                                                            // Ctrl+Click: toggle multi-select
                                                            result.toggle_data_instance = Some(*inst_id);
                                                        } else {
                                                            // Regular click: single select
                                                            result.selected_data_instance = Some(*inst_id);
                                                        }
                                                    }

                                                    // Context menu for instance
                                                    response.context_menu(|ui| {
                                                        if ui.button("Duplicate").clicked() {
                                                            result.duplicate_data_instance = Some(*inst_id);
                                                            ui.close();
                                                        }
                                                        // Rename button (greyed out if no "name" property)
                                                        let rename_btn = ui.add_enabled(has_name_property, egui::Button::new("Rename"));
                                                        if rename_btn.clicked() && has_name_property {
                                                            result.rename_data_instance = Some(*inst_id);
                                                            ui.close();
                                                        } else if !has_name_property {
                                                            rename_btn.on_hover_text("Cannot rename: Type has no 'name' property");
                                                        }
                                                        ui.separator();
                                                        if ui.button("Delete").clicked() {
                                                            result.delete_data_instance = Some(*inst_id);
                                                            ui.close();
                                                        }
                                                        // Show bulk delete option if multiple items selected
                                                        if let Selection::MultipleDataInstances(ids) = &editor_state.selection {
                                                            if ids.contains(inst_id) && ids.len() > 1 {
                                                                ui.separator();
                                                                if ui.button(format!("Delete {} Selected", ids.len())).clicked() {
                                                                    result.delete_selected_data_instances = true;
                                                                    ui.close();
                                                                }
                                                            }
                                                        }
                                                    });
                                                }
                                            }

                                            // List Level entities with position info
                                            for (level_id, entity_id, display) in &level_entities {
                                                // Check if this item is being renamed
                                                let is_renaming = matches!(
                                                    &editor_state.renaming_item,
                                                    Some(RenamingItem::Entity(lid, eid)) if *lid == *level_id && *eid == *entity_id
                                                );

                                                // Check if selected (single or multi)
                                                let selected = matches!(
                                                    &editor_state.selection,
                                                    Selection::Entity(lid, eid) if *lid == *level_id && *eid == *entity_id
                                                ) || matches!(
                                                    &editor_state.selection,
                                                    Selection::MultipleEntities(items) if items.contains(&(*level_id, *entity_id))
                                                );

                                                if is_renaming {
                                                    // Show inline text edit for rename
                                                    let response = ui.text_edit_singleline(&mut editor_state.rename_buffer);
                                                    if response.lost_focus() {
                                                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                                            result.commit_rename = Some(editor_state.rename_buffer.clone());
                                                        }
                                                        result.cancel_rename = true;
                                                    }
                                                    response.request_focus();
                                                } else {
                                                    let response = ui.selectable_label(selected, display);

                                                    if response.clicked() {
                                                        let modifiers = ui.input(|i| i.modifiers);
                                                        if modifiers.ctrl {
                                                            // Ctrl+Click: toggle multi-select
                                                            result.toggle_entity = Some((*level_id, *entity_id));
                                                        } else {
                                                            // Regular click: single select
                                                            result.selected_entity = Some((*level_id, *entity_id));
                                                        }
                                                    }

                                                    // Context menu for level entity
                                                    response.context_menu(|ui| {
                                                        if ui.button("Duplicate").clicked() {
                                                            result.duplicate_entity = Some((*level_id, *entity_id));
                                                            ui.close();
                                                        }
                                                        // Rename button (greyed out if no "name" property)
                                                        let rename_btn = ui.add_enabled(has_name_property, egui::Button::new("Rename"));
                                                        if rename_btn.clicked() && has_name_property {
                                                            result.rename_entity = Some((*level_id, *entity_id));
                                                            ui.close();
                                                        } else if !has_name_property {
                                                            rename_btn.on_hover_text("Cannot rename: Type has no 'name' property");
                                                        }
                                                        ui.separator();
                                                        if ui.button("Delete").clicked() {
                                                            result.delete_entity = Some((*level_id, *entity_id));
                                                            ui.close();
                                                        }
                                                        // Show bulk delete option if multiple items selected
                                                        if let Selection::MultipleEntities(items) = &editor_state.selection {
                                                            if items.contains(&(*level_id, *entity_id)) && items.len() > 1 {
                                                                ui.separator();
                                                                if ui.button(format!("Delete {} Selected", items.len())).clicked() {
                                                                    result.delete_selected_entities = true;
                                                                    ui.close();
                                                                }
                                                            }
                                                        }
                                                        // Integration context menu items
                                                        render_integration_context_menu(ui, bevy_map_integration::editor::ContextMenuTarget::Entity, integration_registry);
                                                    });
                                                }
                                            }

                                            if data_instances.is_empty() && level_entities.is_empty() {
                                                ui.label("(no instances)");
                                            }
                                        });

                                    // "+" button to create new instance
                                    if ui.small_button("+").clicked() {
                                        result.create_data_instance = Some(type_name.to_string());
                                    }

                                    // Placeable indicator
                                    if type_def.placeable {
                                        ui.label("📍");
                                    }
                                });
                            }
                        }
                    }

                    ui.separator();
                    if ui.button("Edit Schema...").clicked() {
                        editor_state.show_schema_editor = true;
                    }
                });

            // Sprite Sheets section
            egui::CollapsingHeader::new("Sprite Sheets")
                .default_open(true)
                .show(ui, |ui| {
                    render_sprite_sheets_section(ui, editor_state, project, &mut result);
                });

            // Dialogues section
            egui::CollapsingHeader::new("Dialogues")
                .default_open(true)
                .show(ui, |ui| {
                    render_dialogues_section(ui, editor_state, project, &mut result);
                });
        });

    result
}

/// Render the levels section with layers nested under each level
fn render_levels_section(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &Project,
    result: &mut TreeViewResult,
    integration_registry: Option<&IntegrationRegistry>,
) {
    // Get placeable types from schema for entity grouping
    let placeable_types: Vec<String> = project
        .schema
        .placeable_type_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Collect level IDs first to avoid borrow issues
    let level_ids: Vec<Uuid> = project.levels.iter().map(|l| l.id).collect();

    for level_id in level_ids {
        let Some(level) = project.levels.iter().find(|l| l.id == level_id) else {
            continue;
        };

        let level_name = level.name.clone();
        let is_selected_level = editor_state.selected_level == Some(level_id);

        // Collect layer info
        let layer_info: Vec<_> = level
            .layers
            .iter()
            .enumerate()
            .map(|(idx, layer)| {
                let is_object_layer =
                    matches!(&layer.data, bevy_map_core::LayerData::Objects { .. });
                let entity_ids: Vec<Uuid> = match &layer.data {
                    bevy_map_core::LayerData::Objects { entities } => entities.clone(),
                    _ => vec![],
                };
                (
                    idx,
                    layer.name.clone(),
                    layer.visible,
                    is_object_layer,
                    entity_ids,
                )
            })
            .collect();

        // Collect entity info for this level
        let level_entities: Vec<_> = level
            .entities
            .iter()
            .map(|e| {
                let display_name = e
                    .properties
                    .get("name")
                    .and_then(|v| match v {
                        bevy_map_core::Value::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "(unnamed)".to_string());
                (e.id, e.type_name.clone(), e.position, display_name)
            })
            .collect();

        // Check if this level is being renamed
        let is_renaming = matches!(
            &editor_state.renaming_item,
            Some(RenamingItem::Level(id)) if *id == level_id
        );

        if is_renaming {
            // Show inline text edit for rename
            ui.horizontal(|ui| {
                ui.label("▶"); // Collapsed indicator
                let text_response = ui.text_edit_singleline(&mut editor_state.rename_buffer);
                if text_response.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !editor_state.rename_buffer.is_empty()
                    {
                        result.commit_rename = Some(editor_state.rename_buffer.clone());
                    }
                    result.cancel_rename = true;
                }
                text_response.request_focus();
            });
        } else {
            // Level header
            let header_response = egui::CollapsingHeader::new(&level_name)
                .id_salt(format!("level_{}", level_id))
                .default_open(is_selected_level)
                .show(ui, |ui| {
                    // Show layers under this level
                    for (layer_idx, layer_name, visible, is_object_layer, entity_ids) in &layer_info
                    {
                        let layer_selected = editor_state.selected_level == Some(level_id)
                            && editor_state.selected_layer == Some(*layer_idx);

                        if *is_object_layer {
                            // Object layer: use CollapsingHeader with nested entities
                            render_object_layer(
                                ui,
                                editor_state,
                                result,
                                level_id,
                                *layer_idx,
                                layer_name,
                                *visible,
                                layer_selected,
                                entity_ids,
                                &level_entities,
                                &placeable_types,
                                project,
                                integration_registry,
                            );
                        } else {
                            // Tile layer: simple horizontal layout
                            render_tile_layer(
                                ui,
                                editor_state,
                                result,
                                level_id,
                                *layer_idx,
                                layer_name,
                                *visible,
                                layer_selected,
                                integration_registry,
                            );
                        }
                    }

                    // Add layer buttons at the bottom of each level
                    ui.horizontal(|ui| {
                        if ui.small_button("+ Tile Layer").clicked() {
                            result.add_tile_layer = Some(level_id);
                        }
                        if ui.small_button("+ Object Layer").clicked() {
                            result.add_object_layer = Some(level_id);
                        }
                    });
                });

            // Header right-click context menu
            header_response.header_response.context_menu(|ui| {
                if ui.button("Select Level").clicked() {
                    editor_state.selection = Selection::Level(level_id);
                    editor_state.selected_level = Some(level_id);
                    ui.close();
                }
                if ui.button("Rename").clicked() {
                    result.rename_level = Some(level_id);
                    ui.close();
                }
                ui.separator();
                if ui.button("Duplicate").clicked() {
                    result.duplicate_level = Some(level_id);
                    ui.close();
                }
                if ui.button("Delete").clicked() {
                    result.delete_level = Some(level_id);
                    ui.close();
                }
                ui.separator();
                if ui.button("Add Tile Layer").clicked() {
                    result.add_tile_layer = Some(level_id);
                    ui.close();
                }
                if ui.button("Add Object Layer").clicked() {
                    result.add_object_layer = Some(level_id);
                    ui.close();
                }
            });
        }
    }

    if ui.button("+ New Level").clicked() {
        editor_state.show_new_level_dialog = true;
    }
}

/// Render a tile layer as a simple horizontal row
fn render_tile_layer(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    result: &mut TreeViewResult,
    level_id: Uuid,
    layer_idx: usize,
    layer_name: &str,
    visible: bool,
    layer_selected: bool,
    integration_registry: Option<&IntegrationRegistry>,
) {
    // Check if this layer is being renamed
    let is_renaming = matches!(
        &editor_state.renaming_item,
        Some(RenamingItem::Layer(lid, idx)) if *lid == level_id && *idx == layer_idx
    );

    ui.horizontal(|ui| {
        // Visibility toggle icon
        let vis_icon = if visible { "👁" } else { "○" };
        if ui.small_button(vis_icon).clicked() {
            result.toggle_layer_visibility = Some((level_id, layer_idx));
        }

        if is_renaming {
            // Show inline text edit for rename
            ui.label("[Tile]");
            let text_response = ui.text_edit_singleline(&mut editor_state.rename_buffer);
            if text_response.lost_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !editor_state.rename_buffer.is_empty()
                {
                    result.commit_rename = Some(editor_state.rename_buffer.clone());
                }
                result.cancel_rename = true;
            }
            text_response.request_focus();
        } else {
            // Layer type indicator and name
            let display_text = format!("[Tile] {}", layer_name);
            let response = ui.selectable_label(layer_selected, display_text);

            if response.clicked() {
                editor_state.selection = Selection::Layer(level_id, layer_idx);
                editor_state.selected_level = Some(level_id);
                editor_state.selected_layer = Some(layer_idx);
            }

            response.context_menu(|ui| {
                if ui.button("Rename").clicked() {
                    result.rename_layer = Some((level_id, layer_idx));
                    ui.close();
                }
                if ui.button("Duplicate").clicked() {
                    result.duplicate_layer = Some((level_id, layer_idx));
                    ui.close();
                }
                ui.separator();
                if ui.button("Move Up").clicked() {
                    result.move_layer_up = Some((level_id, layer_idx));
                    ui.close();
                }
                if ui.button("Move Down").clicked() {
                    result.move_layer_down = Some((level_id, layer_idx));
                    ui.close();
                }
                ui.separator();
                if ui.button("Delete").clicked() {
                    result.delete_layer = Some((level_id, layer_idx));
                    ui.close();
                }
                // Integration context menu items
                render_integration_context_menu(
                    ui,
                    bevy_map_integration::editor::ContextMenuTarget::Layer,
                    integration_registry,
                );
            });
        }
    });
}

/// Render an object layer as a collapsing header with nested entity types
#[allow(clippy::too_many_arguments)]
fn render_object_layer(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    result: &mut TreeViewResult,
    level_id: Uuid,
    layer_idx: usize,
    layer_name: &str,
    visible: bool,
    layer_selected: bool,
    entity_ids: &[Uuid],
    level_entities: &[(Uuid, String, [f32; 2], String)],
    placeable_types: &[String],
    project: &Project,
    integration_registry: Option<&IntegrationRegistry>,
) {
    // Get entities on this layer
    let layer_entities: Vec<_> = level_entities
        .iter()
        .filter(|(id, _, _, _)| entity_ids.contains(id))
        .collect();

    let entity_count = layer_entities.len();
    let header_text = if entity_count > 0 {
        format!("[Object] {} ({})", layer_name, entity_count)
    } else {
        format!("[Object] {}", layer_name)
    };

    // Check if this layer is being renamed
    let is_renaming = matches!(
        &editor_state.renaming_item,
        Some(RenamingItem::Layer(lid, idx)) if *lid == level_id && *idx == layer_idx
    );

    // Visibility button before the header
    ui.horizontal(|ui| {
        let vis_icon = if visible { "👁" } else { "○" };
        if ui.small_button(vis_icon).clicked() {
            result.toggle_layer_visibility = Some((level_id, layer_idx));
        }

        if is_renaming {
            // Show inline text edit for rename
            ui.label("[Object]");
            let text_response = ui.text_edit_singleline(&mut editor_state.rename_buffer);
            if text_response.lost_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !editor_state.rename_buffer.is_empty() {
                    result.commit_rename = Some(editor_state.rename_buffer.clone());
                }
                result.cancel_rename = true;
            }
            text_response.request_focus();
        } else {
            let header = egui::CollapsingHeader::new(&header_text)
                .id_salt(format!("object_layer_{}_{}", level_id, layer_idx))
                .default_open(layer_selected)
                .show(ui, |ui| {
                    // Group entities by type
                    for type_name in placeable_types {
                        let type_entities: Vec<_> = layer_entities.iter()
                            .filter(|(_, tn, _, _)| tn == type_name)
                            .collect();

                        if type_entities.is_empty() {
                            continue;
                        }

                        // Get type color
                        let type_color = project.schema.get_type(type_name)
                            .map(|td| {
                                let hex = td.color.trim_start_matches('#');
                                if hex.len() >= 6 {
                                    if let (Ok(r), Ok(g), Ok(b)) = (
                                        u8::from_str_radix(&hex[0..2], 16),
                                        u8::from_str_radix(&hex[2..4], 16),
                                        u8::from_str_radix(&hex[4..6], 16),
                                    ) {
                                        return egui::Color32::from_rgb(r, g, b);
                                    }
                                }
                                egui::Color32::GRAY
                            })
                            .unwrap_or(egui::Color32::GRAY);

                        let type_header = format!("{} ({})", type_name, type_entities.len());

                        egui::CollapsingHeader::new(&type_header)
                            .id_salt(format!("entity_type_{}_{}_{}", level_id, layer_idx, type_name))
                            .default_open(false)
                            .show(ui, |ui| {
                                // Color swatch and select for placement button
                                ui.horizontal(|ui| {
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(12.0, 12.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 2.0, type_color);

                                    if ui.button("Select").clicked() {
                                        result.select_entity_type_for_placement = Some(type_name.clone());
                                    }
                                });

                                // List instances
                                for (entity_id, _, position, display_name) in type_entities {
                                    let pos_text = format!(
                                        "{} @ ({:.0}, {:.0})",
                                        display_name,
                                        position[0],
                                        position[1]
                                    );

                                    let selected = matches!(
                                        editor_state.selection,
                                        Selection::Entity(lid, eid) if lid == level_id && eid == *entity_id
                                    );

                                    let response = ui.selectable_label(selected, pos_text);

                                    if response.clicked() {
                                        editor_state.selection = Selection::Entity(level_id, *entity_id);
                                        editor_state.selected_level = Some(level_id);
                                        editor_state.selected_layer = Some(layer_idx);
                                    }

                                    response.context_menu(|ui| {
                                        if ui.button("Delete").clicked() {
                                            result.delete_entity = Some((level_id, *entity_id));
                                            ui.close();
                                        }
                                        // Integration context menu items
                                        render_integration_context_menu(ui, bevy_map_integration::editor::ContextMenuTarget::Entity, integration_registry);
                                    });
                                }
                            });
                    }

                    // If no entities, show hint
                    if layer_entities.is_empty() {
                        ui.label("(no entities)");
                    }
                });

            // Layer selection when clicking the header
            if header.header_response.clicked() {
                editor_state.selection = Selection::Layer(level_id, layer_idx);
                editor_state.selected_level = Some(level_id);
                editor_state.selected_layer = Some(layer_idx);
            }

            // Context menu for the object layer header
            header.header_response.context_menu(|ui| {
                if ui.button("Rename").clicked() {
                    result.rename_layer = Some((level_id, layer_idx));
                    ui.close();
                }
                if ui.button("Duplicate").clicked() {
                    result.duplicate_layer = Some((level_id, layer_idx));
                    ui.close();
                }
                ui.separator();
                if ui.button("Move Up").clicked() {
                    result.move_layer_up = Some((level_id, layer_idx));
                    ui.close();
                }
                if ui.button("Move Down").clicked() {
                    result.move_layer_down = Some((level_id, layer_idx));
                    ui.close();
                }
                ui.separator();
                if ui.button("Delete").clicked() {
                    result.delete_layer = Some((level_id, layer_idx));
                    ui.close();
                }
                // Integration context menu items
                render_integration_context_menu(ui, bevy_map_integration::editor::ContextMenuTarget::Layer, integration_registry);
            });
        }
    });
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

/// Render the sprite sheets section in the tree view
fn render_sprite_sheets_section(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &Project,
    result: &mut TreeViewResult,
) {
    if project.sprite_sheets.is_empty() {
        ui.label("(no sprite sheets)");
    } else {
        // Collect sprite sheet data to avoid borrow issues
        let sprite_sheet_data: Vec<_> = project
            .sprite_sheets
            .iter()
            .map(|s| (s.id, s.name.clone()))
            .collect();

        for (sprite_sheet_id, sprite_sheet_name) in sprite_sheet_data {
            let display_name = if sprite_sheet_name.is_empty() {
                format!("Sprite Sheet {}", &sprite_sheet_id.to_string()[..8])
            } else {
                sprite_sheet_name.clone()
            };

            let selected = matches!(
                editor_state.selection,
                Selection::SpriteSheet(id) if id == sprite_sheet_id
            );

            // Check if this sprite sheet is being renamed
            let is_renaming = matches!(
                &editor_state.renaming_item,
                Some(RenamingItem::SpriteSheet(id)) if *id == sprite_sheet_id
            );

            if is_renaming {
                let text_response = ui.text_edit_singleline(&mut editor_state.rename_buffer);
                if text_response.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !editor_state.rename_buffer.is_empty()
                    {
                        result.commit_rename = Some(editor_state.rename_buffer.clone());
                    }
                    result.cancel_rename = true;
                }
                text_response.request_focus();
            } else {
                let response = ui.selectable_label(selected, &display_name);

                if response.clicked() {
                    editor_state.selection = Selection::SpriteSheet(sprite_sheet_id);
                }

                if response.double_clicked() {
                    result.edit_sprite_sheet = Some(sprite_sheet_id);
                }

                response.context_menu(|ui| {
                    if ui.button("Edit Animations...").clicked() {
                        result.edit_sprite_sheet = Some(sprite_sheet_id);
                        ui.close();
                    }
                    if ui.button("Edit Sheet Settings...").clicked() {
                        result.edit_sprite_sheet_settings = Some(sprite_sheet_id);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Rename").clicked() {
                        result.rename_sprite_sheet = Some(sprite_sheet_id);
                        ui.close();
                    }
                    if ui.button("Duplicate").clicked() {
                        result.duplicate_sprite_sheet = Some(sprite_sheet_id);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Delete").clicked() {
                        result.delete_sprite_sheet = Some(sprite_sheet_id);
                        ui.close();
                    }
                });
            }
        }
    }

    if ui.button("+ New Sprite Sheet").clicked() {
        result.create_sprite_sheet = true;
    }
}

/// Render the dialogues section in the tree view
fn render_dialogues_section(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &Project,
    result: &mut TreeViewResult,
) {
    if project.dialogues.is_empty() {
        ui.label("(no dialogues)");
    } else {
        // Collect dialogue data to avoid borrow issues
        let dialogue_data: Vec<_> = project
            .dialogues
            .iter()
            .map(|d| (d.id.clone(), d.name.clone()))
            .collect();

        for (dialogue_id, dialogue_name) in dialogue_data {
            let display_name = if dialogue_name.is_empty() {
                format!("Dialogue {}", &dialogue_id[..8.min(dialogue_id.len())])
            } else {
                dialogue_name.clone()
            };

            let selected = matches!(
                editor_state.selection,
                Selection::Dialogue(ref id) if id == &dialogue_id
            );

            // Check if this dialogue is being renamed
            let is_renaming = matches!(
                &editor_state.renaming_item,
                Some(RenamingItem::Dialogue(ref id)) if id == &dialogue_id
            );

            if is_renaming {
                let text_response = ui.text_edit_singleline(&mut editor_state.rename_buffer);
                if text_response.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !editor_state.rename_buffer.is_empty()
                    {
                        result.commit_rename = Some(editor_state.rename_buffer.clone());
                    }
                    result.cancel_rename = true;
                }
                text_response.request_focus();
            } else {
                let response = ui.selectable_label(selected, &display_name);

                if response.clicked() {
                    editor_state.selection = Selection::Dialogue(dialogue_id.clone());
                }

                if response.double_clicked() {
                    result.edit_dialogue = Some(dialogue_id.clone());
                }

                response.context_menu(|ui| {
                    if ui.button("Edit").clicked() {
                        result.edit_dialogue = Some(dialogue_id.clone());
                        ui.close();
                    }
                    if ui.button("Rename").clicked() {
                        result.rename_dialogue = Some(dialogue_id.clone());
                        ui.close();
                    }
                    if ui.button("Duplicate").clicked() {
                        result.duplicate_dialogue = Some(dialogue_id.clone());
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Delete").clicked() {
                        result.delete_dialogue = Some(dialogue_id.clone());
                        ui.close();
                    }
                });
            }
        }
    }

    if ui.button("+ New Dialogue").clicked() {
        result.create_dialogue = true;
    }
}

/// Render integration context menu items for a given target type.
fn render_integration_context_menu(
    ui: &mut egui::Ui,
    target: bevy_map_integration::editor::ContextMenuTarget,
    integration_registry: Option<&IntegrationRegistry>,
) {
    let Some(registry) = integration_registry else {
        return;
    };

    let items: Vec<_> = registry
        .ui_contributions()
        .iter()
        .filter_map(|ext| {
            if let bevy_map_integration::editor::EditorExtension::ContextMenu {
                target: t,
                name,
                ..
            } = ext
            {
                if *t == target {
                    Some(name.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if !items.is_empty() {
        ui.separator();
        for name in &items {
            if ui.button(name.as_str()).clicked() {
                ui.close();
            }
        }
    }
}
