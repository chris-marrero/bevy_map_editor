//! Tileset and terrain editor window
//!
//! Implements Tiled-style terrain corner/edge painting where users click
//! directly on zones within each tile to assign terrain types.
//!
//! This follows Tiled's tileset editor approach:
//! - Each tile has clickable zones (corners, edges, or both for Mixed)
//! - Clicking assigns the selected terrain to that zone of that tile
//! - Visual overlays show assigned terrain with curved boundaries

use bevy_egui::egui::{self, Color32, Pos2, Shape};
use bevy_map_autotile::terrain::Color as TerrainColor;
use bevy_map_autotile::TerrainSetType;
use std::f32::consts::PI;

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
// Per-Tile Zone Detection (Tiled-style)
// ============================================================================

/// Determine which zone was clicked within a tile based on local coordinates.
///
/// For Corner terrain sets (4 zones):
/// ```text
/// +-------+
/// | 0 | 1 |  TL=0, TR=1
/// |---+---|
/// | 2 | 3 |  BL=2, BR=3
/// +-------+
/// ```
///
/// For Edge terrain sets (4 zones):
/// ```text
/// +---0---+
/// |       |  Top=0, Right=1, Bottom=2, Left=3
/// 3       1
/// |       |
/// +---2---+
/// ```
///
/// For Mixed terrain sets (8 zones, center ignored):
/// ```text
/// +---+---+---+
/// | 0 | 1 | 2 |  TL=0, Top=1, TR=2
/// +---+---+---+
/// | 7 | X | 3 |  Left=7, (center ignored), Right=3
/// +---+---+---+
/// | 6 | 5 | 4 |  BL=6, Bottom=5, BR=4
/// +---+---+---+
/// ```
fn get_tile_zone(local_x: f32, local_y: f32, set_type: TerrainSetType) -> Option<usize> {
    match set_type {
        TerrainSetType::Corner => {
            // Simple quadrant detection
            let right = local_x > 0.5;
            let bottom = local_y > 0.5;
            Some(match (right, bottom) {
                (false, false) => 0, // TL
                (true, false) => 1,  // TR
                (false, true) => 2,  // BL
                (true, true) => 3,   // BR
            })
        }
        TerrainSetType::Edge => {
            // Find nearest edge by comparing distance to center
            let dx = (local_x - 0.5).abs();
            let dy = (local_y - 0.5).abs();
            if dy > dx {
                // Vertical distance larger = closer to top/bottom edge
                Some(if local_y < 0.5 { 0 } else { 2 }) // Top or Bottom
            } else {
                // Horizontal distance larger = closer to left/right edge
                Some(if local_x < 0.5 { 3 } else { 1 }) // Left or Right
            }
        }
        TerrainSetType::Mixed => {
            // 3x3 grid, center returns None (not clickable)
            let zone_x = if local_x < 0.33 {
                0
            } else if local_x < 0.67 {
                1
            } else {
                2
            };
            let zone_y = if local_y < 0.33 {
                0
            } else if local_y < 0.67 {
                1
            } else {
                2
            };
            match (zone_x, zone_y) {
                (0, 0) => Some(0), // TL
                (1, 0) => Some(1), // Top
                (2, 0) => Some(2), // TR
                (2, 1) => Some(3), // Right
                (2, 2) => Some(4), // BR
                (1, 2) => Some(5), // Bottom
                (0, 2) => Some(6), // BL
                (0, 1) => Some(7), // Left
                (1, 1) => None,    // Center - not clickable
                _ => None,
            }
        }
    }
}

/// Generate points for an arc approximation
fn arc_points(cx: f32, cy: f32, r: f32, start: f32, end: f32, segments: usize) -> Vec<Pos2> {
    (0..=segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let angle = start + t * (end - start);
            Pos2::new(cx + r * angle.cos(), cy + r * angle.sin())
        })
        .collect()
}

