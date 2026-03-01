//! Automap Rule Editor window
//!
//! Three-column layout:
//!   Col 1 (~180px): Rule set list + settings
//!   Col 2 (~220px): Rule list for selected rule set
//!   Col 3 (remainder): Pattern editor (input/output grids, brush palette)
//!
//! Entry points: `Tools > Automap Rule Editor...` menu item or `Ctrl+Shift+A`.
//! Opening sets `editor_state.show_automap_editor = true`; the window's own
//! close button sets it back to false via the `open` flag passed to egui.
//!
//! Per-spec: Troi's automap_ux_spec.md sections 2–15.

use bevy_egui::egui::{self};
use uuid::Uuid;

use bevy_map_automap::{
    ApplyMode, CellMatcher, CellOutput, EdgeHandling, InputConditionGroup, OutputAlternative, Rule,
    RuleSet, RuleSetSettings,
};

use crate::project::Project;
use crate::EditorState;

// ─── Column layout constants ──────────────────────────────────────────────────

const COL1_WIDTH: f32 = 180.0;
const COL2_WIDTH: f32 = 220.0;

// ─── State types ─────────────────────────────────────────────────────────────

/// Which tab is active in the pattern editor (column 3).
///
/// Mirrors the TilesetEditorTab pattern from tileset_editor.rs.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AutomapEditorTab {
    #[default]
    InputPattern,
    OutputPatterns,
}

/// Active brush type for input pattern cells.
///
/// "Other" is deferred per ESCALATE-05 in Troi's spec.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InputBrushType {
    /// Always matches — used to ignore a cell in the pattern.
    #[default]
    Ignore,
    /// Matches only an empty cell.
    Empty,
    /// Matches any non-empty cell.
    NonEmpty,
    /// Matches a specific tile ID.
    Tile,
    /// Matches anything except a specific tile ID.
    NotTile,
}

/// Active brush type for output pattern cells.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum OutputBrushType {
    /// Leave the cell unchanged when the rule fires.
    #[default]
    LeaveUnchanged,
    /// Write a specific tile to this cell.
    Tile,
}

/// Editor state for the Automap Rule Editor window.
///
/// Lives on `EditorState` (not as a separate Bevy resource), matching the
/// pattern used by `TilesetEditorState` and `DialogueEditorState`.
pub struct AutomapEditorState {
    /// Index into `project.automap_config.rule_sets`.
    pub selected_rule_set: Option<usize>,
    /// Index into the selected rule set's `rules` vec.
    pub selected_rule: Option<usize>,
    /// Active tab in column 3.
    pub active_tab: AutomapEditorTab,
    /// Active brush type when painting input pattern cells.
    pub input_brush: InputBrushType,
    /// Active brush type when painting output pattern cells.
    pub output_brush: OutputBrushType,
    /// Tile ID used when the brush type is `Tile` or `NotTile` (input) or `Tile` (output).
    pub brush_tile_id: Option<u32>,
    /// Target level UUID for "Run Rules". Defaults to the editor's selected level.
    pub target_level: Option<Uuid>,
    /// When true, automap runs automatically after every tile paint operation.
    ///
    /// Hook point confirmed non-blocking per ESCALATE-01 — implemented as
    /// a UI-only toggle here; the actual auto-run hook is a separate task.
    pub auto_on_draw: bool,
    /// Status message from the most recent "Run Rules" operation.
    pub last_run_status: Option<String>,
    /// Which output alternative is selected for editing in the Output tab.
    pub selected_output_alt: Option<usize>,
    /// Rule set pending inline delete confirmation (index, confirmed?).
    pub pending_delete_rule_set: Option<usize>,
    /// Rule pending inline delete confirmation (index).
    ///
    /// Rules are deleted without confirmation per Troi's spec (section 5).
    pub pending_delete_rule: Option<usize>,
    /// Whether the Rule Set Settings collapsible section is expanded.
    pub rule_set_settings_open: bool,
    /// Layer UUID selected in the "Input" layer combo in the layer mapping strip.
    ///
    /// Defaults to `None`; set to the first available layer when a level is selected.
    /// Used as the `layer_id` when creating new `InputConditionGroup`s.
    pub selected_input_layer: Option<Uuid>,
    /// Layer UUID selected in the "Output" layer combo in the layer mapping strip.
    ///
    /// Defaults to `None`; set to the first available layer when a level is selected.
    /// Used as the `layer_id` when creating new `OutputAlternative`s.
    pub selected_output_layer: Option<Uuid>,
}

