//! Tileset and terrain editor window
//!
//! Implements Tiled-style terrain corner/edge painting where users click
//! directly on tile corners or edges to assign terrain types.

use bevy_egui::egui::{self, Color32, Shape};
use bevy_map_autotile::terrain::Color as TerrainColor;
use bevy_map_autotile::TerrainSetType;

use super::TilesetTextureCache;
use crate::project::Project;
use crate::EditorState;

/// State for the tileset editor
#[derive(Default)]
pub struct TilesetEditorState {
    pub selected_tab: TilesetEditorTab,
    pub selected_image_idx: Option<usize>,
    /// Selected terrain index for painting on tiles
    pub selected_terrain_for_assignment: Option<usize>,
    /// Selected tile for property editing
    pub selected_tile_for_properties: Option<u32>,
}

// ============================================================================
// Zone Detection - determines which corner/edge was clicked
// ============================================================================

/// Detect which zone was clicked based on terrain set type.
/// Returns INTERNAL TileTerrainData index (0-3 for Corner/Edge, 0-7 for Mixed).
/// The conversion to Tiled-compatible WangId happens in `tile_terrain_to_wang_id()`.
fn detect_click_zone(
    local_x: f32,
    local_y: f32,
    tile_size: f32,
    set_type: TerrainSetType,
) -> Option<usize> {
    let nx = local_x / tile_size;
    let ny = local_y / tile_size;

    match set_type {
        TerrainSetType::Corner => {
            // Internal: 0=TL, 1=TR, 2=BL, 3=BR (maps to WangId 7,1,5,3)
            let is_top = ny < 0.5;
            let is_left = nx < 0.5;
            Some(match (is_left, is_top) {
                (true, true) => 0,   // TL
                (false, true) => 1,  // TR
                (true, false) => 2,  // BL
                (false, false) => 3, // BR
            })
        }
        TerrainSetType::Edge => {
            // Internal: 0=Top, 1=Right, 2=Bottom, 3=Left (maps to WangId 0,2,4,6)
            let cx = nx - 0.5;
            let cy = ny - 0.5;
            Some(if cx.abs() > cy.abs() {
                if cx > 0.0 { 1 } else { 3 } // Right or Left
            } else {
                if cy < 0.0 { 0 } else { 2 } // Top or Bottom
            })
        }
        TerrainSetType::Mixed => {
            // Internal: 0=TL, 1=Top, 2=TR, 3=Right, 4=BR, 5=Bottom, 6=BL, 7=Left
            // Maps to WangId: 7, 0, 1, 2, 3, 4, 5, 6
            let zone_x = if nx < 0.333 { 0 } else if nx < 0.667 { 1 } else { 2 };
            let zone_y = if ny < 0.333 { 0 } else if ny < 0.667 { 1 } else { 2 };
            match (zone_x, zone_y) {
                (0, 0) => Some(0), // TL corner
                (1, 0) => Some(1), // Top edge
                (2, 0) => Some(2), // TR corner
                (2, 1) => Some(3), // Right edge
                (2, 2) => Some(4), // BR corner
                (1, 2) => Some(5), // Bottom edge
                (0, 2) => Some(6), // BL corner
                (0, 1) => Some(7), // Left edge
                _ => None,         // Center - no action
            }
        }
    }
}

// ============================================================================
// Overlay Drawing - renders colored shapes at terrain positions
// ============================================================================

/// Draw all terrain overlays for a tile based on its TileTerrainData
fn draw_terrain_overlays(
    painter: &egui::Painter,
    rect: egui::Rect,
    set_type: TerrainSetType,
    terrain_data: &bevy_map_autotile::terrain::TileTerrainData,
    terrain_colors: &[Color32],
) {
    let position_count = set_type.position_count();

    for pos in 0..position_count {
        if let Some(terrain_idx) = terrain_data.get(pos) {
            if let Some(&color) = terrain_colors.get(terrain_idx) {
                // Make color semi-transparent
                let overlay_color = Color32::from_rgba_unmultiplied(
                    color.r(),
                    color.g(),
                    color.b(),
                    180,
                );

                match set_type {
                    TerrainSetType::Corner => {
                        draw_corner_overlay(painter, rect, pos, overlay_color);
                    }
                    TerrainSetType::Edge => {
                        draw_edge_overlay(painter, rect, pos, overlay_color);
                    }
                    TerrainSetType::Mixed => {
                        draw_mixed_overlay(painter, rect, pos, overlay_color);
                    }
                }
            }
        }
    }
}