/// Paint terrain at a specific zone within a single tile.
///
/// This is the Tiled-style approach: each click assigns terrain to exactly
/// one position on one tile, rather than affecting multiple tiles.
fn paint_terrain_zone(
    terrain_set: &mut bevy_map_autotile::TerrainSet,
    tile_index: u32,
    position: usize,
    terrain_idx: Option<usize>,
) {
    terrain_set.set_tile_terrain(tile_index, position, terrain_idx);
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
                let overlay_color =
                    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 180);

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

/// Draw a Tiled-style corner overlay with curved inner boundary.
/// Position: 0=TL, 1=TR, 2=BL, 3=BR
/// Uses d = 1/6 of tile size, matching Tiled's wangoverlay.cpp
fn draw_corner_overlay(painter: &egui::Painter, rect: egui::Rect, position: usize, color: Color32) {
    let w = rect.width();
    let d = w / 6.0; // Tiled uses 1/6 of tile size
    let arc_segments = 6;

    let points = match position {
        0 => corner_fill_tl(rect, d, arc_segments),
        1 => corner_fill_tr(rect, d, arc_segments),
        2 => corner_fill_bl(rect, d, arc_segments),
        3 => corner_fill_br(rect, d, arc_segments),
        _ => return,
    };

    painter.add(Shape::convex_polygon(points, color, egui::Stroke::NONE));
}

/// Generate filled region for top-left corner with concave arc boundary
fn corner_fill_tl(rect: egui::Rect, d: f32, segments: usize) -> Vec<Pos2> {
    let tl = rect.left_top();
    let mut pts = vec![tl, Pos2::new(tl.x + 2.0 * d, tl.y)];
    // Arc from (tl.x + 2d, tl.y) curving inward to (tl.x, tl.y + 2d)
    // Arc center at (tl.x + 2d, tl.y + 2d), radius 2d, from -PI/2 to -PI
    pts.extend(arc_points(tl.x + 2.0 * d, tl.y + 2.0 * d, 2.0 * d, -PI / 2.0, -PI, segments));
    pts.push(Pos2::new(tl.x, tl.y + 2.0 * d));
    pts
}

/// Generate filled region for top-right corner with concave arc boundary
fn corner_fill_tr(rect: egui::Rect, d: f32, segments: usize) -> Vec<Pos2> {
    let tr = rect.right_top();
    let mut pts = vec![tr, Pos2::new(tr.x, tr.y + 2.0 * d)];
    // Arc from (tr.x, tr.y + 2d) curving inward to (tr.x - 2d, tr.y)
    // Arc center at (tr.x - 2d, tr.y + 2d), radius 2d, from 0 to -PI/2
    pts.extend(arc_points(tr.x - 2.0 * d, tr.y + 2.0 * d, 2.0 * d, 0.0, -PI / 2.0, segments));
    pts.push(Pos2::new(tr.x - 2.0 * d, tr.y));
    pts
}

/// Generate filled region for bottom-left corner with concave arc boundary
fn corner_fill_bl(rect: egui::Rect, d: f32, segments: usize) -> Vec<Pos2> {
    let bl = rect.left_bottom();
    let mut pts = vec![bl, Pos2::new(bl.x, bl.y - 2.0 * d)];
    // Arc from (bl.x, bl.y - 2d) curving inward to (bl.x + 2d, bl.y)
    // Arc center at (bl.x + 2d, bl.y - 2d), radius 2d, from PI to PI/2
    pts.extend(arc_points(bl.x + 2.0 * d, bl.y - 2.0 * d, 2.0 * d, PI, PI / 2.0, segments));
    pts.push(Pos2::new(bl.x + 2.0 * d, bl.y));
    pts
}

