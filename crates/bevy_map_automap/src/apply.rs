//! The rule engine: applies an [`AutomapConfig`] to a [`Level`].
//!
//! The entry point is [`apply_automap_config`]. Everything below that is an
//! internal helper.

use std::collections::{HashMap, HashSet};

use bevy_map_core::{tile_flip_x, tile_flip_y, tile_index, LayerData, Level};
use rand::Rng;
use uuid::Uuid;

use crate::{
    ApplyMode, AutomapConfig, CellMatcher, CellOutput, EdgeHandling, InputConditionGroup,
    OutputAlternative, Rule, RuleSet, UNTIL_STABLE_MAX_ITERATIONS,
};

// ─── Public entry point ───────────────────────────────────────────────────────

/// Apply all enabled rule sets in `config` to `level`, in order.
///
/// `rng` is used for weighted random output selection. Call this before
/// recording an undo snapshot — the caller is responsible for diffing the
/// level before and after to build the [`AutomapCommand`](bevy_map_editor).
pub fn apply_automap_config(level: &mut Level, config: &AutomapConfig, rng: &mut impl Rng) {
    for rule_set in &config.rule_sets {
        if !rule_set.disabled {
            apply_ruleset(level, rule_set, rng);
        }
    }
}

// ─── RuleSet application ──────────────────────────────────────────────────────

/// Apply a single rule set to `level`.
fn apply_ruleset(level: &mut Level, rule_set: &RuleSet, rng: &mut impl Rng) {
    match rule_set.settings.apply_mode {
        ApplyMode::Once => {
            apply_ruleset_once(level, rule_set, rng);
        }
        ApplyMode::UntilStable => {
            // Build the set of layer indices referenced by this rule set.
            // We snapshot only those layers on each iteration to keep allocations minimal.
            let layer_indices = collect_referenced_layer_indices(level, rule_set);

            for iteration in 0..UNTIL_STABLE_MAX_ITERATIONS {
                let snapshot = snapshot_layers(level, &layer_indices);
                apply_ruleset_once(level, rule_set, rng);
                if layers_match_snapshot(level, &snapshot) {
                    // No cells changed — the map is stable.
                    let _ = iteration; // suppress unused warning in release
                    return;
                }
            }
            eprintln!(
                "[automap] rule set '{}' did not converge after {} iterations",
                rule_set.name, UNTIL_STABLE_MAX_ITERATIONS
            );
        }
    }
}

/// Collect the unique layer indices (within `level.layers`) that this rule set
/// references, for snapshot purposes. Unresolved UUIDs are silently skipped.
fn collect_referenced_layer_indices(level: &Level, rule_set: &RuleSet) -> Vec<usize> {
    let mut indices: Vec<usize> = Vec::new();
    for rule in &rule_set.rules {
        for group in &rule.input_groups {
            if let Some(idx) = find_layer_index(level, group.layer_id) {
                if !indices.contains(&idx) {
                    indices.push(idx);
                }
            }
        }
        for alt in &rule.output_alternatives {
            if let Some(idx) = find_layer_index(level, alt.layer_id) {
                if !indices.contains(&idx) {
                    indices.push(idx);
                }
            }
        }
    }
    indices
}

/// Snapshot the tile data for each layer index in `layer_indices`.
/// Returns a map from layer_index → flat row-major `Vec<Option<u32>>`.
fn snapshot_layers(level: &Level, layer_indices: &[usize]) -> HashMap<usize, Vec<Option<u32>>> {
    let mut snapshot = HashMap::with_capacity(layer_indices.len());
    for &layer_idx in layer_indices {
        if let Some(layer) = level.get_layer(layer_idx) {
            if let LayerData::Tiles { tiles, .. } = &layer.data {
                snapshot.insert(layer_idx, tiles.clone());
            }
        }
    }
    snapshot
}

/// Return `true` if all snapshotted layers still match their snapshot.
fn layers_match_snapshot(level: &Level, snapshot: &HashMap<usize, Vec<Option<u32>>>) -> bool {
    for (&layer_idx, old_tiles) in snapshot {
        if let Some(layer) = level.get_layer(layer_idx) {
            if let LayerData::Tiles { tiles, .. } = &layer.data {
                if tiles != old_tiles {
                    return false;
                }
            }
        }
    }
    true
}

// ─── Single pass ─────────────────────────────────────────────────────────────

