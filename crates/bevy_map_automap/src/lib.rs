//! Rule-based automapping engine for bevy_map_editor.
//!
//! The primary entry point is [`apply_automap_config`], which applies an [`AutomapConfig`]
//! to a [`Level`](bevy_map_core::Level) using a caller-supplied random number generator.
//!
//! This crate has no Bevy dependency. It operates on plain data from `bevy_map_core`.

mod apply;
mod types;

pub use apply::apply_automap_config;
pub use types::{
    ApplyMode, AutomapConfig, CellMatcher, CellOutput, EdgeHandling, InputConditionGroup,
    OutputAlternative, Rule, RuleSet, RuleSetSettings,
};

/// Maximum number of full-pass iterations in [`ApplyMode::UntilStable`] mode
/// before aborting and returning with the current state.
///
/// A rule set that does not converge within this many passes is assumed to be
/// cycling — this is common with probabilistic output alternatives, where
/// different tiles can be selected on each pass. The cap guarantees termination.
///
/// At 100 iterations on a 256×256 level with 10 rules, this is approximately
/// 65 million cell evaluations — measurable but not a hang.
pub const UNTIL_STABLE_MAX_ITERATIONS: u32 = 100;