/// Generate filled region for bottom-right corner with concave arc boundary
fn corner_fill_br(rect: egui::Rect, d: f32, segments: usize) -> Vec<Pos2> {
    let br = rect.right_bottom();
    let mut pts = vec![br, Pos2::new(br.x - 2.0 * d, br.y)];
    // Arc from (br.x - 2d, br.y) curving inward to (br.x, br.y - 2d)
    // Arc center at (br.x - 2d, br.y - 2d), radius 2d, from PI/2 to 0
    pts.extend(arc_points(br.x - 2.0 * d, br.y - 2.0 * d, 2.0 * d, PI / 2.0, 0.0, segments));
    pts.push(Pos2::new(br.x, br.y - 2.0 * d));
    pts
}

/// Draw a Tiled-style edge overlay (strip along edge with rounded ends).
/// Position: 0=Top, 1=Right, 2=Bottom, 3=Left
/// Uses d = 1/6 of tile size, matching Tiled's wangoverlay.cpp
fn draw_edge_overlay(painter: &egui::Painter, rect: egui::Rect, position: usize, color: Color32) {
    let w = rect.width();
    let d = w / 6.0; // Tiled uses 1/6 of tile size
    let arc_segments = 4;

    let points = match position {
        0 => edge_strip_top(rect, d, arc_segments),
        1 => edge_strip_right(rect, d, arc_segments),
        2 => edge_strip_bottom(rect, d, arc_segments),
        3 => edge_strip_left(rect, d, arc_segments),
        _ => return,
    };

    painter.add(Shape::convex_polygon(points, color, egui::Stroke::NONE));
}

/// Generate edge strip for top edge
fn edge_strip_top(rect: egui::Rect, d: f32, segments: usize) -> Vec<Pos2> {
    let left = rect.left() + 2.0 * d;
    let right = rect.right() - 2.0 * d;
    let top = rect.top();

    let mut pts = Vec::new();
    // Start at top-left of strip
    pts.push(Pos2::new(left, top));
    // Go to top-right
    pts.push(Pos2::new(right, top));
    // Arc at right end (semicircle going down and back)
    pts.extend(arc_points(right, top + d, d, -PI / 2.0, PI / 2.0, segments));
    // Go back left along bottom
    pts.push(Pos2::new(left, top + 2.0 * d));
    // Arc at left end (semicircle going up and back)
    pts.extend(arc_points(left, top + d, d, PI / 2.0, 3.0 * PI / 2.0, segments));
    pts
}

/// Generate edge strip for right edge
fn edge_strip_right(rect: egui::Rect, d: f32, segments: usize) -> Vec<Pos2> {
    let top = rect.top() + 2.0 * d;
    let bottom = rect.bottom() - 2.0 * d;
    let right = rect.right();

    let mut pts = Vec::new();
    pts.push(Pos2::new(right, top));
    pts.push(Pos2::new(right, bottom));
    // Arc at bottom (semicircle going left and back)
    pts.extend(arc_points(right - d, bottom, d, 0.0, PI, segments));
    pts.push(Pos2::new(right - 2.0 * d, top));
    // Arc at top (semicircle going right and back)
    pts.extend(arc_points(right - d, top, d, PI, 2.0 * PI, segments));
    pts
}

/// Generate edge strip for bottom edge
fn edge_strip_bottom(rect: egui::Rect, d: f32, segments: usize) -> Vec<Pos2> {
    let left = rect.left() + 2.0 * d;
    let right = rect.right() - 2.0 * d;
    let bottom = rect.bottom();

    let mut pts = Vec::new();
    pts.push(Pos2::new(left, bottom));
    pts.push(Pos2::new(right, bottom));
    // Arc at right end (semicircle going up and back)
    pts.extend(arc_points(right, bottom - d, d, PI / 2.0, -PI / 2.0, segments));
    pts.push(Pos2::new(left, bottom - 2.0 * d));
    // Arc at left end (semicircle going down and back)
    pts.extend(arc_points(left, bottom - d, d, -PI / 2.0, -3.0 * PI / 2.0, segments));
    pts
}

