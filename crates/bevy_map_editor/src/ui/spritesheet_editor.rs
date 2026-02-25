//! SpriteSheet Editor - Asset setup for spritesheets
//!
//! This module provides a standalone editor for configuring spritesheet assets:
//! - Loading spritesheet images
//! - Configuring grid dimensions (frame width/height, columns/rows)
//! - Setting pivot points
//! - Preview grid overlay

use bevy_egui::egui;
use bevy_map_animation::SpriteData;
use uuid::Uuid;

/// State for the SpriteSheet Editor
#[derive(Default, Clone)]
pub struct SpriteSheetEditorState {
    /// Asset ID being edited (project-level asset)
    pub asset_id: Option<Uuid>,
    /// Copy of sprite data being edited
    pub sprite_data: SpriteData,
    /// Path input buffer for text field
    pub sheet_path_input: String,
    /// Last loaded sheet path (for detecting changes)
    pub loaded_sheet_path: Option<String>,
    /// Texture ID for the loaded spritesheet
    pub spritesheet_texture_id: Option<egui::TextureId>,
    /// Size of the loaded spritesheet in pixels
    pub spritesheet_size: Option<(f32, f32)>,
    /// Zoom level for grid view
    pub zoom: f32,
    /// Scroll position for grid view
    pub scroll_offset: egui::Vec2,
    /// Hovered frame index for preview highlighting
    pub hovered_frame: Option<usize>,
}

impl SpriteSheetEditorState {
    pub fn new() -> Self {
        Self {
            zoom: 2.0, // Start at 2x zoom for better visibility
            ..Default::default()
        }
    }

    /// Create editor state from sprite data for editing a project-level asset
    pub fn from_sprite_data(sprite_data: SpriteData, asset_id: Uuid) -> Self {
        Self {
            asset_id: Some(asset_id),
            sprite_data: sprite_data.clone(),
            sheet_path_input: sprite_data.sheet_path,
            loaded_sheet_path: None,
            spritesheet_texture_id: None,
            spritesheet_size: None,
            zoom: 2.0,
            scroll_offset: egui::Vec2::ZERO,
            hovered_frame: None,
        }
    }

    /// Check if the spritesheet needs to be (re)loaded
    pub fn needs_texture_load(&self) -> bool {
        let current_path = &self.sprite_data.sheet_path;
        !current_path.is_empty()
            && (self.loaded_sheet_path.as_ref() != Some(current_path)
                || self.spritesheet_texture_id.is_none())
    }

    /// Set the loaded texture info
    pub fn set_texture(&mut self, texture_id: egui::TextureId, width: f32, height: f32) {
        self.spritesheet_texture_id = Some(texture_id);
        self.spritesheet_size = Some((width, height));
        self.loaded_sheet_path = Some(self.sprite_data.sheet_path.clone());
    }

    /// Clear the loaded texture
    pub fn clear_texture(&mut self) {
        self.spritesheet_texture_id = None;
        self.spritesheet_size = None;
        self.loaded_sheet_path = None;
    }

    /// Get the edited sprite data
    pub fn get_sprite_data(&self) -> SpriteData {
        self.sprite_data.clone()
    }
}

/// Result from SpriteSheet Editor rendering
#[derive(Default)]
pub struct SpriteSheetEditorResult {
    /// Whether sprite data was changed and should be saved
    pub changed: bool,
    /// Whether the editor should close
    pub close: bool,
    /// Whether to open file browser for spritesheet selection
    pub browse_spritesheet: bool,
    /// Whether the spritesheet path changed and needs reloading
    pub reload_spritesheet: bool,
}

/// Render the SpriteSheet Editor window
pub fn render_spritesheet_editor(
    ctx: &egui::Context,
    state: &mut SpriteSheetEditorState,
) -> SpriteSheetEditorResult {
    let mut result = SpriteSheetEditorResult::default();
    let mut is_open = true;

    egui::Window::new("SpriteSheet Editor")
        .open(&mut is_open)
        .resizable(true)
        .default_size([900.0, 700.0])
        .show(ctx, |ui| {
            // Top toolbar
            ui.horizontal(|ui| {
                if ui.button("Save & Close").clicked() {
                    result.changed = true;
                    result.close = true;
                }
                if ui.button("Cancel").clicked() {
                    result.close = true;
                }
                ui.separator();
                ui.label("Zoom:");
                if ui.button("-").clicked() {
                    state.zoom = (state.zoom - 0.25).max(0.25);
                }
                ui.label(format!("{:.0}%", state.zoom * 100.0));
                if ui.button("+").clicked() {
                    state.zoom = (state.zoom + 0.25).min(4.0);
                }
            });

            ui.separator();

            // Main content - left: settings, right: grid preview
            ui.columns(2, |columns| {
                // Left column: Spritesheet settings
                columns[0].vertical(|ui| {
                    render_spritesheet_settings(ui, state, &mut result);
                });

                // Right column: Grid preview
                columns[1].vertical(|ui| {
                    render_spritesheet_grid_preview(ui, state);
                });
            });
        });

    // If the window close button was clicked, mark as closed
    if !is_open {
        result.close = true;
    }

    result
}