impl Default for AutomapEditorState {
    fn default() -> Self {
        Self {
            selected_rule_set: None,
            selected_rule: None,
            active_tab: AutomapEditorTab::InputPattern,
            input_brush: InputBrushType::Ignore,
            output_brush: OutputBrushType::LeaveUnchanged,
            brush_tile_id: None,
            target_level: None,
            auto_on_draw: false,
            last_run_status: None,
            selected_output_alt: None,
            pending_delete_rule_set: None,
            pending_delete_rule: None,
            rule_set_settings_open: true,
            selected_input_layer: None,
            selected_output_layer: None,
        }
    }
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Render the Automap Rule Editor window.
///
/// Called from `render_ui` in `ui/mod.rs` when `editor_state.show_automap_editor` is true.
/// The `open` flag is passed directly to `egui::Window::open` so the title-bar X button
/// sets `show_automap_editor = false` automatically.
pub fn render_automap_editor(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    // Grab `show_automap_editor` as a local so we can pass `&mut bool` to egui.
    let mut is_open = editor_state.show_automap_editor;

    egui::Window::new("Automap Rule Editor")
        .open(&mut is_open)
        .default_size([900.0, 640.0])
        .min_size([700.0, 480.0])
        .collapsible(true)
        .resizable(true)
        .show(ctx, |ui| {
            render_top_strip(ui, editor_state, project);
            ui.separator();
            render_three_columns(ui, editor_state, project);
            ui.separator();
            render_layer_mapping_strip(ui, editor_state, project);
        });

    editor_state.show_automap_editor = is_open;
}

// ─── Top strip ───────────────────────────────────────────────────────────────

/// Toolbar strip at the top of the window (above the three columns).
///
/// Contains: [Run Rules]  [Auto on Draw: ON/OFF]  Level: [combo]  status label
fn render_top_strip(ui: &mut egui::Ui, editor_state: &mut EditorState, project: &mut Project) {
    ui.horizontal(|ui| {
        let state = &mut editor_state.automap_editor_state;
        let config = &project.automap_config;

        // Determine disabled reason for the Run Rules button.
        let run_disabled_reason = if state.target_level.is_none() {
            Some("No level selected")
        } else if config.rule_sets.is_empty() {
            Some("No rule sets defined")
        } else if config.rule_sets.iter().all(|rs| rs.rules.is_empty()) {
            Some("No rules defined")
        } else {
            None
        };

        let run_enabled = run_disabled_reason.is_none();

        let run_btn = ui.add_enabled(
            run_enabled,
            egui::Button::new("Run Rules"),
        );
        if run_btn.clicked() {
            editor_state.pending_action =
                Some(crate::ui::PendingAction::RunAutomapRules);
        }
        if let Some(reason) = run_disabled_reason {
            run_btn.on_disabled_hover_text(reason);
        }

        ui.add_space(8.0);

        // Auto on Draw toggle — text communicates state explicitly (no color-only signaling).
        let auto_label = if editor_state.automap_editor_state.auto_on_draw {
            "Auto on Draw: ON"
        } else {
            "Auto on Draw: OFF"
        };
        let auto_on = editor_state.automap_editor_state.auto_on_draw;
        if ui.selectable_label(auto_on, auto_label).clicked() {
            editor_state.automap_editor_state.auto_on_draw = !auto_on;
        }

        ui.add_space(8.0);

        // Level selector combo.
        ui.label("Level:");
        let level_names: Vec<(Uuid, String)> = project
            .levels
            .iter()
            .map(|l| (l.id, l.name.clone()))
            .collect();

        let current_level_name = editor_state
            .automap_editor_state
            .target_level
            .and_then(|id| level_names.iter().find(|(lid, _)| *lid == id))
            .map(|(_, name)| name.as_str())
            .unwrap_or("(none)");

        egui::ComboBox::from_label("Target level for automap")
            .selected_text(current_level_name)
            .show_ui(ui, |ui| {
                for (id, name) in &level_names {
                    let selected = editor_state.automap_editor_state.target_level == Some(*id);
                    if ui.selectable_label(selected, name).clicked() {
                        editor_state.automap_editor_state.target_level = Some(*id);
                    }
                }
            });

        ui.add_space(8.0);

        // Status label from last Run Rules operation.
        if let Some(ref status) = editor_state.automap_editor_state.last_run_status.clone() {
            ui.label(status);
        }
    });
}

// ─── Three-column layout ──────────────────────────────────────────────────────

/// Three-column layout using `ui.horizontal` with explicit width allocation.
///
/// Per ESCALATE-02 resolution: option A — `ui.allocate_ui_with_layout` with
/// explicit width for columns 1 and 2; column 3 fills the remainder.
fn render_three_columns(ui: &mut egui::Ui, editor_state: &mut EditorState, project: &mut Project) {
    ui.horizontal(|ui| {
        // Column 1: Rule Sets (~180px fixed)
        ui.allocate_ui_with_layout(
            egui::vec2(COL1_WIDTH, ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                render_rule_set_column(ui, editor_state, project);
            },
        );

        ui.separator();

        // Column 2: Rules (~220px fixed)
        ui.allocate_ui_with_layout(
            egui::vec2(COL2_WIDTH, ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                render_rule_column(ui, editor_state, project);
            },
        );

        ui.separator();

        // Column 3: Pattern Editor (fills remaining width)
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            render_pattern_editor_column(ui, editor_state, project);
        });
    });
}

// ─── Column 1: Rule Sets ──────────────────────────────────────────────────────

fn render_rule_set_column(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    ui.strong("RULE SETS");

    // Add button
    if ui.button("+ Add").clicked() {
        let new_idx = project.automap_config.rule_sets.len();
        project.automap_config.rule_sets.push(RuleSet {
            id: Uuid::new_v4(),
            name: "New Rule Set".to_string(),
            rules: Vec::new(),
            settings: RuleSetSettings::default(),
            disabled: false,
        });
        editor_state.automap_editor_state.selected_rule_set = Some(new_idx);
        // Clear rule selection when rule set changes.
        editor_state.automap_editor_state.selected_rule = None;
        project.mark_dirty();
    }

    ui.separator();

    let rule_set_count = project.automap_config.rule_sets.len();

    if rule_set_count == 0 {
        ui.label("No rule sets.");
        ui.label("Click [+ Add]");
        ui.label("to create one.");
        return;
    }

    // Scrollable rule set list
    egui::ScrollArea::vertical()
        .id_salt("rule_set_list")
        .max_height(ui.available_height() - 120.0) // leave room for settings
        .show(ui, |ui| {
            let mut move_up: Option<usize> = None;
            let mut move_down: Option<usize> = None;
            let mut confirm_delete: Option<usize> = None;
            let mut cancel_delete: Option<usize> = None;

            for idx in 0..rule_set_count {
                let name = project.automap_config.rule_sets[idx].name.clone();
                let selected = editor_state.automap_editor_state.selected_rule_set == Some(idx);

                // Pending delete confirmation for this item?
                if editor_state.automap_editor_state.pending_delete_rule_set == Some(idx) {
                    ui.horizontal(|ui| {
                        ui.label(format!("Delete '{}'?", name));
                        if ui.button("Yes").clicked() {
                            confirm_delete = Some(idx);
                        }
                        if ui.button("No").clicked() {
                            cancel_delete = Some(idx);
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        // Up/Down reorder buttons per Troi spec section 4.
                        ui.add_enabled_ui(idx > 0, |ui| {
                            if ui
                                .button("^")
                                .on_hover_text("Move rule set up")
                                .clicked()
                            {
                                move_up = Some(idx);
                            }
                        });
                        ui.add_enabled_ui(idx < rule_set_count - 1, |ui| {
                            if ui
                                .button("v")
                                .on_hover_text("Move rule set down")
                                .clicked()
                            {
                                move_down = Some(idx);
                            }
                        });

                        // Selectable label for the rule set name.
                        // Capture the response to attach the context menu directly (per Troi spec section 4).
                        let response = ui.selectable_label(selected, &name);
                        if response.clicked() && !selected {
                            editor_state.automap_editor_state.selected_rule_set = Some(idx);
                            // Clear rule selection when switching rule sets.
                            editor_state.automap_editor_state.selected_rule = None;
                        }
                        response.context_menu(|ui| {
                            if ui.button("Delete Rule Set").clicked() {
                                editor_state.automap_editor_state.pending_delete_rule_set =
                                    Some(idx);
                                ui.close();
                            }
                        });
                    });
                }
            }

            // Apply deferred mutations after the loop (borrow checker).
            if let Some(idx) = move_up {
                project.automap_config.rule_sets.swap(idx - 1, idx);
                if editor_state.automap_editor_state.selected_rule_set == Some(idx) {
                    editor_state.automap_editor_state.selected_rule_set = Some(idx - 1);
                }
                project.mark_dirty();
            }
            if let Some(idx) = move_down {
                project.automap_config.rule_sets.swap(idx, idx + 1);
                if editor_state.automap_editor_state.selected_rule_set == Some(idx) {
                    editor_state.automap_editor_state.selected_rule_set = Some(idx + 1);
                }
                project.mark_dirty();
            }
            if let Some(idx) = confirm_delete {
                project.automap_config.rule_sets.remove(idx);
                // Fix selection index after removal.
                let new_sel = if project.automap_config.rule_sets.is_empty() {
                    None
                } else {
                    Some(idx.saturating_sub(1))
                };
                editor_state.automap_editor_state.selected_rule_set = new_sel;
                editor_state.automap_editor_state.selected_rule = None;
                editor_state.automap_editor_state.pending_delete_rule_set = None;
                project.mark_dirty();
            }
            if let Some(_idx) = cancel_delete {
                editor_state.automap_editor_state.pending_delete_rule_set = None;
            }
        });

    // Rule Set Settings (collapsible, only when a rule set is selected)
    if let Some(sel_idx) = editor_state.automap_editor_state.selected_rule_set {
        if sel_idx < project.automap_config.rule_sets.len() {
            ui.separator();
            let open = &mut editor_state.automap_editor_state.rule_set_settings_open;
            egui::CollapsingHeader::new("Rule Set Settings")
                .default_open(true)
                .open(Some(*open))
                .show(ui, |ui| {
                    *open = true; // keep synced
                    let rule_set = &mut project.automap_config.rule_sets[sel_idx];

                    // Name field — label precedes field for accessibility.
                    ui.horizontal(|ui| {
                        ui.label("Rule Set Name:");
                        ui.text_edit_singleline(&mut rule_set.name);
                    });

                    // Edge Handling combo.
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_label("Edge Handling:")
                            .selected_text(edge_handling_label(rule_set.settings.edge_handling))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut rule_set.settings.edge_handling,
                                    EdgeHandling::Skip,
                                    "Ignore",
                                );
                                ui.selectable_value(
                                    &mut rule_set.settings.edge_handling,
                                    EdgeHandling::TreatAsEmpty,
                                    "Fixed",
                                );
                            });
                    });

                    // Apply Mode combo.
                    // ESCALATE-03: "Until Stable" is included with the hard-coded cap of 100
                    // iterations (see bevy_map_automap::UNTIL_STABLE_MAX_ITERATIONS).
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_label("Apply Mode:")
                            .selected_text(apply_mode_label(rule_set.settings.apply_mode))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut rule_set.settings.apply_mode,
                                    ApplyMode::Once,
                                    "Once",
                                );
                                ui.selectable_value(
                                    &mut rule_set.settings.apply_mode,
                                    ApplyMode::UntilStable,
                                    "Until Stable",
                                );
                            });
                    });
                });
        }
    }
}