/// Draw a corner triangle overlay (for Corner type terrain sets)
/// Position: 0=TL, 1=TR, 2=BL, 3=BR
fn draw_corner_overlay(
    painter: &egui::Painter,
    rect: egui::Rect,
    position: usize,
    color: Color32,
) {
    let center = rect.center();
    let tl = rect.left_top();
    let tr = rect.right_top();
    let bl = rect.left_bottom();
    let br = rect.right_bottom();
    let top_mid = egui::pos2(center.x, rect.top());
    let bottom_mid = egui::pos2(center.x, rect.bottom());
    let left_mid = egui::pos2(rect.left(), center.y);
    let right_mid = egui::pos2(rect.right(), center.y);

    let points = match position {
        0 => vec![tl, top_mid, center, left_mid],      // Top-Left
        1 => vec![top_mid, tr, right_mid, center],     // Top-Right
        2 => vec![left_mid, center, bottom_mid, bl],   // Bottom-Left
        3 => vec![center, right_mid, br, bottom_mid],  // Bottom-Right
        _ => return,
    };

    painter.add(Shape::convex_polygon(points, color, egui::Stroke::NONE));
}

/// Draw an edge overlay (for Edge type terrain sets)
/// Position: 0=Top, 1=Right, 2=Bottom, 3=Left
fn draw_edge_overlay(
    painter: &egui::Painter,
    rect: egui::Rect,
    position: usize,
    color: Color32,
) {
    let center = rect.center();
    let tl = rect.left_top();
    let tr = rect.right_top();
    let bl = rect.left_bottom();
    let br = rect.right_bottom();

    let points = match position {
        0 => vec![tl, tr, center],           // Top
        1 => vec![tr, br, center],           // Right
        2 => vec![br, bl, center],           // Bottom
        3 => vec![bl, tl, center],           // Left
        _ => return,
    };

    painter.add(Shape::convex_polygon(points, color, egui::Stroke::NONE));
}

/// Draw a mixed overlay (for Mixed type terrain sets - 3x3 grid)
/// Position: 0=TL, 1=Top, 2=TR, 3=Right, 4=BR, 5=Bottom, 6=BL, 7=Left
fn draw_mixed_overlay(
    painter: &egui::Painter,
    rect: egui::Rect,
    position: usize,
    color: Color32,
) {
    let w = rect.width() / 3.0;
    let h = rect.height() / 3.0;
    let left = rect.left();
    let top = rect.top();

    // Calculate cell position based on 3x3 grid layout
    let cell_rect = match position {
        0 => egui::Rect::from_min_size(egui::pos2(left, top), egui::vec2(w, h)),               // TL
        1 => egui::Rect::from_min_size(egui::pos2(left + w, top), egui::vec2(w, h)),           // Top
        2 => egui::Rect::from_min_size(egui::pos2(left + 2.0 * w, top), egui::vec2(w, h)),     // TR
        3 => egui::Rect::from_min_size(egui::pos2(left + 2.0 * w, top + h), egui::vec2(w, h)), // Right
        4 => egui::Rect::from_min_size(egui::pos2(left + 2.0 * w, top + 2.0 * h), egui::vec2(w, h)), // BR
        5 => egui::Rect::from_min_size(egui::pos2(left + w, top + 2.0 * h), egui::vec2(w, h)), // Bottom
        6 => egui::Rect::from_min_size(egui::pos2(left, top + 2.0 * h), egui::vec2(w, h)),     // BL
        7 => egui::Rect::from_min_size(egui::pos2(left, top + h), egui::vec2(w, h)),           // Left
        _ => return,
    };

    painter.rect_filled(cell_rect, 0.0, color);
}

/// Draw grayed-out center indicator for Mixed terrain sets
/// Shows that the center cell is not clickable (Tiled-compatible behavior)
fn draw_mixed_center_indicator(painter: &egui::Painter, rect: egui::Rect) {
    let w = rect.width() / 3.0;
    let h = rect.height() / 3.0;
    let left = rect.left();
    let top = rect.top();

    // Center cell is at (1, 1) in the 3x3 grid
    let center_rect = egui::Rect::from_min_size(
        egui::pos2(left + w, top + h),
        egui::vec2(w, h),
    );

    // Draw a subtle X pattern to indicate non-clickable
    let gray = Color32::from_rgba_unmultiplied(128, 128, 128, 60);
    let tl = center_rect.left_top();
    let tr = center_rect.right_top();
    let bl = center_rect.left_bottom();
    let br = center_rect.right_bottom();

    painter.line_segment([tl, br], egui::Stroke::new(1.0, gray));
    painter.line_segment([tr, bl], egui::Stroke::new(1.0, gray));
}