/// Render spritesheet settings (path, frame size, grid config, pivot)
fn render_spritesheet_settings(
    ui: &mut egui::Ui,
    state: &mut SpriteSheetEditorState,
    result: &mut SpriteSheetEditorResult,
) {
    ui.heading("Spritesheet Settings");

    // Sheet path with Browse button
    ui.horizontal(|ui| {
        ui.label("Sheet Path:");
    });
    ui.horizontal(|ui| {
        let text_response = ui.add(
            egui::TextEdit::singleline(&mut state.sheet_path_input)
                .desired_width(200.0)
                .hint_text("path/to/spritesheet.png"),
        );
        if text_response.changed() {
            state.sprite_data.sheet_path = state.sheet_path_input.clone();
            result.changed = true;
            result.reload_spritesheet = true;
        }

        // Browse button (native only via rfd)
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Browse...").clicked() {
            result.browse_spritesheet = true;
        }

        // Reload button
        if ui.button("⟳").on_hover_text("Reload spritesheet").clicked() {
            state.clear_texture();
            result.reload_spritesheet = true;
        }
    });

    // Show spritesheet info if loaded
    if let Some((width, height)) = state.spritesheet_size {
        ui.label(format!("Image: {}x{} px", width as u32, height as u32));

        // If frame dimensions aren't set yet and image is loaded, suggest auto-detecting
        if state.sprite_data.frame_width == 0 || state.sprite_data.frame_height == 0 {
            ui.colored_label(
                egui::Color32::from_rgb(200, 200, 0),
                "ℹ Set frame size or use auto-detect",
            );
        }
    } else if !state.sprite_data.sheet_path.is_empty() {
        ui.colored_label(egui::Color32::YELLOW, "⏳ Image loading...");
    }

    ui.add_space(8.0);

    // Grid configuration section
    ui.group(|ui| {
        ui.label("Grid Configuration:");

        ui.horizontal(|ui| {
            ui.label("Frame Width:");
            let mut width = state.sprite_data.frame_width as i32;
            if ui
                .add(egui::DragValue::new(&mut width).range(1..=1024))
                .changed()
            {
                state.sprite_data.frame_width = width.max(1) as u32;
                result.changed = true;
            }

            ui.label("Height:");
            let mut height = state.sprite_data.frame_height as i32;
            if ui
                .add(egui::DragValue::new(&mut height).range(1..=1024))
                .changed()
            {
                state.sprite_data.frame_height = height.max(1) as u32;
                result.changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Columns:");
            let mut cols = state.sprite_data.columns as i32;
            if ui
                .add(egui::DragValue::new(&mut cols).range(1..=100))
                .changed()
            {
                state.sprite_data.columns = cols.max(1) as u32;
                result.changed = true;
            }

            ui.label("Rows:");
            let mut rows = state.sprite_data.rows as i32;
            if ui
                .add(egui::DragValue::new(&mut rows).range(1..=100))
                .changed()
            {
                state.sprite_data.rows = rows.max(1) as u32;
                result.changed = true;
            }
        });

        // Auto-detect buttons
        if let Some((img_width, img_height)) = state.spritesheet_size {
            ui.horizontal(|ui| {
                if ui
                    .button("🔍 Auto-detect")
                    .on_hover_text("Calculate rows/columns from frame size")
                    .clicked()
                {
                    if state.sprite_data.frame_width > 0 && state.sprite_data.frame_height > 0 {
                        state.sprite_data.columns =
                            (img_width as u32) / state.sprite_data.frame_width;
                        state.sprite_data.rows =
                            (img_height as u32) / state.sprite_data.frame_height;
                        result.changed = true;
                    }
                }

                // Quick preset buttons for common grid layouts
                if ui.button("1×1").on_hover_text("Single sprite").clicked() {
                    state.sprite_data.columns = 1;
                    state.sprite_data.rows = 1;
                    result.changed = true;
                }
                if ui
                    .button("4×4")
                    .on_hover_text("4 columns, 4 rows")
                    .clicked()
                {
                    state.sprite_data.columns = 4;
                    state.sprite_data.rows = 4;
                    result.changed = true;
                }
                if ui.button("8×1").on_hover_text("8 columns, 1 row").clicked() {
                    state.sprite_data.columns = 8;
                    state.sprite_data.rows = 1;
                    result.changed = true;
                }
            });
        }
    });

    ui.add_space(8.0);

    // Pivot configuration
    ui.group(|ui| {
        ui.label("Pivot Point:");
        ui.horizontal(|ui| {
            ui.label("X:");
            let mut px = state.sprite_data.pivot_x;
            if ui
                .add(egui::DragValue::new(&mut px).range(0.0..=1.0).speed(0.01))
                .changed()
            {
                state.sprite_data.pivot_x = px;
                result.changed = true;
            }

            ui.label("Y:");
            let mut py = state.sprite_data.pivot_y;
            if ui
                .add(egui::DragValue::new(&mut py).range(0.0..=1.0).speed(0.01))
                .changed()
            {
                state.sprite_data.pivot_y = py;
                result.changed = true;
            }

            // Preset pivot buttons
            if ui.button("Center").on_hover_text("0.5, 0.5").clicked() {
                state.sprite_data.pivot_x = 0.5;
                state.sprite_data.pivot_y = 0.5;
                result.changed = true;
            }
            if ui.button("Bottom").on_hover_text("0.5, 1.0").clicked() {
                state.sprite_data.pivot_x = 0.5;
                state.sprite_data.pivot_y = 1.0;
                result.changed = true;
            }
        });
    });

    ui.add_space(8.0);

    let total_frames = state.sprite_data.total_frames();
    ui.label(format!("Total frames: {}", total_frames));

    // Sprite data name
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label("Name:");
        if ui
            .text_edit_singleline(&mut state.sprite_data.name)
            .changed()
        {
            result.changed = true;
        }
    });
}

