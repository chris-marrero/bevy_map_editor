use crate::plugin_meta::ContributionDef;

/// A runtime UI contribution for the editor.
#[derive(Debug, Clone)]
pub enum EditorExtension {
    Panel {
        name: String,
    },
    MenuItem {
        path: String,
    },
    InspectorSection {
        name: String,
    },
    ToolbarButton {
        name: String,
    },
    ContextMenu {
        target: ContextMenuTarget,
        name: String,
    },
}

/// Target for a context menu contribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextMenuTarget {
    Entity,
    Layer,
}

impl EditorExtension {
    /// Convert a metadata contribution definition into a runtime editor extension.
    pub fn from_def(def: ContributionDef) -> Option<Self> {
        match def {
            ContributionDef::Panel { name } => Some(EditorExtension::Panel { name }),
            ContributionDef::MenuItem { path } => Some(EditorExtension::MenuItem { path }),
            ContributionDef::InspectorSection { name } => {
                Some(EditorExtension::InspectorSection { name })
            }
            ContributionDef::ToolbarButton { name } => {
                Some(EditorExtension::ToolbarButton { name })
            }
            ContributionDef::ContextMenu { target, name } => {
                let target = match target.to_lowercase().as_str() {
                    "entity" => ContextMenuTarget::Entity,
                    "layer" => ContextMenuTarget::Layer,
                    _ => return None,
                };
                Some(EditorExtension::ContextMenu { target, name })
            }
        }
    }
}
