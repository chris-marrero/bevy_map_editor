//! bevy_map_dialogue - Dialogue tree types and runtime for bevy_map_editor
//!
//! This crate provides types for creating branching dialogue trees with support for:
//! - Multiple node types (Text, Choice, Condition, Action, End)
//! - Player choices with optional conditions
//! - Speaker assignments
//! - Conditional branching
//! - Action triggers
//!
//! # Usage
//!
//! ```rust,ignore
//! use bevy_map_dialogue::{DialogueTree, DialogueNode, DialogueNodeType, DialogueChoice};
//!
//! let mut tree = DialogueTree::new("greeting");
//! tree.name = "Merchant Greeting".to_string();
//!
//! // Add a choice node
//! let choice_node = DialogueNode {
//!     node_type: DialogueNodeType::Choice,
//!     speaker: "Merchant".to_string(),
//!     text: "Welcome! What would you like?".to_string(),
//!     choices: vec![
//!         DialogueChoice { text: "Buy items".to_string(), next_node: Some("shop".to_string()), ..default() },
//!         DialogueChoice { text: "Sell items".to_string(), next_node: Some("sell".to_string()), ..default() },
//!         DialogueChoice { text: "Goodbye".to_string(), next_node: Some("end".to_string()), ..default() },
//!     ],
//!     ..default()
//! };
//! tree.add_node(choice_node);
//! ```

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Type of dialogue node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Reflect)]
#[serde(rename_all = "lowercase")]
pub enum DialogueNodeType {
    /// NPC speaks text, then continues to next node
    #[default]
    Text,
    /// Player chooses from multiple options
    Choice,
    /// Check a condition and branch
    Condition,
    /// Execute an action (give item, start quest, etc.)
    Action,
    /// End of dialogue
    End,
}

impl DialogueNodeType {
    /// Get the display name for this node type
    pub fn display_name(&self) -> &'static str {
        match self {
            DialogueNodeType::Text => "Text",
            DialogueNodeType::Choice => "Choice",
            DialogueNodeType::Condition => "Condition",
            DialogueNodeType::Action => "Action",
            DialogueNodeType::End => "End",
        }
    }

    /// Get all available node types
    pub fn all() -> &'static [DialogueNodeType] {
        &[
            DialogueNodeType::Text,
            DialogueNodeType::Choice,
            DialogueNodeType::Condition,
            DialogueNodeType::Action,
            DialogueNodeType::End,
        ]
    }

    /// Get the color for this node type (RGB)
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            DialogueNodeType::Text => (100, 149, 237),      // Cornflower blue
            DialogueNodeType::Choice => (255, 165, 0),     // Orange
            DialogueNodeType::Condition => (147, 112, 219), // Medium purple
            DialogueNodeType::Action => (50, 205, 50),     // Lime green
            DialogueNodeType::End => (220, 20, 60),        // Crimson
        }
    }
}

/// A player choice option in a dialogue
#[derive(Debug, Clone, Serialize, Deserialize, Default, Reflect)]
pub struct DialogueChoice {
    /// Display text for this choice
    pub text: String,
    /// Node to go to when this choice is selected
    pub next_node: Option<String>,
    /// Condition required to show this choice (script/expression string)
    pub condition: Option<String>,
}

impl DialogueChoice {
    /// Create a new choice with text and target node
    pub fn new(text: impl Into<String>, next_node: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            next_node: Some(next_node.into()),
            condition: None,
        }
    }

    /// Create a choice with a condition
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }
}

fn default_position() -> (f32, f32) {
    (0.0, 0.0)
}

/// A single node in the dialogue tree
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct DialogueNode {
    /// Unique identifier for this node
    pub id: String,
    /// Type of this node
    #[serde(default)]
    pub node_type: DialogueNodeType,
    /// Speaker name (for text nodes)
    #[serde(default)]
    pub speaker: String,
    /// The text content of this node
    #[serde(default)]
    pub text: String,
    /// Choices available to the player (for choice nodes)
    #[serde(default)]
    pub choices: Vec<DialogueChoice>,
    /// Next node to go to (for linear flow)
    pub next_node: Option<String>,
    /// Condition to check before showing this node (script/expression string)
    pub condition: Option<String>,
    /// Action to execute when entering this node (script/expression string)
    pub action: Option<String>,
    /// Position in the editor (x, y)
    #[serde(default = "default_position")]
    pub position: (f32, f32),
}

impl Default for DialogueNode {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            node_type: DialogueNodeType::Text,
            speaker: String::new(),
            text: String::new(),
            choices: Vec::new(),
            next_node: None,
            condition: None,
            action: None,
            position: (0.0, 0.0),
        }
    }
}

