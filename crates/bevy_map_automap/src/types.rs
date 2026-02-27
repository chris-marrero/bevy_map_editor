//! Data types for the rule-based automapping engine.
//!
//! The primary entry point is [`AutomapConfig`], which holds a list of [`RuleSet`]s.
//! Each rule set runs its rules in order against a [`Level`](bevy_map_core::Level).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Config ──────────────────────────────────────────────────────────────────

/// Top-level automapping configuration stored per project.
///
/// Holds an ordered list of rule sets. Each rule set runs its rules in the order
/// they appear; rule sets themselves also run in order.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutomapConfig {
    pub rule_sets: Vec<RuleSet>,
}

// ─── RuleSet ─────────────────────────────────────────────────────────────────

/// A named group of rules that share a common [`RuleSetSettings`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    /// Stable identifier for this rule set.
    pub id: Uuid,
    pub name: String,
    pub rules: Vec<Rule>,
    pub settings: RuleSetSettings,
    /// When `true`, this rule set is skipped during [`apply_automap_config`](crate::apply_automap_config).
    #[serde(default)]
    pub disabled: bool,
}

/// Shared settings for a [`RuleSet`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSetSettings {
    pub edge_handling: EdgeHandling,
    pub apply_mode: ApplyMode,
}

impl Default for RuleSetSettings {
    fn default() -> Self {
        Self {
            edge_handling: EdgeHandling::default(),
            apply_mode: ApplyMode::default(),
        }
    }
}

/// How out-of-bounds cell accesses are handled when a pattern extends past the level edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EdgeHandling {
    /// The entire pattern position is skipped — no rule fires if any cell is out of bounds.
    #[default]
    Skip,
    /// Out-of-bounds cells are treated as empty (`None`). Matchers that require a tile
    /// (e.g., [`CellMatcher::Tile`], [`CellMatcher::NonEmpty`]) will fail on OOB cells.
    TreatAsEmpty,
}

/// Controls how many times a rule set is applied to the level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ApplyMode {
    /// Apply the rule set exactly once (one full pass over the level). Default.
    #[default]
    Once,
    /// Re-apply the rule set until no cells change between passes, or until
    /// [`UNTIL_STABLE_MAX_ITERATIONS`](crate::UNTIL_STABLE_MAX_ITERATIONS) is reached.
    ///
    /// Note: rule sets with probabilistic outputs may never reach stability if different
    /// alternatives can be selected on each pass. The iteration cap guarantees termination.
    UntilStable,
}

// ─── Rule ─────────────────────────────────────────────────────────────────────

/// A single automapping rule.
///
/// A rule fires at position `(x, y)` when **all** input condition groups match
/// at that position (AND logic). When it fires, one output alternative is chosen
/// by weighted random selection and applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Stable identifier for this rule.
    pub id: Uuid,
    pub name: String,
    /// All groups must match for the rule to fire (AND logic).
    pub input_groups: Vec<InputConditionGroup>,
    /// Exactly one alternative is selected per match, by weighted random selection.
    pub output_alternatives: Vec<OutputAlternative>,
    /// When `true`, cells written by an earlier match in this rule's scan are not
    /// overwritten by later matches in the same pass.
    #[serde(default)]
    pub no_overlapping_output: bool,
}

// ─── InputConditionGroup ─────────────────────────────────────────────────────

/// A grid of [`CellMatcher`]s applied to a single layer.
///
/// The pattern is `(2 * half_width + 1)` cells wide and `(2 * half_height + 1)`
/// cells tall, stored in row-major order. The center cell (at `[half_height *
/// (2 * half_width + 1) + half_width]`) corresponds to the candidate position
/// `(x, y)` in the level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConditionGroup {
    /// References the [`Layer::id`](bevy_map_core::Layer) this group reads from.
    pub layer_id: Uuid,
    /// Half-width of the pattern (full width = `2 * half_width + 1`).
    pub half_width: u32,
    /// Half-height of the pattern (full height = `2 * half_height + 1`).
    pub half_height: u32,
    /// Row-major cell matchers. Length must equal `(2*half_width+1) * (2*half_height+1)`.
    pub matchers: Vec<CellMatcher>,
}

/// Determines whether a single cell matches a condition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellMatcher {
    /// Always matches. Use this to ignore a cell in the pattern.
    Ignore,
    /// Matches only if the cell is empty (`None`).
    Empty,
    /// Matches only if the cell contains any tile.
    NonEmpty,
    /// Matches if the cell contains a tile with this index, regardless of flip state.
    /// Flip bits are stripped before comparison.
    Tile(u32),
    /// Matches if the cell is empty OR contains a tile with a different index.
    /// Flip bits are stripped before comparison.
    NotTile(u32),
    /// Matches if the cell contains a tile with this exact index and flip state.
    /// Both the tile index and the flip bits must match.
    TileFlipped { id: u32, flip_x: bool, flip_y: bool },
    /// Matches any tile whose base index is not listed as [`Tile`](CellMatcher::Tile),
    /// [`NotTile`](CellMatcher::NotTile), or [`TileFlipped`](CellMatcher::TileFlipped)
    /// anywhere in the containing rule's input groups.
    ///
    /// If the rule has no `Tile`, `NotTile`, or `TileFlipped` matchers, `Other` matches
    /// every non-empty cell.
    Other,
}

// ─── OutputAlternative ───────────────────────────────────────────────────────

/// One possible output when a rule fires.
///
/// Multiple alternatives on a rule allow weighted random selection of different
/// tile configurations. An alternative with `weight = 0` is never selected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputAlternative {
    /// Stable identifier for this alternative.
    pub id: Uuid,
    /// References the [`Layer::id`](bevy_map_core::Layer) this alternative writes to.
    pub layer_id: Uuid,
    /// Half-width of the output grid.
    pub half_width: u32,
    /// Half-height of the output grid.
    pub half_height: u32,
    /// Row-major cell outputs. Length must equal `(2*half_width+1) * (2*half_height+1)`.
    pub outputs: Vec<CellOutput>,
    /// Selection weight. `0` means this alternative is never chosen. Default: 1.
    #[serde(default = "default_weight")]
    pub weight: u32,
}

fn default_weight() -> u32 {
    1
}

/// The output written to a single cell when a rule fires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellOutput {
    /// Leave the cell unchanged.
    Ignore,
    /// Erase the cell (write `None`).
    Empty,
    /// Write this tile value to the cell. Flip bits in the value are preserved as-is.
    Tile(u32),
}
