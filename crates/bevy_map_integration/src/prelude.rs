pub use crate::manager::PluginManager;
pub use crate::plugin_meta::{PluginInfo, PluginMeta, PropertyDef, PropertyType};
pub use crate::registry::IntegrationRegistry;

#[cfg(feature = "editor")]
pub use crate::editor::{ContextMenuTarget, EditorExtension};
