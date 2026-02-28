# Automap Rule Editor — UX Specification

**Author:** Counselor Deanna Troi, UX Designer
**Sprint:** Automapping
**Status:** COMPLETE — ready for SE implementation
**Task:** T-01

---

## 1. Overview and Design Intent

The Automap Rule Editor gives the user a way to define, inspect, and run rule-based automapping operations: given a painted tile map, apply pattern-matching rules that replace matched regions with specified output tiles.

The mental model the user brings: "I have a set of rules. Each rule describes a pattern I want to match on the map, and what I want placed when a match is found. I want to group related rules, run them, and undo the result if I don't like it."

The editor must make the rule structure — rule sets containing rules, rules containing input and output patterns — immediately visible. The user should never have to open a dialog to understand what a rule does.

---

## 2. Panel Placement and How It Is Opened

### Entry Points

**Three entry points, all triggering the same behavior:**

1. **Menu bar:** `Tools > Automap Rule Editor...`
   - Sets `editor_state.show_automap_editor = true`
   - Mirrors how the Tileset Editor is opened: `Tools > Tileset Editor...` sets `editor_state.show_tileset_editor = true`
   - No PendingAction dispatch needed — direct flag toggle, same as tileset editor and dialogue editor

2. **Keyboard shortcut:** `Ctrl+Shift+A`
   - Handled in `commands/shortcuts.rs` — toggles `editor_state.show_automap_editor`

3. **Project menu (future):** Not specified in this sprint. [ESCALATE-01: See section 13]

### Window Behavior

The automap editor opens as a **free-floating resizable window**, same as the Tileset Editor (`egui::Window::new(...).open(&mut is_open).collapsible(true).resizable(true)`).

- Default size: `[900.0, 640.0]`
- Minimum size: `[700.0, 480.0]`
- Window title: "Automap Rule Editor"
- Closeable via title bar X button — sets `show_automap_editor = false`
- State persists for the session (closing and reopening restores the editor to the same rule set and selection)

---

## 3. Full Layout

The editor uses a **three-column layout** inside the window.

```
+------------------------------------------------------------------+
| Automap Rule Editor                                          [x] |
+------------------------------------------------------------------+
| [Run Rules]  [Auto on Draw: ON/OFF]       Level: [combo-------v] |
+------------------------------------------------------------------+
| RULE SETS        | RULES               | PATTERN EDITOR          |
| (col 1 ~180px)   | (col 2 ~220px)      | (col 3 fills remainder) |
+------------------+---------------------+--------------------------|
| [+ Add]          | [+ Add]  [Dup] [Del]| [Input] [Output] tabs   |
| ----------       | ------------------- | ----------------------- |
| > Terrain Fill   | > Fill Grass (sel)  | Rule: "Fill Grass"      |
|   Cave Rooms     |   Fill Rock         | Name: [___________]     |
|   Boss Spawns    |   Clear Edges       | [x] NoOverlappingOutput |
|                  |                     | ----------------------- |
|                  | Rule settings:      | Grid:  3x3  [-][+]      |
| [Settings]       |   edge: [Wrap    v] |  +---+---+---+          |
|                  |   apply: [Once   v] |  |   | ! |   |          |
|                  |                     |  +---+---+---+          |
|                  |                     |  |   | G |   |          |
|                  |                     |  +---+---+---+          |
|                  |                     |  |   |   |   |          |
|                  |                     |  +---+---+---+          |
|                  |                     |                         |
|                  |                     | Brush: [Tile--------v]  |
|                  |                     | Tile:  [combo-------v]  |
|                  |                     | [Clear Brush]           |
|                  |                     | ----------------------- |
|                  |                     | Outputs (alternatives): |
|                  |                     | [+ Add Alt]             |
|                  |                     | [Alt 1: 70%] [x]        |
|                  |                     |  +---+---+---+          |
|                  |                     |  |   | G |   |          |
|                  |                     |  ...                    |
|                  |                     | [Alt 2: 30%] [x]        |
|                  |                     |  ...                    |
+------------------+---------------------+-------------------------+
| Layer mapping: Input layer [combo---v]  Output layer [combo---v] |
+------------------------------------------------------------------+
```

