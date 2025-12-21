//! Entity registry for automatic entity spawning from map data
//!
//! This module provides a registry-based system for spawning game entities
//! from EntityInstance data in map files.

use bevy::prelude::*;
use bevy_map_core::{EntityInstance, Value};
use std::collections::HashMap;
use std::marker::PhantomData;
use uuid::Uuid;

/// Trait implemented by entities that can be spawned from map data.
///
/// This trait is typically implemented via the `#[derive(MapEntity)]` macro
/// from `bevy_map_derive`.
///
/// # Example
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_map_derive::MapEntity;
///
/// #[derive(Component, MapEntity)]
/// #[map_entity(type_name = "NPC")]
/// pub struct Npc {
///     #[map_prop]
///     pub name: String,
///     #[map_prop(default = 100)]
///     pub health: i32,
/// }
/// ```
pub trait MapEntityType: Component + Sized + Send + Sync + 'static {
    /// Returns the type name as used in the map editor
    fn type_name() -> &'static str;

    /// Creates an instance of this component from map entity data
    fn from_instance(instance: &EntityInstance) -> Self;

    /// Returns the property names for sprite fields (for manual sprite handle injection)
    /// Override this if your entity has fields that should receive sprite handles.
    fn sprite_properties() -> &'static [&'static str] {
        &[]
    }

    /// Inject a sprite handle into the component for the given property name.
    /// Override this if your entity has sprite fields that need manual handle injection.
    fn inject_sprite_handle(
        &mut self,
        _property_name: &str,
        _handle: bevy::prelude::Handle<bevy::prelude::Image>,
    ) {
        // Default: no-op
    }
}

/// Marker component for entities spawned from map data
#[derive(Component)]
pub struct MapEntityMarker {
    /// The unique ID of the original EntityInstance
    pub instance_id: Uuid,
    /// The type name from the map editor
    pub type_name: String,
}

/// Raw properties from the map editor, accessible by runtime systems
#[derive(Component, Debug, Clone)]
pub struct EntityProperties {
    /// All properties as they were defined in the map editor
    pub properties: HashMap<String, Value>,
}

/// Component marking an entity that has an associated dialogue
///
/// This is automatically attached to entities that have a "dialogue" property
/// or properties ending in "_dialogue" in their map data.
///
/// # Example
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_map_runtime::{Dialogue, MapDialogues};
///
/// fn interact_with_npc(
///     query: Query<(Entity, &Dialogue)>,
///     map_dialogues: Res<MapDialogues>,
/// ) {
///     for (entity, dialogue) in query.iter() {
///         if let Some(tree) = map_dialogues.get(&dialogue.dialogue_id) {
///             // Start the dialogue
///         }
///     }
/// }
/// ```
#[derive(Component, Debug, Clone)]
pub struct Dialogue {
    /// The ID of the dialogue tree associated with this entity
    pub dialogue_id: String,
}

/// Trait object for spawning entities
trait EntitySpawner: Send + Sync {
    fn spawn(&self, commands: &mut Commands, instance: &EntityInstance, transform: Transform);
}

/// Generic spawner implementation for any MapEntityType
struct TypedSpawner<T: MapEntityType> {
    _marker: PhantomData<T>,
}

impl<T: MapEntityType> EntitySpawner for TypedSpawner<T> {
    fn spawn(&self, commands: &mut Commands, instance: &EntityInstance, transform: Transform) {
        let component = T::from_instance(instance);

        // Parse entity color from instance if available, otherwise use a default
        let color = instance
            .get_string("_editor_color")
            .and_then(parse_hex_color)
            .unwrap_or(Color::srgba(0.2, 0.6, 1.0, 0.8)); // Default blue

        // Get marker size from instance or use default
        let marker_size = instance.get_float("_editor_marker_size").unwrap_or(16.0) as f32;

        commands.spawn((
            component,
            transform,
            // Required for visibility
            Visibility::default(),
            // Placeholder visual - colored rectangle
            Sprite {
                color,
                custom_size: Some(Vec2::splat(marker_size)),
                ..default()
            },
            MapEntityMarker {
                instance_id: instance.id,
                type_name: instance.type_name.clone(),
            },
            EntityProperties {
                properties: instance.properties.clone(),
            },
        ));
    }
}

/// Parse a hex color string like "#ff0000" or "#ff000080" (with alpha)
fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::srgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            0.8,
        ))
    } else if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
        Some(Color::srgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ))
    } else {
        None
    }
}

/// Registry storing entity spawners by type name
#[derive(Resource, Default)]
pub struct EntityRegistry {
    spawners: HashMap<String, Box<dyn EntitySpawner>>,
}