/// Generate edge strip for left edge
fn edge_strip_left(rect: egui::Rect, d: f32, segments: usize) -> Vec<Pos2> {
    let top = rect.top() + 2.0 * d;
    let bottom = rect.bottom() - 2.0 * d;
    let left = rect.left();

    let mut pts = Vec::new();
    pts.push(Pos2::new(left, top));
    pts.push(Pos2::new(left, bottom));
    // Arc at bottom (semicircle going right and back)
    pts.extend(arc_points(left + d, bottom, d, PI, 0.0, segments));
    pts.push(Pos2::new(left + 2.0 * d, top));
    // Arc at top (semicircle going left and back)
    pts.extend(arc_points(left + d, top, d, 0.0, -PI, segments));
    pts
}

/// Draw a mixed overlay (for Mixed type terrain sets - 3x3 grid)
/// Position: 0=TL, 1=Top, 2=TR, 3=Right, 4=BR, 5=Bottom, 6=BL, 7=Left
fn draw_mixed_overlay(painter: &egui::Painter, rect: egui::Rect, position: usize, color: Color32) {
    let w = rect.width() / 3.0;
    let h = rect.height() / 3.0;
    let left = rect.left();
    let top = rect.top();

    // Calculate cell position based on 3x3 grid layout
    let cell_rect = match position {
        0 => egui::Rect::from_min_size(egui::pos2(left, top), egui::vec2(w, h)), // TL
        1 => egui::Rect::from_min_size(egui::pos2(left + w, top), egui::vec2(w, h)), // Top
        2 => egui::Rect::from_min_size(egui::pos2(left + 2.0 * w, top), egui::vec2(w, h)), // TR
        3 => egui::Rect::from_min_size(egui::pos2(left + 2.0 * w, top + h), egui::vec2(w, h)), // Right
        4 => egui::Rect::from_min_size(egui::pos2(left + 2.0 * w, top + 2.0 * h), egui::vec2(w, h)), // BR
        5 => egui::Rect::from_min_size(egui::pos2(left + w, top + 2.0 * h), egui::vec2(w, h)), // Bottom
        6 => egui::Rect::from_min_size(egui::pos2(left, top + 2.0 * h), egui::vec2(w, h)),     // BL
        7 => egui::Rect::from_min_size(egui::pos2(left, top + h), egui::vec2(w, h)), // Left
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
    let center_rect = egui::Rect::from_min_size(egui::pos2(left + w, top + h), egui::vec2(w, h));

    // Draw a subtle X pattern to indicate non-clickable
    let gray = Color32::from_rgba_unmultiplied(128, 128, 128, 60);
    let tl = center_rect.left_top();
    let tr = center_rect.right_top();
    let bl = center_rect.left_bottom();
    let br = center_rect.right_bottom();

    painter.line_segment([tl, br], egui::Stroke::new(1.0, gray));
    painter.line_segment([tr, bl], egui::Stroke::new(1.0, gray));
}

/// Draw hover highlight for the currently hovered zone within a tile.
/// This gives visual feedback showing which zone will be affected on click.
fn draw_zone_hover_highlight(
    painter: &egui::Painter,
    rect: egui::Rect,
    position: usize,
    set_type: TerrainSetType,
) {
    let hover_color = Color32::from_rgba_unmultiplied(255, 255, 100, 100);

    match set_type {
        TerrainSetType::Corner => {
            draw_corner_overlay(painter, rect, position, hover_color);
        }
        TerrainSetType::Edge => {
            draw_edge_overlay(painter, rect, position, hover_color);
        }
        TerrainSetType::Mixed => {
            draw_mixed_overlay(painter, rect, position, hover_color);
        }
    }
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
                if ui
                    .selectable_label(
                        editor_state.tileset_editor_state.selected_tab == TilesetEditorTab::Images,
                        "Images",
                    )
                    .clicked()
                {
                    editor_state.tileset_editor_state.selected_tab = TilesetEditorTab::Images;
                }
                if ui
                    .selectable_label(
                        editor_state.tileset_editor_state.selected_tab
                            == TilesetEditorTab::TerrainSets,
                        "Terrain Sets",
                    )
                    .clicked()
                {
                    editor_state.tileset_editor_state.selected_tab = TilesetEditorTab::TerrainSets;
                }
                if ui
                    .selectable_label(
                        editor_state.tileset_editor_state.selected_tab
                            == TilesetEditorTab::TileProperties,
                        "Tile Properties",
                    )
                    .clicked()
                {
                    editor_state.tileset_editor_state.selected_tab =
                        TilesetEditorTab::TileProperties;
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
    let tileset_data = project
        .tilesets
        .iter()
        .find(|t| t.id == tileset_id)
        .map(|t| (t.tile_size, t.images.clone(), !t.images.is_empty()));

    // Split into left panel (terrain list) and right panel (tileset preview)
    // Using columns() for proper space distribution like the animation editor
    ui.columns(2, |columns| {
        // Left column: Terrain set and terrain list (with scroll area to prevent clipping)
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(&mut columns[0], |ui| {
            ui.heading("Terrain Sets");

            // List terrain sets for this tileset
            let terrain_sets: Vec<_> = project
                .autotile_config
                .terrain_sets
                .iter()
                .filter(|ts| ts.tileset_id == tileset_id)
                .map(|ts| (ts.id, ts.name.clone(), ts.set_type, ts.terrains.len()))
                .collect();

            for (ts_id, name, set_type, terrain_count) in &terrain_sets {
                ui.horizontal(|ui| {
                    let selected = editor_state.selected_terrain_set == Some(*ts_id);
                    if ui.selectable_label(selected, name).clicked() {
                        editor_state.selected_terrain_set = Some(*ts_id);
                        editor_state
                            .tileset_editor_state
                            .selected_terrain_for_assignment = None;
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
                            let selected = editor_state
                                .tileset_editor_state
                                .selected_terrain_for_assignment
                                == Some(idx);

                            // Color swatch
                            let (rect, _) = ui
                                .allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 2.0, color);

                            // Terrain name with selection
                            if ui.selectable_label(selected, &terrain.name).clicked() {
                                editor_state
                                    .tileset_editor_state
                                    .selected_terrain_for_assignment = Some(idx);
                            }
                        });
                    }

                    // Buttons
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("+ Add Terrain").clicked() {
                            editor_state.show_add_terrain_to_set_dialog = true;
                        }
                        if editor_state
                            .tileset_editor_state
                            .selected_terrain_for_assignment
                            .is_some()
                        {
                            if ui.button("Clear Selection").clicked() {
                                editor_state
                                    .tileset_editor_state
                                    .selected_terrain_for_assignment = None;
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

/// Render the tileset with Tiled-style per-tile zone painting.
/// Users click on zones within each tile to assign terrain to that tile's corner/edge/mixed position.
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
    let tile_display_size = 48.0f32;
    let display_size = egui::vec2(tile_display_size, tile_display_size);
    let mut virtual_offset = 0u32;

    // Get terrain set info for overlays
    let terrain_info = editor_state
        .selected_terrain_set
        .and_then(|ts_id| project.autotile_config.get_terrain_set(ts_id))
        .map(|ts| {
            let colors: Vec<Color32> = ts
                .terrains
                .iter()
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

        let image_virtual_offset = virtual_offset;

        ui.collapsing(&image.name, |ui| {
            if image.columns == 0 || image.rows == 0 {
                ui.label("Image not loaded yet");
                return;
            }

            let uv_tile_width = 1.0 / image.columns.max(1) as f32;
            let uv_tile_height = 1.0 / image.rows.max(1) as f32;

            // Calculate full grid size (with spacing)
            let spacing = 2.0f32;
            let grid_width =
                image.columns as f32 * tile_display_size + (image.columns - 1) as f32 * spacing;
            let grid_height =
                image.rows as f32 * tile_display_size + (image.rows - 1) as f32 * spacing;

            // Allocate the entire grid area for interaction
            let (grid_rect, grid_response) = ui.allocate_exact_size(
                egui::vec2(grid_width, grid_height),
                egui::Sense::click_and_drag(),
            );

            let grid_origin = grid_rect.left_top();

            // Draw all tiles
            for row in 0..image.rows {
                for col in 0..image.columns {
                    let local_index = row * image.columns + col;
                    let virtual_index = image_virtual_offset + local_index;

                    // Calculate tile rect (accounting for spacing)
                    let tile_x =
                        grid_origin.x + col as f32 * (tile_display_size + spacing);
                    let tile_y =
                        grid_origin.y + row as f32 * (tile_display_size + spacing);
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(tile_x, tile_y),
                        display_size,
                    );

                    // Draw tile texture
                    if let Some(tex_id) = texture_id {
                        let uv_min =
                            egui::pos2(col as f32 * uv_tile_width, row as f32 * uv_tile_height);
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
                        if let Some(terrain_data) = editor_state
                            .selected_terrain_set
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
                }
            }

            // Handle zone-based interaction (Tiled-style per-tile zones)
            if let Some(st) = set_type {
                let effective_tile_size = tile_display_size + spacing;

                // Helper to convert pointer position to tile + local coordinates
                let get_tile_and_local = |pos: egui::Pos2| -> Option<(u32, f32, f32, egui::Rect)> {
                    if !grid_rect.contains(pos) {
                        return None;
                    }
                    let rel_x = pos.x - grid_origin.x;
                    let rel_y = pos.y - grid_origin.y;

                    // Find which tile cell (accounting for spacing)
                    let col = (rel_x / effective_tile_size).floor() as i32;
                    let row = (rel_y / effective_tile_size).floor() as i32;

                    if col < 0 || col >= image.columns as i32 || row < 0 || row >= image.rows as i32 {
                        return None;
                    }

                    let col = col as u32;
                    let row = row as u32;

                    // Calculate tile rect
                    let tile_x = grid_origin.x + col as f32 * effective_tile_size;
                    let tile_y = grid_origin.y + row as f32 * effective_tile_size;
                    let tile_rect = egui::Rect::from_min_size(
                        egui::pos2(tile_x, tile_y),
                        egui::vec2(tile_display_size, tile_display_size),
                    );

                    // Only consider clicks within the actual tile (not spacing)
                    if !tile_rect.contains(pos) {
                        return None;
                    }

                    // Calculate local coordinates (0.0 to 1.0) within tile
                    let local_x = (pos.x - tile_rect.left()) / tile_display_size;
                    let local_y = (pos.y - tile_rect.top()) / tile_display_size;

                    let tile_index = image_virtual_offset + row * image.columns + col;
                    Some((tile_index, local_x, local_y, tile_rect))
                };

                // Draw hover highlight on the zone under cursor
                if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    if let Some((tile_index, local_x, local_y, tile_rect)) = get_tile_and_local(hover_pos) {
                        if let Some(position) = get_tile_zone(local_x, local_y, st) {
                            // Draw hover highlight
                            draw_zone_hover_highlight(ui.painter(), tile_rect, position, st);

                            // Show tooltip
                            let zone_name = match st {
                                TerrainSetType::Corner => match position {
                                    0 => "Top-Left",
                                    1 => "Top-Right",
                                    2 => "Bottom-Left",
                                    3 => "Bottom-Right",
                                    _ => "Unknown",
                                },
                                TerrainSetType::Edge => match position {
                                    0 => "Top",
                                    1 => "Right",
                                    2 => "Bottom",
                                    3 => "Left",
                                    _ => "Unknown",
                                },
                                TerrainSetType::Mixed => match position {
                                    0 => "Top-Left",
                                    1 => "Top",
                                    2 => "Top-Right",
                                    3 => "Right",
                                    4 => "Bottom-Right",
                                    5 => "Bottom",
                                    6 => "Bottom-Left",
                                    7 => "Left",
                                    _ => "Unknown",
                                },
                            };
                            let tooltip = format!(
                                "Tile {} - {} zone\nClick to paint\nCtrl+click to clear",
                                tile_index, zone_name
                            );
                            grid_response.clone().on_hover_text(tooltip);
                        }
                    }
                }

                // Handle clicks - paint to single tile's zone
                if grid_response.clicked() || grid_response.dragged() {
                    if let Some(click_pos) = grid_response.interact_pointer_pos() {
                        if let Some((tile_index, local_x, local_y, _tile_rect)) = get_tile_and_local(click_pos) {
                            if let Some(position) = get_tile_zone(local_x, local_y, st) {
                                let ctrl_held = ui.input(|i| i.modifiers.ctrl);
                                let terrain_idx = if ctrl_held {
                                    None // Erase
                                } else {
                                    editor_state
                                        .tileset_editor_state
                                        .selected_terrain_for_assignment
                                };

                                // Paint to this tile's zone only
                                if let Some(ts_id) = editor_state.selected_terrain_set {
                                    if let Some(terrain_set) =
                                        project.autotile_config.get_terrain_set_mut(ts_id)
                                    {
                                        paint_terrain_zone(
                                            terrain_set,
                                            tile_index,
                                            position,
                                            terrain_idx,
                                        );
                                        project.mark_dirty();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        virtual_offset += image.tile_count();
    }
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
    let tileset_data = project
        .tilesets
        .iter()
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
                    egui::ScrollArea::both().max_height(400.0).show(ui, |ui| {
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

            if let Some(tile_idx) = editor_state
                .tileset_editor_state
                .selected_tile_for_properties
            {
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
                    if let Some(tileset) = project.tilesets.iter_mut().find(|t| t.id == tileset_id)
                    {
                        tileset.set_tile_collision(tile_idx, collision);
                        project.mark_dirty();
                    }
                }

                let mut one_way = current_props.one_way;
                ui.add_enabled_ui(collision, |ui| {
                    if ui
                        .checkbox(&mut one_way, "One-way platform (collision from above only)")
                        .changed()
                    {
                        if let Some(tileset) =
                            project.tilesets.iter_mut().find(|t| t.id == tileset_id)
                        {
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
                    if let Some(tileset) = project.tilesets.iter_mut().find(|t| t.id == tileset_id)
                    {
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
                        if ui
                            .add(egui::DragValue::new(&mut speed).range(0.1..=60.0))
                            .changed()
                        {
                            if let Some(tileset) =
                                project.tilesets.iter_mut().find(|t| t.id == tileset_id)
                            {
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
                        let selected = editor_state
                            .tileset_editor_state
                            .selected_tile_for_properties
                            == Some(virtual_index);

                        let response = if let Some(tex_id) = texture_id {
                            let uv_min =
                                egui::pos2(col as f32 * uv_tile_width, row as f32 * uv_tile_height);
                            let uv_max = egui::pos2(
                                (col + 1) as f32 * uv_tile_width,
                                (row + 1) as f32 * uv_tile_height,
                            );

                            #[allow(deprecated)]
                            ui.add(
                                egui::ImageButton::new(egui::load::SizedTexture::new(
                                    tex_id,
                                    display_size,
                                ))
                                .uv(egui::Rect::from_min_max(uv_min, uv_max))
                                .selected(selected)
                                .rounding(0.0),
                            )
                        } else {
                            ui.add(
                                egui::Button::new(format!("{}", virtual_index))
                                    .min_size(display_size)
                                    .selected(selected),
                            )
                        };

                        if response.clicked() {
                            editor_state
                                .tileset_editor_state
                                .selected_tile_for_properties = Some(virtual_index);
                        }

                        response.on_hover_text(format!("Tile {}", virtual_index));
                    }
                });
            }
        });

        virtual_offset += image.tile_count();
    }
}