#[derive(Default, PartialEq)]
pub enum TilesetEditorTab {
    #[default]
    Images,
    TerrainSets,
    TileProperties,
}

/// Render the tileset editor window
pub fn render_tileset_editor(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
    cache: Option<&TilesetTextureCache>,
) {
    if !editor_state.show_tileset_editor {
        return;
    }

    let mut is_open = editor_state.show_tileset_editor;
    egui::Window::new("Tileset Editor")
        .open(&mut is_open)
        .collapsible(true)
        .resizable(true)
        .default_size([800.0, 600.0])
        .min_size([600.0, 400.0])
        .show(ctx, |ui| {
            // Tileset selector
            let current_tileset_name = editor_state
                .selected_tileset
                .and_then(|id| project.tilesets.iter().find(|t| t.id == id))
                .map(|t| t.name.as_str())
                .unwrap_or("(none)");

            ui.horizontal(|ui| {
                ui.label("Tileset:");
                egui::ComboBox::from_id_salt("tileset_editor_selector")
                    .selected_text(current_tileset_name)
                    .show_ui(ui, |ui| {
                        for tileset in &project.tilesets {
                            ui.selectable_value(
                                &mut editor_state.selected_tileset,
                                Some(tileset.id),
                                &tileset.name,
                            );
                        }
                    });
            });

            ui.separator();

            // Tab bar
            ui.horizontal(|ui| {
                if ui.selectable_label(
                    editor_state.tileset_editor_state.selected_tab == TilesetEditorTab::Images,
                    "Images",
                ).clicked() {
                    editor_state.tileset_editor_state.selected_tab = TilesetEditorTab::Images;
                }
                if ui.selectable_label(
                    editor_state.tileset_editor_state.selected_tab == TilesetEditorTab::TerrainSets,
                    "Terrain Sets",
                ).clicked() {
                    editor_state.tileset_editor_state.selected_tab = TilesetEditorTab::TerrainSets;
                }
                if ui.selectable_label(
                    editor_state.tileset_editor_state.selected_tab == TilesetEditorTab::TileProperties,
                    "Tile Properties",
                ).clicked() {
                    editor_state.tileset_editor_state.selected_tab = TilesetEditorTab::TileProperties;
                }
            });

            ui.separator();

            // Tab content
            match editor_state.tileset_editor_state.selected_tab {
                TilesetEditorTab::Images => {
                    render_images_tab(ui, editor_state, project);
                }
                TilesetEditorTab::TerrainSets => {
                    render_terrain_sets_tab(ui, editor_state, project, cache);
                }
                TilesetEditorTab::TileProperties => {
                    render_tile_properties_tab(ui, editor_state, project, cache);
                }
            }
        });

    // Update the editor state if the window was closed via the title bar button
    editor_state.show_tileset_editor = is_open;
}

fn render_images_tab(ui: &mut egui::Ui, editor_state: &mut EditorState, project: &mut Project) {
    let Some(tileset_id) = editor_state.selected_tileset else {
        ui.label("No tileset selected");
        return;
    };

    let Some(tileset) = project.tilesets.iter_mut().find(|t| t.id == tileset_id) else {
        ui.label("Tileset not found");
        return;
    };

    ui.heading("Images");
    ui.separator();

    // List images
    for (idx, image) in tileset.images.iter().enumerate() {
        ui.horizontal(|ui| {
            let selected = editor_state.tileset_editor_state.selected_image_idx == Some(idx);
            if ui.selectable_label(selected, &image.name).clicked() {
                editor_state.tileset_editor_state.selected_image_idx = Some(idx);
            }
            ui.label(format!("{}x{} tiles", image.columns, image.rows));
        });
    }

    ui.separator();

    if ui.button("Add Image...").clicked() {
        editor_state.show_add_tileset_image_dialog = true;
    }
}