// ─── Column 2: Rules ──────────────────────────────────────────────────────────

fn render_rule_column(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    ui.strong("RULES");

    let Some(rs_idx) = editor_state.automap_editor_state.selected_rule_set else {
        ui.label("Select a rule set.");
        return;
    };

    if rs_idx >= project.automap_config.rule_sets.len() {
        editor_state.automap_editor_state.selected_rule_set = None;
        return;
    }

    let rule_count = project.automap_config.rule_sets[rs_idx].rules.len();
    let selected_rule = editor_state.automap_editor_state.selected_rule;
    let has_rule_selected = selected_rule.is_some();

    // Header: Add / Dup / Del buttons.
    ui.horizontal(|ui| {
        if ui.button("+ Add").clicked() {
            let input_layer_id = editor_state.automap_editor_state.selected_input_layer.unwrap_or(Uuid::nil());
            let output_layer_id = editor_state.automap_editor_state.selected_output_layer.unwrap_or(Uuid::nil());
            let new_rule = make_default_rule(input_layer_id, output_layer_id);
            let new_idx = project.automap_config.rule_sets[rs_idx].rules.len();
            project.automap_config.rule_sets[rs_idx].rules.push(new_rule);
            editor_state.automap_editor_state.selected_rule = Some(new_idx);
            project.mark_dirty();
        }

        ui.add_enabled_ui(has_rule_selected, |ui| {
            if ui.button("Dup").clicked() {
                if let Some(sel) = selected_rule {
                    if sel < project.automap_config.rule_sets[rs_idx].rules.len() {
                        let mut cloned = project.automap_config.rule_sets[rs_idx].rules[sel].clone();
                        // New UUID to keep the duplicate distinct.
                        cloned.id = Uuid::new_v4();
                        let insert_at = sel + 1;
                        project.automap_config.rule_sets[rs_idx]
                            .rules
                            .insert(insert_at, cloned);
                        editor_state.automap_editor_state.selected_rule = Some(insert_at);
                        project.mark_dirty();
                    }
                }
            }
        });

        ui.add_enabled_ui(has_rule_selected, |ui| {
            if ui.button("Del").on_hover_text("Delete rule").clicked() {
                if let Some(sel) = selected_rule {
                    if sel < project.automap_config.rule_sets[rs_idx].rules.len() {
                        project.automap_config.rule_sets[rs_idx].rules.remove(sel);
                        let new_sel = if project.automap_config.rule_sets[rs_idx].rules.is_empty() {
                            None
                        } else {
                            Some(sel.saturating_sub(1))
                        };
                        editor_state.automap_editor_state.selected_rule = new_sel;
                        project.mark_dirty();
                    }
                }
            }
        });
    });

    ui.separator();

    if rule_count == 0 {
        ui.label("No rules in this");
        ui.label("rule set. Click");
        ui.label("[+ Add] to begin.");
        return;
    }

    // Scrollable rule list.
    egui::ScrollArea::vertical()
        .id_salt("rule_list")
        .show(ui, |ui| {
            let mut move_up: Option<usize> = None;
            let mut move_down: Option<usize> = None;

            for idx in 0..rule_count {
                let name = project.automap_config.rule_sets[rs_idx].rules[idx].name.clone();
                let selected = editor_state.automap_editor_state.selected_rule == Some(idx);

                ui.horizontal(|ui| {
                    ui.add_enabled_ui(idx > 0, |ui| {
                        if ui.button("^").on_hover_text("Move rule up").clicked() {
                            move_up = Some(idx);
                        }
                    });
                    ui.add_enabled_ui(idx < rule_count - 1, |ui| {
                        if ui.button("v").on_hover_text("Move rule down").clicked() {
                            move_down = Some(idx);
                        }
                    });

                    if ui.selectable_label(selected, &name).clicked() {
                        editor_state.automap_editor_state.selected_rule = Some(idx);
                    }
                });
            }

            if let Some(idx) = move_up {
                project.automap_config.rule_sets[rs_idx].rules.swap(idx - 1, idx);
                if editor_state.automap_editor_state.selected_rule == Some(idx) {
                    editor_state.automap_editor_state.selected_rule = Some(idx - 1);
                }
                project.mark_dirty();
            }
            if let Some(idx) = move_down {
                project.automap_config.rule_sets[rs_idx].rules.swap(idx, idx + 1);
                if editor_state.automap_editor_state.selected_rule == Some(idx) {
                    editor_state.automap_editor_state.selected_rule = Some(idx + 1);
                }
                project.mark_dirty();
            }
        });
}

// ─── Column 3: Pattern Editor ────────────────────────────────────────────────