impl DialogueNode {
    /// Create a new text node
    pub fn new_text(speaker: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            speaker: speaker.into(),
            text: text.into(),
            ..Default::default()
        }
    }

    /// Create a new choice node
    pub fn new_choice(speaker: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            node_type: DialogueNodeType::Choice,
            speaker: speaker.into(),
            text: prompt.into(),
            ..Default::default()
        }
    }

    /// Create an end node
    pub fn new_end() -> Self {
        Self {
            node_type: DialogueNodeType::End,
            ..Default::default()
        }
    }

    /// Create a condition node
    pub fn new_condition(condition: impl Into<String>) -> Self {
        Self {
            node_type: DialogueNodeType::Condition,
            condition: Some(condition.into()),
            ..Default::default()
        }
    }

    /// Create an action node
    pub fn new_action(action: impl Into<String>) -> Self {
        Self {
            node_type: DialogueNodeType::Action,
            action: Some(action.into()),
            ..Default::default()
        }
    }

    /// Set the next node
    pub fn with_next(mut self, next_node: impl Into<String>) -> Self {
        self.next_node = Some(next_node.into());
        self
    }

    /// Add a choice
    pub fn with_choice(mut self, choice: DialogueChoice) -> Self {
        self.choices.push(choice);
        self
    }

    /// Set the position
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }
}

fn default_dialogue_id() -> String {
    Uuid::new_v4().to_string()
}

/// A complete dialogue tree with nodes and connections
#[derive(Debug, Clone, Serialize, Deserialize, Default, Asset, Reflect)]
pub struct DialogueTree {
    /// Unique identifier for the dialogue tree
    #[serde(default = "default_dialogue_id")]
    pub id: String,
    /// Display name for the dialogue
    #[serde(default)]
    pub name: String,
    /// The ID of the starting node
    #[serde(default)]
    pub start_node: String,
    /// All nodes in the dialogue tree
    #[serde(default)]
    #[reflect(ignore)]
    pub nodes: HashMap<String, DialogueNode>,
}