fn render_terrain_sets_tab(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &mut Project,
    cache: Option<&TilesetTextureCache>,
) {
    let Some(tileset_id) = editor_state.selected_tileset else {
        ui.label("No tileset selected");
        return;
    };

    // Clone tileset data to avoid borrow conflicts
    let tileset_data = project.tilesets.iter()
        .find(|t| t.id == tileset_id)
        .map(|t| (t.tile_size, t.images.clone(), !t.images.is_empty()));

    // Split into left panel (terrain list) and right panel (tileset preview)
    // Using columns() for proper space distribution like the animation editor
    ui.columns(2, |columns| {
        // Left column: Terrain set and terrain list
        columns[0].vertical(|ui| {
            ui.heading("Terrain Sets");

            // List terrain sets for this tileset
            let terrain_sets: Vec<_> = project.autotile_config.terrain_sets
                .iter()
                .filter(|ts| ts.tileset_id == tileset_id)
                .map(|ts| (ts.id, ts.name.clone(), ts.set_type, ts.terrains.len()))
                .collect();

            for (ts_id, name, set_type, terrain_count) in &terrain_sets {
                ui.horizontal(|ui| {
                    let selected = editor_state.selected_terrain_set == Some(*ts_id);
                    if ui.selectable_label(selected, name).clicked() {
                        editor_state.selected_terrain_set = Some(*ts_id);
                        editor_state.tileset_editor_state.selected_terrain_for_assignment = None;
                    }
                    ui.small(format!("{:?} ({})", set_type, terrain_count));
                });
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("+ New Set").clicked() {
                    editor_state.show_new_terrain_set_dialog = true;
                }
            });

            // If a terrain set is selected, show its terrains
            if let Some(ts_id) = editor_state.selected_terrain_set {
                if let Some(terrain_set) = project.autotile_config.get_terrain_set(ts_id) {
                    ui.separator();
                    ui.heading("Terrains");

                    // Terrain list with selection
                    for (idx, terrain) in terrain_set.terrains.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let color = terrain_color_to_egui(&terrain.color);
                            let selected = editor_state.tileset_editor_state.selected_terrain_for_assignment == Some(idx);

                            // Color swatch
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(16.0, 16.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, 2.0, color);

                            // Terrain name with selection
                            if ui.selectable_label(selected, &terrain.name).clicked() {
                                editor_state.tileset_editor_state.selected_terrain_for_assignment = Some(idx);
                            }
                        });
                    }

                    // Buttons
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("+ Add Terrain").clicked() {
                            editor_state.show_add_terrain_to_set_dialog = true;
                        }
                        if editor_state.tileset_editor_state.selected_terrain_for_assignment.is_some() {
                            if ui.button("Clear Selection").clicked() {
                                editor_state.tileset_editor_state.selected_terrain_for_assignment = None;
                            }
                        }
                    });

                    // Instructions based on terrain set type
                    ui.separator();
                    ui.small("Click on tile corners/edges to paint terrain.");
                    ui.small("Ctrl+click to clear a position.");
                    ui.add_space(4.0);
                    match terrain_set.set_type {
                        TerrainSetType::Corner => {
                            ui.small("Corner mode: 4 zones per tile (corners)");
                        }
                        TerrainSetType::Edge => {
                            ui.small("Edge mode: 4 zones per tile (edges)");
                        }
                        TerrainSetType::Mixed => {
                            ui.small("Mixed mode: 8 zones per tile");
                        }
                    }
                }
            }
        });

        // Right column: Tileset preview with terrain overlays
        columns[1].vertical(|ui| {
            ui.heading("Tile Assignments");

            if let Some((tile_size, images, has_images)) = tileset_data.clone() {
                if !has_images {
                    ui.label("No images in tileset");
                } else {
                    // Use ScrollArea with available height
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            render_tileset_with_terrain_overlay(
                                ui,
                                editor_state,
                                project,
                                tileset_id,
                                tile_size,
                                &images,
                                cache,
                            );
                        });
                }
            }
        });
    });
}

/// Convert terrain color to egui Color32
fn terrain_color_to_egui(color: &TerrainColor) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8,
        (color.a * 200.0) as u8, // Slightly transparent
    )
}