fn render_pattern_editor_column(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    ui.strong("PATTERN EDITOR");

    let Some(rs_idx) = editor_state.automap_editor_state.selected_rule_set else {
        ui.label("Select a rule to");
        ui.label("edit its pattern.");
        return;
    };

    let Some(r_idx) = editor_state.automap_editor_state.selected_rule else {
        ui.label("Select a rule to");
        ui.label("edit its pattern.");
        return;
    };

    if rs_idx >= project.automap_config.rule_sets.len()
        || r_idx >= project.automap_config.rule_sets[rs_idx].rules.len()
    {
        editor_state.automap_editor_state.selected_rule = None;
        return;
    }

    // Rule header: name display + editable name field + no-overlap checkbox.
    // Clone the name for display before borrowing project mutably.
    let rule_display_name = project.automap_config.rule_sets[rs_idx].rules[r_idx].name.clone();
    ui.label(format!("Rule: \"{}\"", rule_display_name));

    ui.horizontal(|ui| {
        ui.label("Name:");
        // Re-borrow rule mutably — this is inside a separate statement from the label above.
        let rule = &mut project.automap_config.rule_sets[rs_idx].rules[r_idx];
        ui.text_edit_singleline(&mut rule.name);
    });

    {
        let rule = &mut project.automap_config.rule_sets[rs_idx].rules[r_idx];
        ui.checkbox(&mut rule.no_overlapping_output, "No Overlapping Output");
    }

    ui.separator();

    // Tab selector — Input Pattern / Output Patterns.
    ui.horizontal(|ui| {
        let active_tab = editor_state.automap_editor_state.active_tab;
        if ui
            .selectable_label(active_tab == AutomapEditorTab::InputPattern, "Input Pattern")
            .clicked()
        {
            editor_state.automap_editor_state.active_tab = AutomapEditorTab::InputPattern;
        }
        if ui
            .selectable_label(
                active_tab == AutomapEditorTab::OutputPatterns,
                "Output Patterns",
            )
            .clicked()
        {
            editor_state.automap_editor_state.active_tab = AutomapEditorTab::OutputPatterns;
        }
    });

    ui.separator();

    // Scrollable content area for the rest of column 3.
    egui::ScrollArea::vertical()
        .id_salt("pattern_editor_scroll")
        .show(ui, |ui| {
            match editor_state.automap_editor_state.active_tab {
                AutomapEditorTab::InputPattern => {
                    render_input_pattern_tab(ui, editor_state, project, rs_idx, r_idx);
                }
                AutomapEditorTab::OutputPatterns => {
                    render_output_patterns_tab(ui, editor_state, project, rs_idx, r_idx);
                }
            }
        });
}

// ─── Input Pattern Tab ────────────────────────────────────────────────────────

fn render_input_pattern_tab(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &mut Project,
    rs_idx: usize,
    r_idx: usize,
) {
    // Grid size controls.
    // The input group is the first one on the rule; we create it if missing.
    let input_layer_id = editor_state.automap_editor_state.selected_input_layer.unwrap_or(Uuid::nil());
    ensure_input_group(project, rs_idx, r_idx, input_layer_id);

    let (half_w, half_h) = {
        let group = &project.automap_config.rule_sets[rs_idx].rules[r_idx].input_groups[0];
        (group.half_width, group.half_height)
    };
    let cols = 2 * half_w + 1;
    let rows = 2 * half_h + 1;

    // Grid dimension controls.
    // ESCALATE-06 resolution: only odd dimensions (1,3,5,7,9) — increment skips even sizes.
    ui.horizontal(|ui| {
        ui.label(format!("Grid: {} x {}", cols, rows));
        ui.add_space(8.0);

        // Column resize buttons.
        ui.add_enabled_ui(half_w > 0, |ui| {
            if ui
                .button("-")
                .on_hover_text("Decrease grid columns")
                .clicked()
            {
                let group = &mut project.automap_config.rule_sets[rs_idx].rules[r_idx]
                    .input_groups[0];
                resize_input_group_cols(group, group.half_width.saturating_sub(1));
                sync_output_grid_dims(project, rs_idx, r_idx);
                project.mark_dirty();
            }
        });
        ui.add_enabled_ui(half_w < 4, |ui| {
            // max half_w = 4 → full width = 9
            if ui
                .button("+")
                .on_hover_text("Increase grid columns")
                .clicked()
            {
                let group = &mut project.automap_config.rule_sets[rs_idx].rules[r_idx]
                    .input_groups[0];
                let new_hw = (group.half_width + 1).min(4);
                resize_input_group_cols(group, new_hw);
                sync_output_grid_dims(project, rs_idx, r_idx);
                project.mark_dirty();
            }
        });

        ui.label("cols");
        ui.add_space(8.0);

        // Row resize buttons.
        ui.add_enabled_ui(half_h > 0, |ui| {
            if ui
                .button("-")
                .on_hover_text("Decrease grid rows")
                .clicked()
            {
                let group = &mut project.automap_config.rule_sets[rs_idx].rules[r_idx]
                    .input_groups[0];
                resize_input_group_rows(group, group.half_height.saturating_sub(1));
                sync_output_grid_dims(project, rs_idx, r_idx);
                project.mark_dirty();
            }
        });
        ui.add_enabled_ui(half_h < 4, |ui| {
            if ui
                .button("+")
                .on_hover_text("Increase grid rows")
                .clicked()
            {
                let group = &mut project.automap_config.rule_sets[rs_idx].rules[r_idx]
                    .input_groups[0];
                let new_hh = (group.half_height + 1).min(4);
                resize_input_group_rows(group, new_hh);
                sync_output_grid_dims(project, rs_idx, r_idx);
                project.mark_dirty();
            }
        });

        ui.label("rows");
    });

    // Input grid.
    let center_col = half_w as usize;
    let center_row = half_h as usize;

    // Clone matchers before the loop so we can borrow project immutably for reading
    // and mutably for writing back — avoiding the double-borrow through egui closures.
    let matchers_snapshot: Vec<CellMatcher> = project.automap_config.rule_sets[rs_idx].rules[r_idx]
        .input_groups[0]
        .matchers
        .clone();

    let mut new_matchers = matchers_snapshot.clone();
    let mut grid_changed = false;

    for row in 0..rows as usize {
        ui.horizontal(|ui| {
            for col in 0..cols as usize {
                let cell_idx = row * cols as usize + col;
                let matcher = &matchers_snapshot[cell_idx];
                let is_center = row == center_row && col == center_col;
                let label = input_cell_label(matcher);
                let accessible_name = format!("Input cell row {} col {}", row, col);

                // Center cell gets a distinct visual — we add a frame around it.
                let response = if is_center {
                    egui::Frame::default()
                        .stroke(egui::Stroke::new(2.0, egui::Color32::YELLOW))
                        .show(ui, |ui| ui.button(&label))
                        .inner
                } else {
                    ui.button(&label)
                };

                // on_hover_text consumes self and returns self, so we reassign.
                let response = response.on_hover_text(&accessible_name);

                if response.clicked() {
                    new_matchers[cell_idx] = brush_to_matcher(
                        editor_state.automap_editor_state.input_brush,
                        editor_state.automap_editor_state.brush_tile_id,
                    );
                    grid_changed = true;
                }

                // Right-click context menu — apply a single brush without changing active brush.
                response.context_menu(|ui| {
                    if ui.button("Ignore").clicked() {
                        new_matchers[cell_idx] = CellMatcher::Ignore;
                        grid_changed = true;
                        ui.close();
                    }
                    if ui.button("Empty").clicked() {
                        new_matchers[cell_idx] = CellMatcher::Empty;
                        grid_changed = true;
                        ui.close();
                    }
                    if ui.button("NonEmpty").clicked() {
                        new_matchers[cell_idx] = CellMatcher::NonEmpty;
                        grid_changed = true;
                        ui.close();
                    }
                });
            }
        });
    }

    if grid_changed {
        project.automap_config.rule_sets[rs_idx].rules[r_idx].input_groups[0].matchers =
            new_matchers;
        project.mark_dirty();
    }

    ui.separator();

    // Input brush palette.
    render_input_brush_palette(ui, editor_state, project);
}