impl DialogueTree {
    /// Create a new empty dialogue tree with a starting text node
    pub fn new(name: impl Into<String>) -> Self {
        let start_id = Uuid::new_v4().to_string();
        let mut nodes = HashMap::new();
        nodes.insert(
            start_id.clone(),
            DialogueNode {
                id: start_id.clone(),
                node_type: DialogueNodeType::Text,
                speaker: String::new(),
                text: "Hello!".to_string(),
                choices: Vec::new(),
                next_node: None,
                condition: None,
                action: None,
                position: (100.0, 100.0),
            },
        );
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            start_node: start_id,
            nodes,
        }
    }

    /// Create an empty dialogue tree without any nodes
    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            start_node: String::new(),
            nodes: HashMap::new(),
        }
    }

    /// Add a new node to the tree
    pub fn add_node(&mut self, node: DialogueNode) -> String {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        id
    }

    /// Remove a node from the tree
    pub fn remove_node(&mut self, id: &str) {
        self.nodes.remove(id);
        // Clean up references to this node
        for node in self.nodes.values_mut() {
            if node.next_node.as_deref() == Some(id) {
                node.next_node = None;
            }
            node.choices.retain(|c| c.next_node.as_deref() != Some(id));
        }
        if self.start_node == id {
            self.start_node = String::new();
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&DialogueNode> {
        self.nodes.get(id)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut DialogueNode> {
        self.nodes.get_mut(id)
    }

    /// Get the start node
    pub fn get_start_node(&self) -> Option<&DialogueNode> {
        self.nodes.get(&self.start_node)
    }

    /// Set the start node
    pub fn set_start_node(&mut self, id: impl Into<String>) {
        self.start_node = id.into();
    }

    /// Get all node IDs
    pub fn node_ids(&self) -> impl Iterator<Item = &str> {
        self.nodes.keys().map(|s| s.as_str())
    }

    /// Check if the tree is valid (has start node and all references are valid)
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.start_node.is_empty() {
            errors.push("No start node defined".to_string());
        } else if !self.nodes.contains_key(&self.start_node) {
            errors.push(format!("Start node '{}' not found", self.start_node));
        }

        for (id, node) in &self.nodes {
            if let Some(next) = &node.next_node {
                if !self.nodes.contains_key(next) {
                    errors.push(format!("Node '{}' references non-existent node '{}'", id, next));
                }
            }
            for choice in &node.choices {
                if let Some(next) = &choice.next_node {
                    if !self.nodes.contains_key(next) {
                        errors.push(format!(
                            "Choice '{}' in node '{}' references non-existent node '{}'",
                            choice.text, id, next
                        ));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Component that holds a handle to a dialogue tree asset
#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct DialogueHandle(#[reflect(ignore)] pub Handle<DialogueTree>);

impl DialogueHandle {
    /// Create a new dialogue handle
    pub fn new(handle: Handle<DialogueTree>) -> Self {
        Self(handle)
    }
}

/// Message to start a dialogue (sent via MessageWriter, read via MessageReader)
#[derive(Message, Debug, Clone)]
pub struct StartDialogueEvent {
    /// The entity that owns the dialogue (e.g., NPC)
    pub speaker_entity: Entity,
    /// The dialogue tree asset handle
    pub dialogue: Handle<DialogueTree>,
}

/// Message sent when the player makes a choice
#[derive(Message, Debug, Clone)]
pub struct DialogueChoiceEvent {
    /// Index of the choice selected
    pub choice_index: usize,
}

/// Message sent when a dialogue ends
#[derive(Message, Debug, Clone)]
pub struct DialogueEndEvent {
    /// The entity that owned the dialogue
    pub speaker_entity: Entity,
}

/// Current state of an active dialogue
#[derive(Resource, Debug, Clone, Default)]
pub struct DialogueRunner {
    /// Whether a dialogue is currently active
    pub active: bool,
    /// The entity that started this dialogue
    pub speaker_entity: Option<Entity>,
    /// Handle to the current dialogue tree
    pub dialogue_handle: Option<Handle<DialogueTree>>,
    /// Current node ID
    pub current_node_id: Option<String>,
}

impl DialogueRunner {
    /// Start a new dialogue
    pub fn start(&mut self, speaker: Entity, dialogue: Handle<DialogueTree>, start_node: String) {
        self.active = true;
        self.speaker_entity = Some(speaker);
        self.dialogue_handle = Some(dialogue);
        self.current_node_id = Some(start_node);
    }

    /// End the current dialogue
    pub fn end(&mut self) {
        self.active = false;
        self.speaker_entity = None;
        self.dialogue_handle = None;
        self.current_node_id = None;
    }

    /// Advance to the next node
    pub fn advance_to(&mut self, node_id: String) {
        self.current_node_id = Some(node_id);
    }

    /// Check if a dialogue is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Plugin for dialogue support
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<DialogueTree>()
            .register_type::<DialogueNodeType>()
            .register_type::<DialogueChoice>()
            .register_type::<DialogueNode>()
            .register_type::<DialogueTree>()
            .register_type::<DialogueHandle>()
            .init_resource::<DialogueRunner>()
            .init_resource::<Messages<StartDialogueEvent>>()
            .init_resource::<Messages<DialogueChoiceEvent>>()
            .init_resource::<Messages<DialogueEndEvent>>()
            .add_systems(Update, (handle_start_dialogue, handle_dialogue_choice));
    }
}

/// System to handle starting dialogues
fn handle_start_dialogue(
    mut events: MessageReader<StartDialogueEvent>,
    mut runner: ResMut<DialogueRunner>,
    dialogues: Res<Assets<DialogueTree>>,
) {
    for event in events.read() {
        if let Some(tree) = dialogues.get(&event.dialogue) {
            if !tree.start_node.is_empty() {
                runner.start(event.speaker_entity, event.dialogue.clone(), tree.start_node.clone());
            }
        }
    }
}

/// System to handle dialogue choices
fn handle_dialogue_choice(
    mut choice_events: MessageReader<DialogueChoiceEvent>,
    mut end_events: MessageWriter<DialogueEndEvent>,
    mut runner: ResMut<DialogueRunner>,
    dialogues: Res<Assets<DialogueTree>>,
) {
    for event in choice_events.read() {
        if !runner.active {
            continue;
        }

        let Some(handle) = &runner.dialogue_handle else { continue };
        let Some(tree) = dialogues.get(handle) else { continue };
        let Some(current_id) = &runner.current_node_id else { continue };
        let Some(node) = tree.get_node(current_id) else { continue };

        // Handle based on node type
        match node.node_type {
            DialogueNodeType::Text => {
                // Advance to next node or end
                if let Some(next) = &node.next_node {
                    runner.advance_to(next.clone());
                } else {
                    let speaker = runner.speaker_entity;
                    runner.end();
                    if let Some(entity) = speaker {
                        end_events.write(DialogueEndEvent { speaker_entity: entity });
                    }
                }
            }
            DialogueNodeType::Choice => {
                // Select the choice and advance
                if let Some(choice) = node.choices.get(event.choice_index) {
                    if let Some(next) = &choice.next_node {
                        runner.advance_to(next.clone());
                    } else {
                        let speaker = runner.speaker_entity;
                        runner.end();
                        if let Some(entity) = speaker {
                            end_events.write(DialogueEndEvent { speaker_entity: entity });
                        }
                    }
                }
            }
            DialogueNodeType::End => {
                let speaker = runner.speaker_entity;
                runner.end();
                if let Some(entity) = speaker {
                    end_events.write(DialogueEndEvent { speaker_entity: entity });
                }
            }
            _ => {
                // For condition/action nodes, advance to next
                if let Some(next) = &node.next_node {
                    runner.advance_to(next.clone());
                }
            }
        }
    }
}