/// Render the tileset with Tiled-style terrain corner/edge overlays.
/// Users can click directly on tile corners/edges to paint terrain assignments.
fn render_tileset_with_terrain_overlay(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &mut Project,
    _tileset_id: uuid::Uuid,
    _tile_size: u32,
    images: &[bevy_map_core::TilesetImage],
    cache: Option<&TilesetTextureCache>,
) {
    // Larger display size for easier clicking on corners/edges
    let display_size = egui::vec2(48.0, 48.0);
    let mut virtual_offset = 0u32;

    // Get terrain set info for overlays
    let terrain_info = editor_state.selected_terrain_set
        .and_then(|ts_id| project.autotile_config.get_terrain_set(ts_id))
        .map(|ts| {
            let colors: Vec<Color32> = ts.terrains.iter()
                .map(|t| terrain_color_to_egui(&t.color))
                .collect();
            (ts.set_type, colors)
        });

    let (set_type, terrain_colors) = match terrain_info {
        Some((st, tc)) => (Some(st), tc),
        None => (None, Vec::new()),
    };

    for image in images {
        // Get texture for this image
        let texture_id = cache
            .and_then(|c| c.loaded.get(&image.id))
            .map(|(_, tex_id, _, _)| *tex_id);

        ui.collapsing(&image.name, |ui| {
            if image.columns == 0 || image.rows == 0 {
                ui.label("Image not loaded yet");
                return;
            }

            let uv_tile_width = 1.0 / image.columns.max(1) as f32;
            let uv_tile_height = 1.0 / image.rows.max(1) as f32;

            for row in 0..image.rows {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);

                    for col in 0..image.columns {
                        let local_index = row * image.columns + col;
                        let virtual_index = virtual_offset + local_index;

                        // Allocate space and get response for interaction
                        // Use click_and_drag so drag events go to tiles, not ScrollArea
                        let (rect, response) = ui.allocate_exact_size(
                            display_size,
                            egui::Sense::click_and_drag(),
                        );

                        // Draw tile texture
                        if let Some(tex_id) = texture_id {
                            let uv_min = egui::pos2(
                                col as f32 * uv_tile_width,
                                row as f32 * uv_tile_height,
                            );
                            let uv_max = egui::pos2(
                                (col + 1) as f32 * uv_tile_width,
                                (row + 1) as f32 * uv_tile_height,
                            );

                            // Draw texture using mesh
                            let mut mesh = egui::Mesh::with_texture(tex_id);
                            mesh.add_rect_with_uv(
                                rect,
                                egui::Rect::from_min_max(uv_min, uv_max),
                                Color32::WHITE,
                            );
                            ui.painter().add(Shape::mesh(mesh));
                        } else {
                            // Fallback: draw placeholder
                            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(60));
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{}", virtual_index),
                                egui::FontId::default(),
                                Color32::WHITE,
                            );
                        }

                        // Draw terrain overlays at all assigned positions
                        if let Some(st) = set_type {
                            if let Some(terrain_data) = editor_state.selected_terrain_set
                                .and_then(|ts_id| project.autotile_config.get_terrain_set(ts_id))
                                .and_then(|ts| ts.get_tile_terrain(virtual_index))
                            {
                                draw_terrain_overlays(
                                    ui.painter(),
                                    rect,
                                    st,
                                    terrain_data,
                                    &terrain_colors,
                                );
                            }

                            // Draw center indicator for Mixed terrain (shows it's not clickable)
                            if st == TerrainSetType::Mixed {
                                draw_mixed_center_indicator(ui.painter(), rect);
                            }
                        }

                        // Draw thin border to show tile boundaries
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(1.0, Color32::from_gray(80)),
                            egui::StrokeKind::Inside,
                        );

                        // Handle click or drag - detect which zone was clicked
                        // Drag support allows painting over multiple tiles
                        if response.clicked() || response.dragged() {
                            if let Some(click_pos) = response.interact_pointer_pos() {
                                let local_x = click_pos.x - rect.left();
                                let local_y = click_pos.y - rect.top();

                                if let Some(st) = set_type {
                                    if let Some(position) = detect_click_zone(
                                        local_x,
                                        local_y,
                                        display_size.x,
                                        st,
                                    ) {
                                        handle_terrain_position_click(
                                            editor_state,
                                            project,
                                            virtual_index,
                                            position,
                                            ui.input(|i| i.modifiers.ctrl),
                                        );
                                    }
                                }
                            }
                        }

                        // Build tooltip with terrain info for all positions
                        let tooltip_text = build_tile_tooltip(
                            virtual_index,
                            set_type,
                            editor_state,
                            project,
                        );
                        response.on_hover_text(tooltip_text);
                    }
                });
            }
        });

        virtual_offset += image.tile_count();
    }
}