/// Render the spritesheet grid preview (hover only, no click selection)
fn render_spritesheet_grid_preview(ui: &mut egui::Ui, state: &mut SpriteSheetEditorState) {
    ui.heading("Grid Preview");
    ui.label("Hover over frames to see frame numbers");

    let frame_w = state.sprite_data.frame_width as f32 * state.zoom;
    let frame_h = state.sprite_data.frame_height as f32 * state.zoom;
    let cols = state.sprite_data.columns.max(1);
    let rows = state.sprite_data.rows.max(1);

    let total_w = frame_w * cols as f32;
    let total_h = frame_h * rows as f32;

    // Scroll area for the grid
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (response, painter) =
                ui.allocate_painter(egui::vec2(total_w, total_h), egui::Sense::hover());

            let rect = response.rect;

            // Draw background
            painter.rect_filled(rect, 0.0, egui::Color32::from_gray(40));

            // Draw spritesheet image if loaded
            if let (Some(texture_id), Some((img_width, img_height))) =
                (state.spritesheet_texture_id, state.spritesheet_size)
            {
                // Calculate the portion of the image to show based on grid settings
                let grid_width =
                    state.sprite_data.columns as f32 * state.sprite_data.frame_width as f32;
                let grid_height =
                    state.sprite_data.rows as f32 * state.sprite_data.frame_height as f32;

                // UV coordinates for the grid portion
                let u_max = (grid_width / img_width).min(1.0);
                let v_max = (grid_height / img_height).min(1.0);

                let mesh = egui::Mesh {
                    texture_id,
                    indices: vec![0, 1, 2, 0, 2, 3],
                    vertices: vec![
                        egui::epaint::Vertex {
                            pos: rect.min,
                            uv: egui::pos2(0.0, 0.0),
                            color: egui::Color32::WHITE,
                        },
                        egui::epaint::Vertex {
                            pos: egui::pos2(rect.max.x, rect.min.y),
                            uv: egui::pos2(u_max, 0.0),
                            color: egui::Color32::WHITE,
                        },
                        egui::epaint::Vertex {
                            pos: rect.max,
                            uv: egui::pos2(u_max, v_max),
                            color: egui::Color32::WHITE,
                        },
                        egui::epaint::Vertex {
                            pos: egui::pos2(rect.min.x, rect.max.y),
                            uv: egui::pos2(0.0, v_max),
                            color: egui::Color32::WHITE,
                        },
                    ],
                };
                painter.add(egui::Shape::mesh(mesh));
            }

            // Detect hovered frame
            let mut hovered_idx: Option<usize> = None;
            if let Some(pos) = response.hover_pos() {
                let local_x = pos.x - rect.min.x;
                let local_y = pos.y - rect.min.y;

                let col = (local_x / frame_w) as u32;
                let row = (local_y / frame_h) as u32;

                if col < cols && row < rows {
                    hovered_idx = Some(state.sprite_data.grid_to_frame(col, row));
                }
            }
            state.hovered_frame = hovered_idx;

            // Draw frame grid overlay
            for row in 0..rows {
                for col in 0..cols {
                    let frame_idx = state.sprite_data.grid_to_frame(col, row);
                    let x = rect.min.x + col as f32 * frame_w;
                    let y = rect.min.y + row as f32 * frame_h;
                    let frame_rect =
                        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(frame_w, frame_h));

                    // Highlight hovered frame
                    let is_hovered = hovered_idx == Some(frame_idx);
                    if is_hovered {
                        painter.rect_filled(
                            frame_rect,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(255, 200, 100, 80),
                        );
                    }

                    // Draw grid lines
                    painter.rect_stroke(
                        frame_rect,
                        0.0,
                        egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgba_unmultiplied(200, 200, 200, 150),
                        ),
                        egui::StrokeKind::Middle,
                    );

                    // Draw frame number
                    let text_color = if is_hovered {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 200)
                    };
                    painter.text(
                        egui::pos2(x + 4.0, y + 4.0),
                        egui::Align2::LEFT_TOP,
                        format!("{}", frame_idx),
                        egui::FontId::proportional(10.0 * state.zoom.max(0.5)),
                        text_color,
                    );
                }
            }
        });
}