/// Render the brush palette for the input pattern tab.
fn render_input_brush_palette(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &Project,
) {
    let state = &mut editor_state.automap_editor_state;

    ui.horizontal(|ui| {
        egui::ComboBox::from_label("Brush Type:")
            .selected_text(input_brush_label(state.input_brush))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.input_brush, InputBrushType::Ignore, "Ignore");
                ui.selectable_value(&mut state.input_brush, InputBrushType::Empty, "Empty");
                ui.selectable_value(
                    &mut state.input_brush,
                    InputBrushType::NonEmpty,
                    "NonEmpty",
                );
                ui.selectable_value(&mut state.input_brush, InputBrushType::Tile, "Tile");
                ui.selectable_value(&mut state.input_brush, InputBrushType::NotTile, "NOT Tile");
            });
    });

    // Tile selector — only visible when brush is Tile or NotTile.
    if matches!(state.input_brush, InputBrushType::Tile | InputBrushType::NotTile) {
        // Collect all (tileset_name, tile_id) pairs for the combo.
        let tile_options: Vec<(String, u32)> = project
            .tilesets
            .iter()
            .flat_map(|ts| {
                let total_tiles: u32 = ts.images.iter().map(|img| img.columns * img.rows).sum();
                (0..total_tiles).map(move |id| (format!("{} #{}", ts.name, id), id))
            })
            .collect();

        let current_tile_name = state
            .brush_tile_id
            .and_then(|id| tile_options.iter().find(|(_, tid)| *tid == id))
            .map(|(name, _)| name.as_str())
            .unwrap_or("(none)");

        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Tile:")
                .selected_text(current_tile_name)
                .show_ui(ui, |ui| {
                    for (name, id) in &tile_options {
                        let selected = state.brush_tile_id == Some(*id);
                        if ui.selectable_label(selected, name).clicked() {
                            state.brush_tile_id = Some(*id);
                        }
                    }
                });
        });
    }

    if ui.button("Reset Brush to Ignore").clicked() {
        state.input_brush = InputBrushType::Ignore;
        state.brush_tile_id = None;
    }
}

// ─── Output Patterns Tab ──────────────────────────────────────────────────────

fn render_output_patterns_tab(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &mut Project,
    rs_idx: usize,
    r_idx: usize,
) {
    let output_layer_id = editor_state.automap_editor_state.selected_output_layer.unwrap_or(Uuid::nil());

    // Ensure at least one output alternative exists.
    if project.automap_config.rule_sets[rs_idx].rules[r_idx]
        .output_alternatives
        .is_empty()
    {
        let (hw, hh) = get_input_half_dims(project, rs_idx, r_idx);
        let cell_count = ((2 * hw + 1) * (2 * hh + 1)) as usize;
        project.automap_config.rule_sets[rs_idx].rules[r_idx]
            .output_alternatives
            .push(make_default_output_alt(hw, hh, cell_count, output_layer_id));
        project.mark_dirty();
    }

    ui.label("Outputs (alternatives):");

    if ui.button("+ Add Alternative").clicked() {
        let (hw, hh) = get_input_half_dims(project, rs_idx, r_idx);
        let cell_count = ((2 * hw + 1) * (2 * hh + 1)) as usize;
        project.automap_config.rule_sets[rs_idx].rules[r_idx]
            .output_alternatives
            .push(make_default_output_alt(hw, hh, cell_count, output_layer_id));
        project.mark_dirty();
    }

    ui.separator();

    let alt_count = project.automap_config.rule_sets[rs_idx].rules[r_idx]
        .output_alternatives
        .len();

    // Compute total weight for percentage display.
    let total_weight: u32 = project.automap_config.rule_sets[rs_idx].rules[r_idx]
        .output_alternatives
        .iter()
        .map(|a| a.weight)
        .sum::<u32>()
        .max(1); // avoid divide-by-zero

    let mut delete_alt: Option<usize> = None;
    let mut grid_changes: Vec<(usize, usize, CellOutput)> = Vec::new();

    for alt_idx in 0..alt_count {
        // Snapshot weight and outputs for rendering.
        let weight = project.automap_config.rule_sets[rs_idx].rules[r_idx].output_alternatives
            [alt_idx]
            .weight;
        let pct = (weight as f32 / total_weight as f32 * 100.0).round() as u32;
        let outputs_snapshot: Vec<CellOutput> = project.automap_config.rule_sets[rs_idx].rules
            [r_idx]
            .output_alternatives[alt_idx]
            .outputs
            .clone();
        let hw = project.automap_config.rule_sets[rs_idx].rules[r_idx].output_alternatives
            [alt_idx]
            .half_width;
        let hh = project.automap_config.rule_sets[rs_idx].rules[r_idx].output_alternatives
            [alt_idx]
            .half_height;

        ui.horizontal(|ui| {
            ui.strong(format!("Alt {}", alt_idx + 1));

            ui.label("Weight:");
            let mut w = weight;
            if ui
                .add(egui::DragValue::new(&mut w).range(1..=100))
                .on_hover_text(format!(
                    "Output weight for alternative {}",
                    alt_idx + 1
                ))
                .changed()
            {
                project.automap_config.rule_sets[rs_idx].rules[r_idx].output_alternatives
                    [alt_idx]
                    .weight = w;
                project.mark_dirty();
            }

            ui.label(format!("({}%)", pct));

            // Delete button — disabled when this is the only alternative.
            ui.add_enabled_ui(alt_count > 1, |ui| {
                if ui
                    .button("x")
                    .on_hover_text("Delete this output alternative")
                    .clicked()
                {
                    delete_alt = Some(alt_idx);
                }
            });
        });

        // Output grid.
        let cols = 2 * hw + 1;
        let rows = 2 * hh + 1;
        let mut local_changes: Vec<(usize, CellOutput)> = Vec::new();

        for row in 0..rows as usize {
            ui.horizontal(|ui| {
                for col in 0..cols as usize {
                    let cell_idx = row * cols as usize + col;
                    let output = &outputs_snapshot[cell_idx];
                    let label = output_cell_label(output);
                    let accessible_name =
                        format!("Output cell row {} col {} alt {}", row, col, alt_idx + 1);

                    let response = ui.button(&label).on_hover_text(&accessible_name);

                    if response.clicked() {
                        let new_output = output_brush_to_cell_output(
                            editor_state.automap_editor_state.output_brush,
                            editor_state.automap_editor_state.brush_tile_id,
                        );
                        local_changes.push((cell_idx, new_output));
                    }

                    response.context_menu(|ui| {
                        if ui.button("Leave Unchanged").clicked() {
                            local_changes.push((cell_idx, CellOutput::Ignore));
                            ui.close();
                        }
                        if ui.button("Empty").clicked() {
                            local_changes.push((cell_idx, CellOutput::Empty));
                            ui.close();
                        }
                    });
                }
            });
        }

        for (cell_idx, new_output) in local_changes {
            grid_changes.push((alt_idx, cell_idx, new_output));
        }

        ui.separator();
    }

    // Apply deferred grid changes.
    for (alt_idx, cell_idx, new_output) in grid_changes {
        if let Some(cell) = project.automap_config.rule_sets[rs_idx].rules[r_idx]
            .output_alternatives[alt_idx]
            .outputs
            .get_mut(cell_idx)
        {
            *cell = new_output;
            project.mark_dirty();
        }
    }

    if let Some(idx) = delete_alt {
        project.automap_config.rule_sets[rs_idx].rules[r_idx]
            .output_alternatives
            .remove(idx);
        project.mark_dirty();
    }

    ui.separator();

    // Output brush palette.
    render_output_brush_palette(ui, editor_state, project);
}