/// Build tooltip text showing terrain assignments at all positions
fn build_tile_tooltip(
    tile_index: u32,
    set_type: Option<TerrainSetType>,
    editor_state: &EditorState,
    project: &Project,
) -> String {
    let mut text = format!("Tile {}", tile_index);

    if let Some(st) = set_type {
        if let Some(terrain_data) = editor_state.selected_terrain_set
            .and_then(|ts_id| project.autotile_config.get_terrain_set(ts_id))
            .and_then(|ts| ts.get_tile_terrain(tile_index))
        {
            text.push_str("\n\nTerrain assignments:");
            let position_count = st.position_count();
            for pos in 0..position_count {
                let pos_name = st.position_name(pos);
                let terrain_name = terrain_data.get(pos)
                    .and_then(|idx| {
                        editor_state.selected_terrain_set
                            .and_then(|ts_id| project.autotile_config.get_terrain_set(ts_id))
                            .and_then(|ts| ts.terrains.get(idx))
                            .map(|t| t.name.as_str())
                    })
                    .unwrap_or("-");
                text.push_str(&format!("\n  {}: {}", pos_name, terrain_name));
            }
        } else {
            text.push_str("\n\nNo terrain assigned");
        }
    }

    text.push_str("\n\nClick corner/edge to paint");
    text.push_str("\nCtrl+click to clear");
    text
}

/// Handle click on a specific terrain position within a tile (Tiled-style painting)
fn handle_terrain_position_click(
    editor_state: &mut EditorState,
    project: &mut Project,
    tile_index: u32,
    position: usize,
    ctrl_held: bool,
) {
    let Some(ts_id) = editor_state.selected_terrain_set else {
        return;
    };

    // Get the terrain set mutably
    let Some(terrain_set) = project.autotile_config.get_terrain_set_mut(ts_id) else {
        return;
    };

    if ctrl_held {
        // Clear terrain assignment at this position
        terrain_set.set_tile_terrain(tile_index, position, None);
    } else if let Some(terrain_idx) = editor_state.tileset_editor_state.selected_terrain_for_assignment {
        // Set terrain assignment at this position
        terrain_set.set_tile_terrain(tile_index, position, Some(terrain_idx));
    }

    project.mark_dirty();
}