### Column Widths

- Column 1 (Rule Sets): fixed 180px
- Column 2 (Rules): fixed 220px
- Column 3 (Pattern Editor): fills remaining horizontal space

Column boundaries are drawn with `ui.separator()` in a `ui.columns(3, |cols| {...})` layout or equivalent `ui.horizontal` with explicit width constraints.

[ESCALATE-02: egui's `columns()` API provides equal-width columns only. For fixed-width column 1 and 2, the SE must use nested panels or allocate widths manually with `ui.allocate_ui_with_layout`. Data should confirm the implementation approach before the SE begins. See section 13.]

### Empty States

**No rule sets exist:**
```
+------------------+
| RULE SETS        |
| [+ Add]          |
|                  |
| No rule sets.    |
| Click [+ Add]    |
| to create one.   |
+------------------+
```

**Rule set selected but no rules exist:**
```
+---------------------+
| RULES               |
| [+ Add]  [Dup] [Del]|
|                     |
| No rules in this    |
| rule set. Click     |
| [+ Add] to begin.   |
+---------------------+
```

**No rule selected:**
```
+-------------------------+
| PATTERN EDITOR          |
|                         |
| Select a rule to        |
| edit its pattern.       |
+-------------------------+
```

---

## 4. Rule Set Management

### Rule Set List (Column 1)

The rule set list is a scrollable vertical list of selectable labels, one per rule set. The currently selected rule set is highlighted (egui `selectable_label` with `selected = true`).

```
| > Terrain Fill    |   <- selected (highlighted)
|   Cave Rooms      |   <- not selected
|   Boss Spawns     |   <- not selected
```

Clicking a rule set selects it. Column 2 updates to show the rules belonging to that rule set. Column 3 clears (shows "Select a rule to edit its pattern.") unless a rule within the newly selected rule set was the previous selection — in that case, clear the pattern editor selection. Selection does not persist across rule sets.

**Add Rule Set button:** `[+ Add]` above the list.
- Adds a new rule set with name "New Rule Set" and default settings
- Selects the new rule set immediately
- The name field in the rule-set settings panel (at bottom of column 1 or inline) enters edit focus

**Rule Set Settings (below the list, visible when a rule set is selected):**

```
+------------------+
| [Settings]       |   <- collapsible section header, default open
| Name: [________] |
| Edge: [Wrap    v] |
| Apply:[Once    v] |
+------------------+
```

These settings apply to the selected rule set only.

- **Name:** Single-line text field. Label "Rule Set Name:" to the left for accessibility. Accessible as `ui.label("Rule Set Name:"); ui.text_edit_singleline(&mut name)`.
- **Edge handling:** ComboBox labeled "Edge Handling:". Options: "Wrap", "Ignore", "Fixed". Default: "Wrap".
  - Wrap: pattern matching wraps at map edges
  - Ignore: cells outside the map are treated as non-matching
  - Fixed: cells outside the map match Empty
- **Apply mode:** ComboBox labeled "Apply Mode:". Options: "Once", "Until Stable". Default: "Once".
  - Once: rules applied one pass over the map
  - Until Stable: rules applied repeatedly until no changes occur (capped at a reasonable iteration limit — [ESCALATE-03: SE must confirm cap value or expose it as a setting. See section 13.])

**No delete button for rule sets in column 1 header.** Delete is via right-click context menu on the rule set item:
- "Delete Rule Set" — with confirmation: a small inline confirmation prompt replaces the item text ("Delete 'Terrain Fill'? [Yes] [No]").

**Reordering rule sets:** Up/Down arrow buttons per rule set item. This is the final design — not a fallback. Drag-and-drop is explicitly deferred to a future sprint.

```
| [^] [v]  Terrain Fill (selected) |
| [^] [v]  Cave Rooms              |
| [^] [v]  Boss Spawns             |
```

Up button is disabled when the rule set is already first. Down button is disabled when the rule set is already last. Tooltips: "Move rule set up" / "Move rule set down".

---

## 5. Rule List with Reordering (Column 2)

### Rule List

A scrollable list of selectable labels, one per rule in the selected rule set. Clicking a rule selects it and populates column 3 with that rule's pattern editor.

```
| > Fill Grass (selected)  |   <- selected
|   Fill Rock              |
|   Clear Edges            |
```

**Header controls:**

```
| [+ Add]  [Dup]  [Del] |
```

- `[+ Add]`: Adds a new rule with name "New Rule" and an empty 3x3 input grid. Selects the new rule. The rule's name field receives focus in column 3.
- `[Dup]`: Duplicates the selected rule. The duplicate is inserted immediately after the original. The duplicate is selected. Disabled (greyed) when no rule is selected.
- `[Del]`: Deletes the selected rule. No confirmation prompt for single rules (they are cheap to recreate). If the deleted rule was the last rule, column 3 shows the empty state. Disabled when no rule is selected.

**Reordering rules within a rule set:** Up/Down arrow buttons per rule item. This is the final design — not a fallback. Drag-and-drop is explicitly deferred to a future sprint.

```
| [^] [v]  Fill Grass (selected) |
| [^] [v]  Fill Rock             |
```

Up button is disabled when the rule is already first. Down button is disabled when the rule is already last. Tooltips: "Move rule up" / "Move rule down".

---

## 6. Pattern Editor (Column 3) — Per-Rule Settings

Column 3 is divided into sections, scrollable vertically if content overflows.

### 6a. Rule Header

```
| Rule: "Fill Grass"          |
| Name: [___________________] |
| [x] No Overlapping Output   |
```

- "Rule:" is a static label showing the rule name (non-editable display, for orientation)
- "Name:" label + single-line text edit for the rule name. Accessible: label precedes the field in layout.
- "No Overlapping Output" checkbox: when checked, the rule engine skips positions where the output pattern would overlap with a previous rule output in this pass. Accessible label: "No Overlapping Output".

### 6b. Input Pattern Section

Tabs at the top of the content area switch between input and output pattern views:

```
| [Input Pattern]  [Output Patterns] |
```

These are `selectable_label` tabs mirroring the TilesetEditorTab pattern.

**Input Pattern tab content:**

```
| Grid: 3 x 3   [-] [+] (cols)  [-] [+] (rows) |
```

The grid can be resized from 1x1 to 9x9. Resize buttons are labeled:
- "Decrease columns", "Increase columns", "Decrease rows", "Increase rows" (accessible labels on the `[-]` and `[+]` buttons, distinguished by position and tooltip)

When the grid is resized, new cells default to "Ignore". Cells removed by shrinking are discarded without confirmation.

**The input pattern grid:**

```
  +-------+-------+-------+
  |  [?]  |  [!]  |  [?]  |     <- row 0
  +-------+-------+-------+
  |  [?]  |  [G]  |  [?]  |     <- row 1 (center)
  +-------+-------+-------+
  |  [?]  |  [?]  |  [?]  |     <- row 2
  +-------+-------+-------+
```

Each cell in the input pattern grid is a button that, when clicked with the currently active brush, paints that cell. Right-click on a cell opens a context menu with all brush options for that cell (shortcut for changing a single cell without switching the active brush).

**Cell visual representations:**

| Brush type | Display in cell |
|---|---|
| Ignore | `?` (dim text, no background) |
| Empty | `_` (underscore glyph, distinct style) |
| NonEmpty | `*` (asterisk, distinct style) |
| Tile (specific) | Tile preview thumbnail (if texture cache available) or tile ID number |
| NOT(Tile) | `!T` where T is tile ID or thumbnail, with a red strikethrough visual |
| Other | [ESCALATE-05: "Other" brush type — see section 13] |

The center cell of the input grid (the "origin" cell — where the rule is anchored on the map) is visually distinguished: a small dot or ring indicator in the corner of the cell. This is not a separate control — it is always the geometric center of an odd-dimension grid. For even-dimension grids, the origin is the top-left of the center 2x2 quadrant. [ESCALATE-06: Even-dimension grid origin convention — see section 13.]

**Brush palette (below the grid):**

```
| Brush: [Tile v]       |
| Tile:  [combo------v] |
| [Clear Brush]         |
```

- "Brush:" ComboBox. Options for input patterns: "Ignore", "Empty", "NonEmpty", "Tile", "NOT Tile". Label: "Brush Type:".
- "Tile:" ComboBox — visible only when Brush is "Tile" or "NOT Tile". Shows all tiles from all tilesets in the project. Label: "Tile:".
- "[Clear Brush]" button: resets the active brush to "Ignore". Accessible label: "Reset Brush to Ignore".

When "Tile" is selected as the brush type, clicking an input cell sets that cell to match exactly that tile. When "NOT Tile" is selected, clicking sets that cell to match any tile except the specified one.

**Painting behavior:**
- Single click: paints the clicked cell with the active brush
- Click-drag: paints all cells the pointer passes over while the mouse button is held (no drag-reorder conflict — drag reorder only applies to the list columns, not the grid)
- Right-click on a cell: context menu listing all brush type options. Selecting one from the context menu immediately applies that brush to just that cell without affecting the active brush selection.

---

## 7. Output Patterns with Probability Weights (Output Patterns tab)

The Output Patterns tab shows one or more output alternatives. Each alternative has:
- A probability weight (integer, 1-100)
- Its own grid of the same dimensions as the input pattern grid

```
| Outputs (alternatives):       |
| [+ Add Alternative]           |
| ---                           |
| [Alt 1]  Weight: [70  ] [x]  |
|  +-------+-------+-------+   |
|  |  [_]  |  [G]  |  [_]  |   |
|  +-------+-------+-------+   |
|  |  [_]  |  [G]  |  [_]  |   |
|  +-------+-------+-------+   |
|  |  [_]  |  [G]  |  [_]  |   |
|  +-------+-------+-------+   |
| ---                           |
| [Alt 2]  Weight: [30  ] [x]  |
|  +-------+-------+-------+   |
|  ...                          |
```

**Weight field:** `DragValue` widget, range 1-100. Label "Weight:" precedes the field. Accessible label: "Output weight for alternative N".

The actual probability of each alternative is the weight divided by the sum of all weights. The displayed value is the raw weight integer, not the normalized percentage. A computed label "(NNN%)" appears to the right of the weight field, updated in real time as weights change. This is display-only.

**`[+ Add Alternative]` button:** Appends a new output alternative with weight 50 and a grid of blank "Leave Unchanged" cells.

**`[x]` button per alternative:** Deletes that alternative. Disabled when only one alternative exists (a rule must have at least one output).

**Scroll behavior:** The output alternatives section is the primary source of vertical overflow. The entire column 3 content is in a `egui::ScrollArea::vertical()`.

**Output cell brush types:**

| Brush type | Display in cell |
|---|---|
| Leave Unchanged | `-` (dash, muted style) |
| Tile (specific) | Tile preview thumbnail or tile ID |

Output cells do not support Ignore / Empty / NonEmpty / NOT variants — those are input-only concepts. The output brush palette for the output grid:

```
| Brush: [Tile v]       |
| Tile:  [combo------v] |
| [Leave Unchanged]     |
```

"Brush:" ComboBox options for output: "Tile", "Leave Unchanged".
"[Leave Unchanged]" button: shortcut to set active brush to Leave Unchanged. Accessible label: "Set brush to Leave Unchanged".

The grid dimensions of output alternatives always match the input pattern grid. When the input grid is resized, all output grids are resized to match. New cells added by expansion default to "Leave Unchanged".

---

## 8. Layer Mapping UI

A persistent strip at the **bottom of the window**, always visible regardless of which tab is active in column 3.

```
+------------------------------------------------------------------+
| Layer mapping:  Input: [Tile Layer 1 v]   Output: [Tile Layer 1 v]
+------------------------------------------------------------------+
```

- "Layer mapping:" static label
- "Input:" label + ComboBox. Lists all tile layers in the currently selected level. Label accessible as "Input layer for automapping".
- "Output:" label + ComboBox. Same list. Label accessible as "Output layer for automapping".
- Input and output layers may be the same layer (in-place replacement).

When no level is selected, both combos show "(no level)" and are disabled.

---

## 9. Run Rules Trigger, Feedback, and Undo

### Toolbar at Top of Window

```
| [Run Rules]  [Auto on Draw: OFF v]  Level: [Level 1 v] |
```

This strip appears at the top of the window, above the three-column area, separated by a `ui.separator()`.

**`[Run Rules]` button:**
- Accessible label: "Run automap rules"
- Sets `editor_state.pending_action = Some(PendingAction::RunAutomapRules)` — this is the PendingAction dispatch. The button does not execute the operation directly.
- Disabled when: no level is selected, or no rule sets exist, or no rules exist in any rule set
- When disabled, a tooltip explains why: "No level selected", "No rule sets defined", or "No rules defined"

**Level selector:**
- ComboBox showing all levels in the project. Label "Level:" precedes it. Accessible label: "Target level for automap".
- Defaults to the currently selected level in the editor (`editor_state.selected_level`)
- The selection is stored in `AutomapEditorState` (not EditorState, since it is editor-tool-local state)

**Feedback after Run:**

After `RunAutomapRules` is processed:
- A status label at the bottom of the top strip (or inline after the button) displays: "Applied N changes" where N is the number of tiles modified. This is temporary — it clears on the next run or when the editor window is closed.
- If no changes were made: "No changes — no rules matched."
- If an error occurred: the existing error dialog mechanism is used (`editor_state.error_message = Some(...)`)

The status label is a plain `ui.label(...)` and must not use color as the only indicator of success/failure. Use text that is self-describing. [See accessibility section 12.]

**Undo:**
- `RunAutomapRules` is recorded as a single undoable command via `AutomapCommand` pushed to `CommandHistory`
- The undo description (returned by `Command::description()`) is: "Automap: [rule set name]" or "Automap: [N] rule sets" if multiple are run
- Standard Ctrl+Z undoes it

---

## 10. Auto on Draw Behavior

The `[Auto on Draw]` toggle in the top strip controls whether automap rules run automatically whenever the user paints tiles on the target level.

```
| [Auto on Draw: OFF v] |   <- toggle rendered as a selectable label or toggle button
```

Behavior:
- When OFF (default): rules only run when the user clicks "Run Rules"
- When ON: after every tile paint operation that completes on the target level, the automap rules are run automatically. This is a fire-and-forget operation: it does not block the UI. The status strip shows "Auto-applied N changes" after each auto-run.

Implementation note for SE: Auto on Draw hooks into the paint completion event (after `BatchTileCommand` is pushed to history). The SE must confirm with Data where this hook lives. [ESCALATE-07: See section 13.]

The toggle is stored in `AutomapEditorState::auto_on_draw: bool`, defaulting to `false`.

The button displays current state in its label: "Auto on Draw: OFF" when false, "Auto on Draw: ON" when true. The button is a `selectable_label(auto_on_draw, text)` so it has a visually distinct selected state. Do not use color alone to communicate state — the text itself must say ON or OFF.

---

## 11. Per-Rule-Set Settings (detailed)

Already partially covered in section 4. Complete reference:

| Setting | Widget | Options | Default | Accessible label |
|---|---|---|---|---|
| Name | text_edit_singleline | any string | "New Rule Set" | "Rule Set Name:" (label preceding field) |
| Edge Handling | ComboBox | "Wrap", "Ignore", "Fixed" | "Wrap" | "Edge Handling:" (label preceding combo) |
| Apply Mode | ComboBox | "Once", "Until Stable" | "Once" | "Apply Mode:" (label preceding combo) |

These settings are in a collapsible section ("Rule Set Settings") at the bottom of column 1 when a rule set is selected. Default state: expanded.

---

## 12. Accessibility Specification

### Accessible Labels — Mandatory Requirements

Every interactive widget must have an accessible label that egui_kittest can discover via the AccessKit tree. The SE must follow these patterns:

**Text fields:** Always preceded by a `ui.label("...")` in a `ui.horizontal` layout, or use `.labelled_by()` on the response. A text field without an adjacent label is a test-blocking defect.

**Buttons:** The button text is the accessible label. Button text must be self-describing. Do not use icon-only buttons without a tooltip. The only icon-adjacent labels allowed: `[^]`, `[v]` for reorder (tooltip: "Move rule up" / "Move rule down"), `[x]` for delete (tooltip: "Delete").

**ComboBoxes:** Use `egui::ComboBox::from_label("...")` rather than `from_id_salt(...)`. The label argument becomes the accessible label. Every combo must have a distinct, meaningful label.

**Checkboxes:** The checkbox label argument is the accessible label. "No Overlapping Output" is sufficient.

**Grid cells:** Each cell in the input/output grids must have an accessible label of the form "Input cell row N col M" or "Output cell row N col M alt K". The SE must pass a `ui.button("").sense(...)` or equivalent with an accessible name via `egui::Button::new(...).sense(...)` and `.on_hover_text(...)`, or use `Response::labelled_by()`. This is required for Worf's grid interaction tests.

### Tab Order

Within column 3 (pattern editor), the logical tab order is:
1. Rule name field
2. No Overlapping Output checkbox
3. Input/Output tab selector
4. Grid cells (left-to-right, top-to-bottom)
5. Brush type combo
6. Tile selector combo (when visible)
7. Clear/reset brush button
8. (Output tab only) Add Alternative button
9. (Output tab only) Per-alternative: weight field, delete button, then grid cells

egui handles tab order by render order in immediate mode. The SE must render widgets in the above sequence within column 3.

### Keyboard Navigation

- `Tab` / `Shift+Tab`: moves focus between widgets in render order (egui default behavior)
- `Ctrl+Shift+A`: opens/closes the automap editor (from any context in the editor)
- `Ctrl+Z` / `Ctrl+Y`: undo/redo (handled globally, works when automap editor is open)
- When a grid cell has focus: `Arrow keys` move focus to adjacent cells. `Space` or `Enter` paints the focused cell with the current brush.
- When a list item (rule set or rule) has focus: `Arrow keys` move through the list. `Delete` key triggers the delete action (same as clicking `[Del]` button).

Arrow key navigation within the grid requires the SE to track focused cell state in `AutomapEditorState` and respond to key presses via `ui.input(|i| i.key_pressed(...))` inside the grid rendering loop.

### Color-Only Communication

No information may be conveyed only through color. Specific requirements:

- The status label ("Applied N changes", "No changes") uses text, not color alone
- Error states show text alongside any color change
- The "Auto on Draw: ON/OFF" toggle uses text, not just a colored indicator
- Disabled buttons use egui's built-in `add_enabled(false, widget)` which communicates disabled state via AccessKit automatically — do not implement a custom disabled appearance that loses this

### Tooltips

All `[x]`, `[^]`, `[v]`, `[-]`, `[+]` buttons must have `.on_hover_text(...)`:
- `[x]` on alternatives: "Delete this output alternative"
- `[x]` on rules (if icon used): "Delete rule"
- `[^]` on rules: "Move rule up"
- `[v]` on rules: "Move rule down"
- `[-]` on grid cols: "Decrease grid columns"
- `[+]` on grid cols: "Increase grid columns"
- `[-]` on grid rows: "Decrease grid rows"
- `[+]` on grid rows: "Increase grid rows"

---

## 13. Escalation Items [ESCALATE]

The following items require user decisions or Data/SE confirmation before the SE can begin implementing the affected areas. Each is marked in the spec above. Lead should surface these one at a time.

---

**[ESCALATE-01] — Rule Editor: Auto-Run on Tile Draw hook point**

The spec calls for "Auto on Draw" to fire after every tile paint. Where in the system does this hook? The tile paint path ends with a `BatchTileCommand` pushed to `CommandHistory`. Either:
- (a) The `process_edit_actions` system checks `auto_on_draw` after processing `BatchTileCommand` and immediately queues a `RunAutomapRules` action, or
- (b) The AutomapCommand is run directly inside the `BatchTileCommand::execute()` path (this would be wrong — command nesting violates the undo semantics)
- (c) A separate Bevy system observes CommandHistory changes and triggers automap when flagged

Option (a) is cleanest. Data must confirm before SE implements Auto on Draw. This is not blocking for the basic editor — SE can ship the editor without Auto on Draw and implement it in a follow-up task.

**Priority: non-blocking for initial implementation. Auto on Draw may be deferred.**

---

**[ESCALATE-02] — Three-column fixed-width layout in egui**

egui's `ui.columns(N, ...)` provides equal-width columns only. Fixed-width column 1 (180px) and column 2 (220px) require manual width allocation. The SE must confirm the implementation approach:
- Option A: `ui.allocate_ui_with_layout` with explicit width for each column, placed in a `ui.horizontal` block
- Option B: Use `egui::SidePanel::left` and `egui::SidePanel::right` inside the window (this may interact awkwardly with window-level panels)
- Option C: Use `egui::Frame` with `min_rect` constraints

Data must confirm the approach. If none of A/B/C is clean in egui 0.33, the fallback is equal-width columns (three equal columns within the window). This would change the visual proportions but is functionally acceptable.

**Priority: Data must answer before SE begins column layout. Blocking.**

---

**[ESCALATE-03] — "Until Stable" apply mode: iteration cap**

The spec specifies "Until Stable" as an apply mode option. This mode runs rules repeatedly until no tiles change. Without a cap, it could loop forever on certain rule configurations. Should the cap be:
- (a) Hard-coded (e.g., 100 iterations) — simple, not user-visible
- (b) User-configurable per rule set (adds a "Max iterations:" field)
- (c) Not included in the initial sprint — start with "Once" only

**Priority: If (c) is acceptable, the SE removes "Until Stable" from the ComboBox in the initial implementation. This is the recommendation to reduce scope. Lead should ask the user.**

---

**[ESCALATE-04] — CLOSED. Decision made by Troi.**

Reordering mechanism is **Up/Down arrow buttons** for both rule sets and rules. This is the final design, not a fallback.

Rationale: no existing drag-and-drop list pattern exists in this editor. Introducing drag-and-drop here would require the SE to pioneer a gesture interaction with no reference implementation, require Worf to test pointer-held gesture behavior with no existing test framework for it, and would create an interaction conflict between click-to-select and drag-to-reorder in a scrollable list. Up/Down buttons are self-describing, keyboard-accessible, and consistent with the editor's existing interaction vocabulary.

Drag-and-drop reordering is deferred to a future sprint and requires a separate UX spec at that time.

**No action needed. SE implements Up/Down buttons per section 4 and 5.**

---

**[ESCALATE-05] — "Other" input brush type**

The spec lists "Other" as a possible brush type for input cells. The meaning would be: match any tile that is not in the current tileset (e.g., a tile from a different tileset than the one configured for the layer). This is unclear and may not map to anything in the current data model.

**Recommendation: omit "Other" from the initial implementation. Input brush types are: Ignore, Empty, NonEmpty, Tile, NOT(Tile). "Other" is deferred.**

**Priority: Non-blocking. SE should omit Other unless the user requests it.**

---

**[ESCALATE-06] — Even-dimension grid origin convention**

The spec says grids can be 1x1 to 9x9. For odd dimensions, the center cell is unambiguous. For even dimensions (2x2, 4x4, etc.), the origin is ambiguous.

**Recommendation: restrict grid dimensions to odd numbers only (1, 3, 5, 7, 9). The SE should make the increment buttons skip even sizes. This avoids the ambiguity entirely.**

**Priority: Non-blocking design decision. SE should implement odd-only unless the user requests even support.**

---

**[ESCALATE-07] — AutomapCommand and data model location**

The spec assumes `AutomapCommand` exists and `project.automap_config.rule_sets` is the data path. Worf's test plan (testing.md) lists these as open questions. Data must confirm:

1. Where do `RuleSet`, `Rule`, and related types live? (Which crate, which file?)
2. Does `project.automap_config` exist as a named field on `Project`, and what is its type?
3. Is `AutomapCommand::execute()` a pure function over `&mut Project`?
4. What is the `AutomapCommand` constructor signature?

**Priority: Blocking for SE. The SE cannot write `automap_editor.rs` without knowing the data types. Data must answer these before SE begins.**

---

## 14. New PendingAction Variant Required

The SE must add the following variant to `PendingAction` in `crates/bevy_map_editor/src/ui/dialogs.rs`:

```rust
/// Run the automap rule sets against the current level
RunAutomapRules,
```

And process it in `process_edit_actions()` in `ui/mod.rs`.

The `show_automap_editor` flag must be added to `EditorState` in `crates/bevy_map_editor/src/lib.rs`, matching the pattern of `show_tileset_editor`, `show_dialogue_editor`, etc.

The menu item must be added to `ui/menu_bar.rs` in the Tools menu, after the existing "Schema Editor..." item:

```rust
ui.separator();
if ui.button("Automap Rule Editor...").clicked() {
    editor_state.show_automap_editor = true;
    ui.close();
}
```

---

## 15. New State Types Required

The SE must introduce `AutomapEditorState` (analogous to `TilesetEditorState`) with at minimum these fields:

```rust
pub struct AutomapEditorState {
    /// Currently selected rule set index
    pub selected_rule_set: Option<usize>,
    /// Currently selected rule index within the selected rule set
    pub selected_rule: Option<usize>,
    /// Active tab in column 3: Input or Output
    pub active_tab: AutomapEditorTab,
    /// Active brush type for input pattern painting
    pub input_brush: InputBrushType,
    /// Active brush type for output pattern painting
    pub output_brush: OutputBrushType,
    /// Selected tile ID for Tile and NOT(Tile) brushes
    pub brush_tile_id: Option<u32>,
    /// Target level for Run Rules
    pub target_level: Option<LevelId>,
    /// Auto on Draw toggle
    pub auto_on_draw: bool,
    /// Status message from last Run Rules operation
    pub last_run_status: Option<String>,
    /// Currently active output alternative index for editing
    pub selected_output_alt: Option<usize>,
}
```

Where:

```rust
#[derive(Default, PartialEq)]
pub enum AutomapEditorTab {
    #[default]
    InputPattern,
    OutputPatterns,
}

#[derive(Default, PartialEq)]
pub enum InputBrushType {
    #[default]
    Ignore,
    Empty,
    NonEmpty,
    Tile,
    NotTile,
}

#[derive(Default, PartialEq)]
pub enum OutputBrushType {
    #[default]
    LeaveUnchanged,
    Tile,
}
```

`AutomapEditorState` should be a field on `EditorState`, not a separate resource. Pattern: `pub automap_editor_state: AutomapEditorState`.

---

## 16. Spec Status and Open Questions Summary

| Item | Status |
|---|---|
| Panel placement and entry points | FINAL |
| Three-column layout | FINAL, pending ESCALATE-02 (column widths) |
| Rule set management | FINAL — Up/Down buttons, drag-and-drop deferred |
| Rule list with reordering | FINAL — Up/Down buttons, drag-and-drop deferred |
| Input pattern grid | FINAL, pending ESCALATE-05 (Other brush), ESCALATE-06 (even dims) |
| Output alternatives | FINAL |
| Layer mapping | FINAL |
| Per-rule settings | FINAL |
| Per-rule-set settings | FINAL, pending ESCALATE-03 (Until Stable cap) |
| Run Rules / feedback / undo | FINAL |
| Auto on Draw | FINAL, pending ESCALATE-01 (hook point; may be deferred) |
| Accessibility | FINAL |
| Data model (types/crate) | BLOCKED on ESCALATE-07 |

---

## 17. Checkpoint (Session End)

**Current state:** Spec is complete. All 17 sections written. Seven [ESCALATE] items identified.

**Next action for SE:** Read this spec, then escalate ESCALATE-07 to Data (data model location and types) before writing any code. Do not begin implementation until Data resolves ESCALATE-07.

**Next action for Worf:** This spec resolves the T-01 blocker. Worf may now begin writing UI interaction tests for the automap editor once the SE has produced an initial implementation reviewed by Data. Worf should reference section 12 (accessibility) for widget label strings. Exact label strings from the SE implementation must be confirmed before writing `get_by_label()` calls.

**Blockers outstanding:**
- ESCALATE-02: column layout approach (Data must answer, blocking SE)
- ESCALATE-03: Until Stable cap (user or Data decision, non-blocking if deferred)
- ESCALATE-04: CLOSED — Up/Down buttons, decided by Troi
- ESCALATE-05: Other brush (omit for now, non-blocking)
- ESCALATE-06: even grid dimensions (restrict to odd, non-blocking)
- ESCALATE-07: data model location (Data must answer, BLOCKING)