/// Render the brush palette for the output pattern tab.
fn render_output_brush_palette(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &Project,
) {
    let state = &mut editor_state.automap_editor_state;

    ui.horizontal(|ui| {
        egui::ComboBox::from_label("Brush:")
            .selected_text(output_brush_label(state.output_brush))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut state.output_brush,
                    OutputBrushType::Tile,
                    "Tile",
                );
                ui.selectable_value(
                    &mut state.output_brush,
                    OutputBrushType::LeaveUnchanged,
                    "Leave Unchanged",
                );
            });
    });

    // Tile selector — only visible when brush is Tile.
    if state.output_brush == OutputBrushType::Tile {
        let tile_options: Vec<(String, u32)> = project
            .tilesets
            .iter()
            .flat_map(|ts| {
                let total_tiles: u32 = ts.images.iter().map(|img| img.columns * img.rows).sum();
                (0..total_tiles).map(move |id| (format!("{} #{}", ts.name, id), id))
            })
            .collect();

        let current_tile_name = state
            .brush_tile_id
            .and_then(|id| tile_options.iter().find(|(_, tid)| *tid == id))
            .map(|(name, _)| name.as_str())
            .unwrap_or("(none)");

        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Tile: ")
                .selected_text(current_tile_name)
                .show_ui(ui, |ui| {
                    for (name, id) in &tile_options {
                        let selected = state.brush_tile_id == Some(*id);
                        if ui.selectable_label(selected, name).clicked() {
                            state.brush_tile_id = Some(*id);
                        }
                    }
                });
        });
    }

    if ui.button("Set brush to Leave Unchanged").clicked() {
        state.output_brush = OutputBrushType::LeaveUnchanged;
    }
}

// ─── Bottom strip: Layer Mapping ──────────────────────────────────────────────