fn render_tile_properties_tab(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &mut Project,
    cache: Option<&TilesetTextureCache>,
) {
    let Some(tileset_id) = editor_state.selected_tileset else {
        ui.label("No tileset selected");
        return;
    };

    // Clone tileset data to avoid borrow conflicts
    let tileset_data = project.tilesets.iter()
        .find(|t| t.id == tileset_id)
        .map(|t| (t.tile_size, t.images.clone(), !t.images.is_empty()));

    // Split into left panel (tile selector) and right panel (properties editor)
    ui.horizontal(|ui| {
        // Left panel: Tile selector
        ui.vertical(|ui| {
            ui.set_min_width(250.0);
            ui.heading("Select Tile");

            if let Some((tile_size, images, has_images)) = tileset_data.clone() {
                if !has_images {
                    ui.label("No images in tileset");
                } else {
                    egui::ScrollArea::both()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            render_tile_selector_for_properties(
                                ui,
                                editor_state,
                                tile_size,
                                &images,
                                cache,
                            );
                        });
                }
            }
        });

        ui.separator();

        // Right panel: Property editor
        ui.vertical(|ui| {
            ui.heading("Tile Properties");

            if let Some(tile_idx) = editor_state.tileset_editor_state.selected_tile_for_properties {
                ui.label(format!("Editing Tile {}", tile_idx));
                ui.separator();

                // Get current properties (or default)
                let tileset = project.tilesets.iter().find(|t| t.id == tileset_id);
                let current_props = tileset
                    .and_then(|t| t.get_tile_properties(tile_idx))
                    .cloned()
                    .unwrap_or_default();

                // Collision settings
                ui.heading("Collision");

                let mut collision = current_props.collision;
                if ui.checkbox(&mut collision, "Has collision").changed() {
                    if let Some(tileset) = project.tilesets.iter_mut().find(|t| t.id == tileset_id) {
                        tileset.set_tile_collision(tile_idx, collision);
                        project.mark_dirty();
                    }
                }

                let mut one_way = current_props.one_way;
                ui.add_enabled_ui(collision, |ui| {
                    if ui.checkbox(&mut one_way, "One-way platform (collision from above only)").changed() {
                        if let Some(tileset) = project.tilesets.iter_mut().find(|t| t.id == tileset_id) {
                            let props = tileset.get_tile_properties_mut(tile_idx);
                            props.one_way = one_way;
                            project.mark_dirty();
                        }
                    }
                });

                ui.separator();

                // Animation settings
                ui.heading("Animation");

                let has_anim = current_props.animation_frames.is_some();
                let mut enable_anim = has_anim;

                if ui.checkbox(&mut enable_anim, "Enable animation").changed() {
                    if let Some(tileset) = project.tilesets.iter_mut().find(|t| t.id == tileset_id) {
                        let props = tileset.get_tile_properties_mut(tile_idx);
                        if enable_anim {
                            props.animation_frames = Some(vec![tile_idx]);
                            props.animation_speed = Some(10.0);
                        } else {
                            props.animation_frames = None;
                            props.animation_speed = None;
                        }
                        project.mark_dirty();
                    }
                }

                if enable_anim {
                    // Animation speed
                    let mut speed = current_props.animation_speed.unwrap_or(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Speed (FPS):");
                        if ui.add(egui::DragValue::new(&mut speed).range(0.1..=60.0)).changed() {
                            if let Some(tileset) = project.tilesets.iter_mut().find(|t| t.id == tileset_id) {
                                let props = tileset.get_tile_properties_mut(tile_idx);
                                props.animation_speed = Some(speed);
                                project.mark_dirty();
                            }
                        }
                    });

                    // Animation frames display
                    ui.label("Frames:");
                    let frames = current_props.animation_frames.clone().unwrap_or_default();
                    ui.horizontal_wrapped(|ui| {
                        for frame in &frames {
                            ui.label(format!("{}", frame));
                        }
                    });
                    ui.small("(Frame editing coming in future update)");
                }

                ui.separator();

                // Custom properties
                ui.heading("Custom Properties");
                if current_props.custom.is_empty() {
                    ui.label("No custom properties set");
                } else {
                    for (key, value) in &current_props.custom {
                        ui.horizontal(|ui| {
                            ui.label(key);
                            ui.label(":");
                            ui.label(format!("{}", value));
                        });
                    }
                }
                ui.small("(Custom property editing coming in future update)");

            } else {
                ui.label("Click a tile on the left to edit its properties");
            }
        });
    });
}

/// Render tile selector grid for the properties tab
fn render_tile_selector_for_properties(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    _tile_size: u32,
    images: &[bevy_map_core::TilesetImage],
    cache: Option<&TilesetTextureCache>,
) {
    let display_size = egui::vec2(32.0, 32.0);
    let mut virtual_offset = 0u32;

    for image in images {
        let texture_id = cache
            .and_then(|c| c.loaded.get(&image.id))
            .map(|(_, tex_id, _, _)| *tex_id);

        ui.collapsing(&image.name, |ui| {
            if image.columns == 0 || image.rows == 0 {
                ui.label("Image not loaded yet");
                return;
            }

            let uv_tile_width = 1.0 / image.columns.max(1) as f32;
            let uv_tile_height = 1.0 / image.rows.max(1) as f32;

            for row in 0..image.rows {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);

                    for col in 0..image.columns {
                        let local_index = row * image.columns + col;
                        let virtual_index = virtual_offset + local_index;
                        let selected = editor_state.tileset_editor_state.selected_tile_for_properties == Some(virtual_index);

                        let response = if let Some(tex_id) = texture_id {
                            let uv_min = egui::pos2(col as f32 * uv_tile_width, row as f32 * uv_tile_height);
                            let uv_max = egui::pos2((col + 1) as f32 * uv_tile_width, (row + 1) as f32 * uv_tile_height);

                            #[allow(deprecated)]
                            ui.add(
                                egui::ImageButton::new(egui::load::SizedTexture::new(tex_id, display_size))
                                    .uv(egui::Rect::from_min_max(uv_min, uv_max))
                                    .selected(selected)
                                    .rounding(0.0)
                            )
                        } else {
                            ui.add(
                                egui::Button::new(format!("{}", virtual_index))
                                    .min_size(display_size)
                                    .selected(selected)
                            )
                        };

                        if response.clicked() {
                            editor_state.tileset_editor_state.selected_tile_for_properties = Some(virtual_index);
                        }

                        response.on_hover_text(format!("Tile {}", virtual_index));
                    }
                });
            }
        });

        virtual_offset += image.tile_count();
    }
}