impl EntityRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new entity type
    pub fn register<T: MapEntityType>(&mut self) {
        self.spawners.insert(
            T::type_name().to_string(),
            Box::new(TypedSpawner::<T> {
                _marker: PhantomData,
            }),
        );
    }

    /// Check if a type is registered
    pub fn is_registered(&self, type_name: &str) -> bool {
        self.spawners.contains_key(type_name)
    }

    /// Get the number of registered types
    pub fn len(&self) -> usize {
        self.spawners.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.spawners.is_empty()
    }

    /// Spawn an entity from an EntityInstance
    ///
    /// Returns true if the entity type was registered, false otherwise.
    /// Note: Unregistered entities are still spawned with a placeholder visual
    /// so they're visible in the game for debugging purposes.
    pub fn spawn(
        &self,
        commands: &mut Commands,
        instance: &EntityInstance,
        base_transform: Transform,
    ) -> bool {
        // Create transform from instance position + base transform
        let entity_transform =
            base_transform * Transform::from_xyz(instance.position[0], instance.position[1], 0.0);

        if let Some(spawner) = self.spawners.get(&instance.type_name) {
            spawner.spawn(commands, instance, entity_transform);
            true
        } else {
            // Spawn unregistered entities with a placeholder visual (red = unregistered)
            commands.spawn((
                entity_transform,
                Visibility::default(),
                Sprite {
                    color: Color::srgba(1.0, 0.2, 0.2, 0.8), // Red for unregistered
                    custom_size: Some(Vec2::splat(16.0)),
                    ..default()
                },
                MapEntityMarker {
                    instance_id: instance.id,
                    type_name: instance.type_name.clone(),
                },
                EntityProperties {
                    properties: instance.properties.clone(),
                },
            ));
            false
        }
    }

    /// Spawn all entities from a list of instances
    ///
    /// Returns the number of unregistered entity types encountered.
    /// Note: All entities are spawned, but unregistered ones get a red placeholder
    /// instead of their game-specific component.
    pub fn spawn_all(
        &self,
        commands: &mut Commands,
        instances: &[EntityInstance],
        base_transform: Transform,
    ) -> usize {
        let mut unregistered = 0;
        for instance in instances {
            if !self.spawn(commands, instance, base_transform) {
                warn!(
                    "Entity type '{}' not registered - spawned with red placeholder (use .register_map_entity::<YourType>() to register)",
                    instance.type_name
                );
                unregistered += 1;
            }
        }
        unregistered
    }
}

/// Extension trait for registering map entities with the Bevy App
pub trait MapEntityExt {
    /// Register a map entity type for automatic spawning
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bevy::prelude::*;
    /// use bevy_map_runtime::prelude::*;
    ///
    /// fn main() {
    ///     App::new()
    ///         .add_plugins(DefaultPlugins)
    ///         .add_plugins(MapRuntimePlugin)
    ///         .register_map_entity::<Npc>()
    ///         .register_map_entity::<Enemy>()
    ///         .run();
    /// }
    /// ```
    fn register_map_entity<T: MapEntityType>(&mut self) -> &mut Self;
}

impl MapEntityExt for App {
    fn register_map_entity<T: MapEntityType>(&mut self) -> &mut Self {
        // Ensure EntityRegistry resource exists
        if !self.world().contains_resource::<EntityRegistry>() {
            self.insert_resource(EntityRegistry::new());
        }

        // Register the type
        self.world_mut()
            .resource_mut::<EntityRegistry>()
            .register::<T>();

        self
    }
}

/// System that automatically attaches `Dialogue` components to entities with dialogue properties
///
/// This system runs each frame and looks for entities that have:
/// - `EntityProperties` component
/// - A property named "dialogue" or ending with "_dialogue"
/// - No existing `Dialogue` component
///
/// When found, it extracts the dialogue ID and attaches a `Dialogue` component.
pub fn attach_dialogues(
    mut commands: Commands,
    query: Query<(Entity, &EntityProperties), Without<Dialogue>>,
) {
    for (entity, props) in query.iter() {
        // Look for dialogue properties
        for (key, value) in &props.properties {
            let is_dialogue_prop = key == "dialogue" || key.ends_with("_dialogue");
            if !is_dialogue_prop {
                continue;
            }

            // Extract dialogue ID from the value
            let dialogue_id = match value {
                // Direct string reference to dialogue ID
                Value::String(id) => Some(id.clone()),
                // Object with "id" field (embedded dialogue reference)
                Value::Object(obj) => obj.get("id").and_then(|v| {
                    if let Value::String(id) = v {
                        Some(id.clone())
                    } else {
                        None
                    }
                }),
                _ => None,
            };

            if let Some(id) = dialogue_id {
                if !id.is_empty() {
                    commands.entity(entity).insert(Dialogue { dialogue_id: id });
                    break; // Only attach one dialogue per entity
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    #[allow(dead_code)]
    struct TestEntity {
        name: String,
        health: i32,
    }

    impl MapEntityType for TestEntity {
        fn type_name() -> &'static str {
            "TestEntity"
        }

        fn from_instance(instance: &EntityInstance) -> Self {
            Self {
                name: instance.get_string("name").unwrap_or("Unknown").to_string(),
                health: instance.get_int("health").unwrap_or(100) as i32,
            }
        }
    }

    #[test]
    fn test_registry_register() {
        let mut registry = EntityRegistry::new();
        assert!(registry.is_empty());

        registry.register::<TestEntity>();

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.is_registered("TestEntity"));
        assert!(!registry.is_registered("OtherEntity"));
    }
}