/// Persistent layer mapping strip at the bottom of the window.
///
/// Always visible regardless of which tab is active in column 3.
fn render_layer_mapping_strip(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    ui.horizontal(|ui| {
        ui.strong("Layer mapping:");

        let target_level_id = editor_state.automap_editor_state.target_level;

        // Collect layers for the selected level.
        let layers: Vec<(Uuid, String)> = target_level_id
            .and_then(|lid| project.levels.iter().find(|l| l.id == lid))
            .map(|level| {
                level
                    .layers
                    .iter()
                    .map(|layer| (layer.id, layer.name.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let has_level = target_level_id.is_some() && !layers.is_empty();

        // Resolve display names for the currently selected layers.
        let current_input_id = editor_state.automap_editor_state.selected_input_layer;
        let current_output_id = editor_state.automap_editor_state.selected_output_layer;

        let input_name = current_input_id
            .and_then(|id| layers.iter().find(|(lid, _)| *lid == id).map(|(_, n)| n.as_str()))
            .unwrap_or("(none)");
        let output_name = current_output_id
            .and_then(|id| layers.iter().find(|(lid, _)| *lid == id).map(|(_, n)| n.as_str()))
            .unwrap_or("(none)");

        // Collect pending selections into locals, then apply after closures to satisfy borrow checker.
        let mut pending_input: Option<Uuid> = None;
        let mut pending_output: Option<Uuid> = None;

        // Input layer combo.
        ui.add_enabled_ui(has_level, |ui| {
            egui::ComboBox::from_label("Input layer for automapping")
                .selected_text(input_name)
                .show_ui(ui, |ui| {
                    for (id, name) in &layers {
                        let selected = current_input_id == Some(*id);
                        if ui.selectable_label(selected, name).clicked() {
                            pending_input = Some(*id);
                        }
                    }
                });
        });

        ui.add_space(16.0);

        // Output layer combo.
        ui.add_enabled_ui(has_level, |ui| {
            egui::ComboBox::from_label("Output layer for automapping")
                .selected_text(output_name)
                .show_ui(ui, |ui| {
                    for (id, name) in &layers {
                        let selected = current_output_id == Some(*id);
                        if ui.selectable_label(selected, name).clicked() {
                            pending_output = Some(*id);
                        }
                    }
                });
        });

        // Apply pending selections now that all egui closures are done.
        if let Some(id) = pending_input {
            editor_state.automap_editor_state.selected_input_layer = Some(id);
        }
        if let Some(id) = pending_output {
            editor_state.automap_editor_state.selected_output_layer = Some(id);
        }
    });
}

// ─── Helper functions ─────────────────────────────────────────────────────────

/// Create a default rule with a 3x3 input group (half_width=1, half_height=1)
/// and one empty output alternative.
///
/// `input_layer_id` and `output_layer_id` are the currently selected layer UUIDs
/// from `AutomapEditorState`. Pass `Uuid::nil()` when no layer is selected.
fn make_default_rule(input_layer_id: Uuid, output_layer_id: Uuid) -> Rule {
    let hw: u32 = 1;
    let hh: u32 = 1;
    let cell_count = ((2 * hw + 1) * (2 * hh + 1)) as usize;
    Rule {
        id: Uuid::new_v4(),
        name: "New Rule".to_string(),
        input_groups: vec![InputConditionGroup {
            layer_id: input_layer_id,
            half_width: hw,
            half_height: hh,
            matchers: vec![CellMatcher::Ignore; cell_count],
        }],
        output_alternatives: vec![make_default_output_alt(hw, hh, cell_count, output_layer_id)],
        no_overlapping_output: false,
    }
}

/// `layer_id` is the currently selected output layer UUID from `AutomapEditorState`.
/// Pass `Uuid::nil()` when no layer is selected.
fn make_default_output_alt(hw: u32, hh: u32, cell_count: usize, layer_id: Uuid) -> OutputAlternative {
    OutputAlternative {
        id: Uuid::new_v4(),
        layer_id,
        half_width: hw,
        half_height: hh,
        outputs: vec![CellOutput::Ignore; cell_count],
        weight: 1,
    }
}

/// Ensure the rule has at least one `InputConditionGroup` (creates a 3x3 default if missing).
///
/// `layer_id` is the currently selected input layer UUID from `AutomapEditorState`.
/// Pass `Uuid::nil()` when no layer is selected.
fn ensure_input_group(project: &mut Project, rs_idx: usize, r_idx: usize, layer_id: Uuid) {
    if project.automap_config.rule_sets[rs_idx].rules[r_idx]
        .input_groups
        .is_empty()
    {
        let hw: u32 = 1;
        let hh: u32 = 1;
        let cell_count = ((2 * hw + 1) * (2 * hh + 1)) as usize;
        project.automap_config.rule_sets[rs_idx].rules[r_idx]
            .input_groups
            .push(InputConditionGroup {
                layer_id,
                half_width: hw,
                half_height: hh,
                matchers: vec![CellMatcher::Ignore; cell_count],
            });
    }
}

/// Get (half_width, half_height) from the rule's first input group, or (1,1) if none.
fn get_input_half_dims(project: &Project, rs_idx: usize, r_idx: usize) -> (u32, u32) {
    project.automap_config.rule_sets[rs_idx].rules[r_idx]
        .input_groups
        .first()
        .map(|g| (g.half_width, g.half_height))
        .unwrap_or((1, 1))
}

/// Resize the input group's column count (half_width), preserving existing matchers.
///
/// Shrinking truncates columns; expanding fills new cells with `CellMatcher::Ignore`.
fn resize_input_group_cols(group: &mut InputConditionGroup, new_half_w: u32) {
    let old_cols = 2 * group.half_width + 1;
    let new_cols = 2 * new_half_w + 1;
    let rows = 2 * group.half_height + 1;
    let mut new_matchers = Vec::with_capacity((new_cols * rows) as usize);
    for row in 0..rows {
        for col in 0..new_cols {
            if col < old_cols {
                let old_idx = (row * old_cols + col) as usize;
                new_matchers.push(group.matchers.get(old_idx).cloned().unwrap_or(CellMatcher::Ignore));
            } else {
                new_matchers.push(CellMatcher::Ignore);
            }
        }
    }
    group.half_width = new_half_w;
    group.matchers = new_matchers;
}

/// Resize the input group's row count (half_height), preserving existing matchers.
fn resize_input_group_rows(group: &mut InputConditionGroup, new_half_h: u32) {
    let cols = 2 * group.half_width + 1;
    let old_rows = 2 * group.half_height + 1;
    let new_rows = 2 * new_half_h + 1;
    let mut new_matchers = Vec::with_capacity((cols * new_rows) as usize);
    for row in 0..new_rows {
        for col in 0..cols {
            if row < old_rows {
                let old_idx = (row * cols + col) as usize;
                new_matchers.push(group.matchers.get(old_idx).cloned().unwrap_or(CellMatcher::Ignore));
            } else {
                new_matchers.push(CellMatcher::Ignore);
            }
        }
    }
    group.half_height = new_half_h;
    group.matchers = new_matchers;
}

/// Sync all output alternative grids to match the current input group dimensions.
///
/// Called after any input grid resize. New output cells default to `CellOutput::Ignore`.
fn sync_output_grid_dims(project: &mut Project, rs_idx: usize, r_idx: usize) {
    let (new_hw, new_hh) = get_input_half_dims(project, rs_idx, r_idx);
    let new_cols = 2 * new_hw + 1;
    let new_rows = 2 * new_hh + 1;
    let new_cell_count = (new_cols * new_rows) as usize;

    for alt in project.automap_config.rule_sets[rs_idx].rules[r_idx]
        .output_alternatives
        .iter_mut()
    {
        let old_cols = 2 * alt.half_width + 1;
        let old_rows = 2 * alt.half_height + 1;
        let mut new_outputs = Vec::with_capacity(new_cell_count);
        for row in 0..new_rows {
            for col in 0..new_cols {
                if row < old_rows && col < old_cols {
                    let old_idx = (row * old_cols + col) as usize;
                    new_outputs.push(
                        alt.outputs.get(old_idx).cloned().unwrap_or(CellOutput::Ignore),
                    );
                } else {
                    new_outputs.push(CellOutput::Ignore);
                }
            }
        }
        alt.half_width = new_hw;
        alt.half_height = new_hh;
        alt.outputs = new_outputs;
    }
}

/// Convert the active input brush + optional tile ID to a `CellMatcher`.
fn brush_to_matcher(brush: InputBrushType, tile_id: Option<u32>) -> CellMatcher {
    match brush {
        InputBrushType::Ignore => CellMatcher::Ignore,
        InputBrushType::Empty => CellMatcher::Empty,
        InputBrushType::NonEmpty => CellMatcher::NonEmpty,
        InputBrushType::Tile => CellMatcher::Tile(tile_id.unwrap_or(0)),
        InputBrushType::NotTile => CellMatcher::NotTile(tile_id.unwrap_or(0)),
    }
}

/// Convert the active output brush + optional tile ID to a `CellOutput`.
fn output_brush_to_cell_output(brush: OutputBrushType, tile_id: Option<u32>) -> CellOutput {
    match brush {
        OutputBrushType::LeaveUnchanged => CellOutput::Ignore,
        OutputBrushType::Tile => CellOutput::Tile(tile_id.unwrap_or(0)),
    }
}

/// Short display label for a `CellMatcher` in the input grid.
fn input_cell_label(matcher: &CellMatcher) -> String {
    match matcher {
        CellMatcher::Ignore => "?".to_string(),
        CellMatcher::Empty => "_".to_string(),
        CellMatcher::NonEmpty => "*".to_string(),
        CellMatcher::Tile(id) => format!("{}", id),
        CellMatcher::NotTile(id) => format!("!{}", id),
        CellMatcher::TileFlipped { id, .. } => format!("{}f", id),
        CellMatcher::Other => "~".to_string(),
    }
}

/// Short display label for a `CellOutput` in the output grid.
fn output_cell_label(output: &CellOutput) -> String {
    match output {
        CellOutput::Ignore => "-".to_string(),
        CellOutput::Empty => "_".to_string(),
        CellOutput::Tile(id) => format!("{}", id),
    }
}

fn edge_handling_label(e: EdgeHandling) -> &'static str {
    match e {
        EdgeHandling::Skip => "Ignore",
        EdgeHandling::TreatAsEmpty => "Fixed",
    }
}

fn apply_mode_label(m: ApplyMode) -> &'static str {
    match m {
        ApplyMode::Once => "Once",
        ApplyMode::UntilStable => "Until Stable",
    }
}

fn input_brush_label(b: InputBrushType) -> &'static str {
    match b {
        InputBrushType::Ignore => "Ignore",
        InputBrushType::Empty => "Empty",
        InputBrushType::NonEmpty => "NonEmpty",
        InputBrushType::Tile => "Tile",
        InputBrushType::NotTile => "NOT Tile",
    }
}

fn output_brush_label(b: OutputBrushType) -> &'static str {
    match b {
        OutputBrushType::LeaveUnchanged => "Leave Unchanged",
        OutputBrushType::Tile => "Tile",
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;
    use egui_kittest::Harness;

    use bevy_map_core::{Layer, Level};
    use uuid::Uuid;

    use crate::project::Project;
    use crate::testing::{harness_for_menu_bar, menu_bar_state_empty_project};
    use crate::EditorState;

    use super::render_automap_editor;

    // ── Bundle + harness builder ──────────────────────────────────────────────

    /// Bundle for `render_automap_editor` — mirrors `TilesetEditorBundle` from testing.rs.
    struct AutomapEditorBundle {
        editor_state: EditorState,
        project: Project,
    }

    /// Build a `Harness` that renders the automap editor window.
    ///
    /// The harness sets `show_automap_editor = true` before the first frame so
    /// the window is actually rendered. `cache` is always `None`.
    fn harness_for_automap_editor(state: AutomapEditorBundle) -> Harness<'static, AutomapEditorBundle> {
        Harness::new_state(
            |ctx, bundle: &mut AutomapEditorBundle| {
                render_automap_editor(ctx, &mut bundle.editor_state, &mut bundle.project);
            },
            state,
        )
    }

    /// Default bundle: empty project, `show_automap_editor = true`.
    fn automap_editor_default() -> AutomapEditorBundle {
        let mut editor_state = EditorState::default();
        editor_state.show_automap_editor = true;
        AutomapEditorBundle {
            editor_state,
            project: Project::default(),
        }
    }

    /// Bundle with a level that has one tile layer — used for layer combo label tests.
    fn automap_editor_with_level() -> AutomapEditorBundle {
        let mut editor_state = EditorState::default();
        editor_state.show_automap_editor = true;

        let mut project = Project::default();
        let mut level = Level::new("TestLevel".to_string(), 4, 4);
        let layer = Layer::new_tile_layer("Tiles".to_string(), Uuid::nil(), 4, 4);
        let layer_id = layer.id;
        let level_id = level.id;
        level.add_layer(layer);
        project.levels.push(level);

        // Point the editor state at this level and layer.
        editor_state.automap_editor_state.target_level = Some(level_id);
        editor_state.automap_editor_state.selected_input_layer = Some(layer_id);
        editor_state.automap_editor_state.selected_output_layer = Some(layer_id);

        AutomapEditorBundle { editor_state, project }
    }

    // ── Engine integration smoke test (render only) ───────────────────────────

    /// Smoke test: render with default project and `show_automap_editor = true`.
    /// Passes if no panic occurs during rendering.
    #[test]
    fn automap_editor_renders_without_panic_default_project() {
        let mut harness = harness_for_automap_editor(automap_editor_default());
        harness.run();
    }

    // ── "Add Rule Set" button present ─────────────────────────────────────────

    /// The "+ Add" button in column 1 must be discoverable in the AccessKit tree.
    #[test]
    fn automap_editor_add_rule_set_button_present() {
        let mut harness = harness_for_automap_editor(automap_editor_default());
        harness.run();
        // The button label in render_rule_sets_column is "+ Add"
        assert!(
            harness.query_by_label("+ Add").is_some(),
            "Expected '+ Add' button to be present in the AccessKit tree"
        );
    }

    // ── Clicking "Add Rule Set" increments rule_sets.len() ───────────────────

    /// Clicking the "+ Add" button must add one rule set (0 → 1).
    #[test]
    fn automap_editor_add_rule_set_button_adds_rule_set() {
        let mut harness = harness_for_automap_editor(automap_editor_default());
        // Precondition
        assert_eq!(harness.state().project.automap_config.rule_sets.len(), 0);
        harness.run();
        harness.get_by_label("+ Add").click();
        harness.run();
        assert_eq!(
            harness.state().project.automap_config.rule_sets.len(),
            1,
            "Clicking '+ Add' must add one rule set"
        );
    }

    // ── Layer combo labels present with level ─────────────────────────────────

    /// When a level with layers is selected, both combo labels must be in the AccessKit tree.
    #[test]
    fn automap_editor_layer_combo_labels_present_with_level() {
        let mut harness = harness_for_automap_editor(automap_editor_with_level());
        harness.run();
        assert!(
            harness.query_by_label("Input layer for automapping").is_some(),
            "Expected 'Input layer for automapping' label in AccessKit tree"
        );
        assert!(
            harness.query_by_label("Output layer for automapping").is_some(),
            "Expected 'Output layer for automapping' label in AccessKit tree"
        );
    }

    // ── Layer combo labels present without level ──────────────────────────────

    /// Even when no level is selected (combos disabled), both combo labels must still
    /// appear in the AccessKit tree (disabled widgets remain in the tree).
    #[test]
    fn automap_editor_layer_combo_labels_present_without_level() {
        let mut harness = harness_for_automap_editor(automap_editor_default());
        // Precondition: no target_level
        assert!(harness.state().editor_state.automap_editor_state.target_level.is_none());
        harness.run();
        assert!(
            harness.query_by_label("Input layer for automapping").is_some(),
            "Expected 'Input layer for automapping' label even when disabled"
        );
        assert!(
            harness.query_by_label("Output layer for automapping").is_some(),
            "Expected 'Output layer for automapping' label even when disabled"
        );
    }

    // ── Menu bar: Tools → Automap Rule Editor sets show_automap_editor ────────

    /// Clicking Tools → "Automap Rule Editor..." must set `show_automap_editor = true`.
    #[test]
    fn menu_bar_automap_rule_editor_sets_show_flag() {
        let mut state = menu_bar_state_empty_project();
        // Precondition: flag is off.
        state.editor_state.show_automap_editor = false;
        let mut harness = harness_for_menu_bar(state);
        harness.run();
        // Open the Tools menu.
        harness.get_by_label("Tools").click();
        harness.run();
        // Click the menu item.
        harness.get_by_label("Automap Rule Editor...").click();
        harness.run();
        assert!(
            harness.state().editor_state.show_automap_editor,
            "show_automap_editor must be true after clicking 'Automap Rule Editor...'"
        );
    }
}
