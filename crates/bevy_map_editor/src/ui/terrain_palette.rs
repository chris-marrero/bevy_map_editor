//! Terrain painting palette

use bevy_egui::egui;
use uuid::Uuid;

use super::TilesetTextureCache;
use crate::project::Project;
use crate::EditorState;

/// State for terrain painting
pub struct TerrainPaintState {
    /// Currently selected terrain set
    pub selected_terrain_set: Option<Uuid>,
    /// Currently selected terrain index within the set
    pub selected_terrain_idx: Option<usize>,
    /// Whether painting with terrain brush
    pub is_terrain_mode: bool,
}

impl TerrainPaintState {
    pub fn new() -> Self {
        Self {
            selected_terrain_set: None,
            selected_terrain_idx: None,
            is_terrain_mode: false,
        }
    }
}

impl Default for TerrainPaintState {
    fn default() -> Self {
        Self::new()
    }
}

/// Tab mode for the palette
#[derive(Clone, Copy, PartialEq, Default)]
pub enum PaletteTab {
    #[default]
    Tiles,
    Terrains,
}

/// Render the terrain palette
pub fn render_terrain_palette(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &Project,
    cache: Option<&TilesetTextureCache>,
) {
    // Determine current tab based on state
    let current_tab = if editor_state.terrain_paint_state.is_terrain_mode {
        PaletteTab::Terrains
    } else {
        PaletteTab::Tiles
    };

    // Tab for switching between tiles and terrains
    ui.horizontal(|ui| {
        if ui.selectable_label(current_tab == PaletteTab::Tiles, "Tiles").clicked() {
            editor_state.terrain_paint_state.is_terrain_mode = false;
        }
        if ui.selectable_label(current_tab == PaletteTab::Terrains, "Terrains").clicked() {
            editor_state.terrain_paint_state.is_terrain_mode = true;
        }
    });

    ui.separator();

    match current_tab {
        PaletteTab::Tiles => {
            super::render_tileset_palette_with_cache(ui, editor_state, project, cache);
        }
        PaletteTab::Terrains => {
            render_terrain_brushes(ui, editor_state, project);
        }
    }
}

fn render_terrain_brushes(ui: &mut egui::Ui, editor_state: &mut EditorState, project: &Project) {
    let Some(tileset_id) = editor_state.selected_tileset else {
        ui.label("No tileset selected");
        return;
    };

    // Get terrain sets for this tileset
    let terrain_sets: Vec<_> = project.autotile_config.terrain_sets
        .iter()
        .filter(|ts| ts.tileset_id == tileset_id)
        .collect();

    if terrain_sets.is_empty() {
        ui.label("No terrain sets defined for this tileset.");
        ui.label("Use the Tileset Editor to create terrain sets.");
        return;
    }

    // Terrain set selector
    let current_set_name = editor_state.terrain_paint_state.selected_terrain_set
        .and_then(|id| terrain_sets.iter().find(|ts| ts.id == id))
        .map(|ts| ts.name.as_str())
        .unwrap_or("(none)");

    egui::ComboBox::from_label("Terrain Set")
        .selected_text(current_set_name)
        .show_ui(ui, |ui| {
            for ts in &terrain_sets {
                if ui.selectable_value(
                    &mut editor_state.terrain_paint_state.selected_terrain_set,
                    Some(ts.id),
                    &ts.name,
                ).clicked() {
                    // Update local paint state
                    editor_state.terrain_paint_state.selected_terrain_idx = None;

                    // CRITICAL: Also update root EditorState fields that tools/mod.rs reads
                    editor_state.selected_terrain_set = Some(ts.id);
                    editor_state.selected_terrain_in_set = None;
                    editor_state.selected_tileset = Some(ts.tileset_id);
                }
            }
        });

    // Show terrains in the selected set
    if let Some(set_id) = editor_state.terrain_paint_state.selected_terrain_set {
        if let Some(terrain_set) = terrain_sets.iter().find(|ts| ts.id == set_id) {
            ui.label(format!("Type: {:?}", terrain_set.set_type));
            ui.separator();

            for (idx, terrain) in terrain_set.terrains.iter().enumerate() {
                let selected = editor_state.terrain_paint_state.selected_terrain_idx == Some(idx);

                ui.horizontal(|ui| {
                    // Color swatch
                    let color = egui::Color32::from_rgb(
                        (terrain.color.r * 255.0) as u8,
                        (terrain.color.g * 255.0) as u8,
                        (terrain.color.b * 255.0) as u8,
                    );
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 0.0, color);

                    if ui.selectable_label(selected, &terrain.name).clicked() {
                        // Update local paint state
                        editor_state.terrain_paint_state.selected_terrain_idx = Some(idx);
                        editor_state.terrain_paint_state.selected_terrain_set = Some(terrain_set.id);

                        // CRITICAL: Also update root EditorState fields that tools/mod.rs reads
                        editor_state.selected_terrain_set = Some(terrain_set.id);
                        editor_state.selected_terrain_in_set = Some(idx);
                        editor_state.selected_tileset = Some(terrain_set.tileset_id);
                    }
                });
            }
        }
    }
}