/// Run one full pass of the rule set over the level.
fn apply_ruleset_once(level: &mut Level, rule_set: &RuleSet, rng: &mut impl Rng) {
    let edge = rule_set.settings.edge_handling;

    for rule in &rule_set.rules {
        apply_rule(level, rule, edge, rng);
    }
}

/// Apply one rule across every position in the level.
fn apply_rule(level: &mut Level, rule: &Rule, edge: EdgeHandling, rng: &mut impl Rng) {
    // Resolve all input group and output alternative layer UUIDs to indices up front.
    // Skip the entire rule if any reference is unresolvable — this avoids silent partial
    // application that could corrupt the level.
    let input_layer_indices: Vec<Option<usize>> = rule
        .input_groups
        .iter()
        .map(|g| find_layer_index(level, g.layer_id))
        .collect();

    if input_layer_indices.iter().any(|idx| idx.is_none()) {
        // At least one input group references a layer that doesn't exist.
        // Skip the rule silently — the orphan cleanup in validate_automap_config
        // handles this on project load; during a run it means the user deleted
        // a layer without triggering cleanup.
        return;
    }
    let input_layer_indices: Vec<usize> = input_layer_indices.into_iter().flatten().collect();

    let output_layer_indices: Vec<Option<usize>> = rule
        .output_alternatives
        .iter()
        .map(|a| find_layer_index(level, a.layer_id))
        .collect();

    if output_layer_indices.iter().any(|idx| idx.is_none()) {
        return;
    }
    let output_layer_indices: Vec<usize> = output_layer_indices.into_iter().flatten().collect();

    // Collect explicit tile indices across all matchers in this rule.
    // Used by the `Other` matcher to determine "not one of these".
    let explicit_tiles = collect_explicit_tiles(rule);

    // Track positions that have been written (for NoOverlappingOutput).
    // Key: (layer_index, x, y).
    let mut written: HashSet<(usize, u32, u32)> = HashSet::new();

    let width = level.width;
    let height = level.height;

    for y in 0..height {
        for x in 0..width {
            // NoOverlappingOutput: skip if the center cell of the output was already written.
            // We check against the first output alternative's layer and the center cell.
            if rule.no_overlapping_output {
                if let Some(&out_layer_idx) = output_layer_indices.first() {
                    if written.contains(&(out_layer_idx, x, y)) {
                        continue;
                    }
                }
            }

            // Check all input groups. All must match.
            let all_match = input_layer_indices
                .iter()
                .zip(rule.input_groups.iter())
                .all(|(&layer_idx, group)| {
                    group_matches(level, layer_idx, group, x, y, edge, &explicit_tiles)
                });

            if !all_match {
                continue;
            }

            // Select output alternative.
            let Some(alt_idx) = select_output_alternative(&rule.output_alternatives, rng) else {
                continue;
            };
            let alt = &rule.output_alternatives[alt_idx];
            let out_layer_idx = output_layer_indices[alt_idx];

            // Apply the output to the level.
            let grid_w = 2 * alt.half_width + 1;
            let grid_h = 2 * alt.half_height + 1;

            for row in 0..grid_h {
                for col in 0..grid_w {
                    let cell_x = x as i64 + col as i64 - alt.half_width as i64;
                    let cell_y = y as i64 + row as i64 - alt.half_height as i64;

                    // Skip out-of-bounds output cells.
                    if cell_x < 0
                        || cell_y < 0
                        || cell_x >= width as i64
                        || cell_y >= height as i64
                    {
                        continue;
                    }
                    let cell_x = cell_x as u32;
                    let cell_y = cell_y as u32;

                    let output_idx = (row * grid_w + col) as usize;
                    let Some(cell_output) = alt.outputs.get(output_idx) else {
                        continue;
                    };

                    match cell_output {
                        CellOutput::Ignore => {}
                        CellOutput::Empty => {
                            level.set_tile(out_layer_idx, cell_x, cell_y, None);
                            if rule.no_overlapping_output {
                                written.insert((out_layer_idx, cell_x, cell_y));
                            }
                        }
                        CellOutput::Tile(tile_value) => {
                            level.set_tile(out_layer_idx, cell_x, cell_y, Some(*tile_value));
                            if rule.no_overlapping_output {
                                written.insert((out_layer_idx, cell_x, cell_y));
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── Group and cell matching ──────────────────────────────────────────────────

/// Return `true` if all matchers in `group` match at position `(center_x, center_y)`.
fn group_matches(
    level: &Level,
    layer_idx: usize,
    group: &InputConditionGroup,
    center_x: u32,
    center_y: u32,
    edge: EdgeHandling,
    explicit_tiles: &HashSet<u32>,
) -> bool {
    let grid_w = 2 * group.half_width + 1;
    let grid_h = 2 * group.half_height + 1;

    for row in 0..grid_h {
        for col in 0..grid_w {
            let cell_x = center_x as i64 + col as i64 - group.half_width as i64;
            let cell_y = center_y as i64 + row as i64 - group.half_height as i64;

            let matcher_idx = (row * grid_w + col) as usize;
            let Some(matcher) = group.matchers.get(matcher_idx) else {
                // Malformed group — fewer matchers than grid cells. Skip.
                return false;
            };

            let oob = cell_x < 0
                || cell_y < 0
                || cell_x >= level.width as i64
                || cell_y >= level.height as i64;

            if oob {
                match edge {
                    EdgeHandling::Skip => return false,
                    EdgeHandling::TreatAsEmpty => {
                        // OOB treated as None. Only Ignore and Empty can match.
                        if !matcher_matches_cell(matcher, None, explicit_tiles) {
                            return false;
                        }
                    }
                }
            } else {
                let cell = level.get_tile(layer_idx, cell_x as u32, cell_y as u32);
                if !matcher_matches_cell(matcher, cell, explicit_tiles) {
                    return false;
                }
            }
        }
    }
    true
}

/// Test one matcher against one cell value.
///
/// `cell` is `None` for an empty cell or `Some(packed_tile)` for a filled cell,
/// where the packed tile encodes the tile index and flip flags per
/// `bevy_map_core::layer` conventions.
fn matcher_matches_cell(
    matcher: &CellMatcher,
    cell: Option<u32>,
    explicit_tiles: &HashSet<u32>,
) -> bool {
    match matcher {
        CellMatcher::Ignore => true,

        CellMatcher::Empty => cell.is_none(),

        CellMatcher::NonEmpty => cell.is_some(),

        CellMatcher::Tile(id) => {
            // Strip flip bits; compare base index only.
            matches!(cell, Some(v) if tile_index(v) == *id)
        }

        CellMatcher::NotTile(id) => {
            // Empty cell is "not tile X", so NotTile matches empty.
            match cell {
                None => true,
                Some(v) => tile_index(v) != *id,
            }
        }

        CellMatcher::TileFlipped {
            id,
            flip_x,
            flip_y,
        } => {
            // Both base index and exact flip state must match.
            matches!(
                cell,
                Some(v)
                    if tile_index(v) == *id
                        && tile_flip_x(v) == *flip_x
                        && tile_flip_y(v) == *flip_y
            )
        }

        CellMatcher::Other => {
            // Cell must contain a tile whose base index is not in explicit_tiles.
            matches!(cell, Some(v) if !explicit_tiles.contains(&tile_index(v)))
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Collect the set of explicit tile indices referenced by `Tile`, `NotTile`, and
/// `TileFlipped` matchers anywhere in this rule's input groups.
///
/// Used by the `Other` matcher: a cell matches `Other` iff its base tile index is
/// NOT in this set.
fn collect_explicit_tiles(rule: &Rule) -> HashSet<u32> {
    let mut tiles = HashSet::new();
    for group in &rule.input_groups {
        for matcher in &group.matchers {
            match matcher {
                CellMatcher::Tile(id) | CellMatcher::NotTile(id) => {
                    tiles.insert(*id);
                }
                CellMatcher::TileFlipped { id, .. } => {
                    tiles.insert(*id);
                }
                _ => {}
            }
        }
    }
    tiles
}

/// Perform a weighted random selection over `alts`. Returns the selected index,
/// or `None` if all weights are zero (no alternative can be chosen).
fn select_output_alternative(alts: &[OutputAlternative], rng: &mut impl Rng) -> Option<usize> {
    let total: u64 = alts.iter().map(|a| a.weight as u64).sum();
    if total == 0 {
        return None;
    }

    let mut pick = rng.gen_range(0..total);
    for (idx, alt) in alts.iter().enumerate() {
        if pick < alt.weight as u64 {
            return Some(idx);
        }
        pick -= alt.weight as u64;
    }

    // Unreachable if total > 0, but provide a safe fallback.
    Some(alts.len() - 1)
}

/// Look up the index of a layer by its UUID in `level.layers`.
/// Returns `None` if no layer with that ID exists.
fn find_layer_index(level: &Level, layer_id: Uuid) -> Option<usize> {
    level.layers.iter().position(|l| l.id == layer_id)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use uuid::Uuid;

    fn seeded_rng() -> SmallRng {
        SmallRng::seed_from_u64(0)
    }

    #[test]
    fn matcher_ignore_always_matches() {
        let tiles = HashSet::new();
        assert!(matcher_matches_cell(&CellMatcher::Ignore, None, &tiles));
        assert!(matcher_matches_cell(&CellMatcher::Ignore, Some(42), &tiles));
    }

    #[test]
    fn matcher_empty_matches_none_only() {
        let tiles = HashSet::new();
        assert!(matcher_matches_cell(&CellMatcher::Empty, None, &tiles));
        assert!(!matcher_matches_cell(&CellMatcher::Empty, Some(0), &tiles));
    }

    #[test]
    fn matcher_nonempty_matches_some_only() {
        let tiles = HashSet::new();
        assert!(!matcher_matches_cell(&CellMatcher::NonEmpty, None, &tiles));
        assert!(matcher_matches_cell(&CellMatcher::NonEmpty, Some(5), &tiles));
    }

    #[test]
    fn matcher_tile_strips_flip_bits() {
        use bevy_map_core::{TILE_FLIP_X, TILE_FLIP_Y};
        let tiles = HashSet::new();
        // tile index 3 with both flip bits set
        let packed = 3u32 | TILE_FLIP_X | TILE_FLIP_Y;
        // Tile(3) should still match because flip bits are stripped
        assert!(matcher_matches_cell(&CellMatcher::Tile(3), Some(packed), &tiles));
        // Tile(4) should not match
        assert!(!matcher_matches_cell(&CellMatcher::Tile(4), Some(packed), &tiles));
    }

    #[test]
    fn matcher_not_tile_matches_empty_and_different() {
        let tiles = HashSet::new();
        assert!(matcher_matches_cell(&CellMatcher::NotTile(5), None, &tiles));
        assert!(matcher_matches_cell(&CellMatcher::NotTile(5), Some(6), &tiles));
        assert!(!matcher_matches_cell(&CellMatcher::NotTile(5), Some(5), &tiles));
    }

    #[test]
    fn matcher_tile_flipped_requires_exact_flip_state() {
        use bevy_map_core::TILE_FLIP_X;
        let tiles = HashSet::new();
        let flip_x_only = 3u32 | TILE_FLIP_X;

        let m = CellMatcher::TileFlipped { id: 3, flip_x: true, flip_y: false };
        assert!(matcher_matches_cell(&m, Some(flip_x_only), &tiles));

        let m_wrong_y = CellMatcher::TileFlipped { id: 3, flip_x: true, flip_y: true };
        assert!(!matcher_matches_cell(&m_wrong_y, Some(flip_x_only), &tiles));

        let m_no_flip = CellMatcher::TileFlipped { id: 3, flip_x: false, flip_y: false };
        assert!(!matcher_matches_cell(&m_no_flip, Some(flip_x_only), &tiles));
    }

    #[test]
    fn matcher_other_fails_on_empty() {
        let tiles = HashSet::new();
        assert!(!matcher_matches_cell(&CellMatcher::Other, None, &tiles));
    }

    #[test]
    fn matcher_other_matches_tiles_not_in_explicit_set() {
        let mut explicit = HashSet::new();
        explicit.insert(1u32);
        explicit.insert(2u32);
        // tile 3 is not in explicit set → Other matches
        assert!(matcher_matches_cell(&CellMatcher::Other, Some(3), &explicit));
        // tile 1 is in explicit set → Other does not match
        assert!(!matcher_matches_cell(&CellMatcher::Other, Some(1), &explicit));
    }

    #[test]
    fn select_output_all_zero_weight_returns_none() {
        let mut rng = seeded_rng();
        let alts = vec![
            OutputAlternative {
                id: Uuid::new_v4(),
                layer_id: Uuid::new_v4(),
                half_width: 0,
                half_height: 0,
                outputs: vec![CellOutput::Ignore],
                weight: 0,
            },
        ];
        assert!(select_output_alternative(&alts, &mut rng).is_none());
    }

    #[test]
    fn select_output_single_alt_always_chosen() {
        let mut rng = seeded_rng();
        let alts = vec![
            OutputAlternative {
                id: Uuid::new_v4(),
                layer_id: Uuid::new_v4(),
                half_width: 0,
                half_height: 0,
                outputs: vec![CellOutput::Ignore],
                weight: 1,
            },
        ];
        assert_eq!(select_output_alternative(&alts, &mut rng), Some(0));
    }
}
