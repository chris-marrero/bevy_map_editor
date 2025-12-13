//! Custom Entities Demo - Loading Entities from JSON
//!
//! Demonstrates loading entities with properties from a .map.json file.
//! Define entity types in your game, then place them in the editor.
//!
//! This example shows the **recommended approach**: use `MapHandle` to load maps
//! and register custom entity types with `register_map_entity`. Entities are
//! automatically spawned with their components when the map loads.
//!
//! Controls:
//! - Space: List all entities in console
//! - Tab: Cycle through entity type filters
//!
//! Run with: cargo run --example custom_entities_demo -p bevy_map_editor_examples

use bevy::prelude::*;
use bevy_map_derive::MapEntity;
use bevy_map_runtime::{MapEntityExt, MapHandle, MapRuntimePlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Custom Entities Demo - bevy_map_editor".to_string(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MapRuntimePlugin)
        // Register your entity types - maps JSON type_name to Rust component
        .register_map_entity::<Npc>()
        .register_map_entity::<Chest>()
        .register_map_entity::<Item>()
        .init_resource::<FilterState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, update_display))
        .run();
}

/// NPC entity - automatically spawned from JSON
#[derive(Component, MapEntity, Debug, Clone)]
#[map_entity(type_name = "NPC")]
pub struct Npc {
    #[map_prop]
    pub name: String,
    #[map_prop(default = 100)]
    pub health: i32,
    #[map_prop(default = 1)]
    pub level: i32,
    #[map_prop(default = false)]
    pub hostile: bool,
}

/// Chest entity
#[derive(Component, MapEntity, Debug, Clone)]
#[map_entity(type_name = "Chest")]
pub struct Chest {
    #[map_prop]
    pub loot_table: String,
    #[map_prop(default = false)]
    pub locked: bool,
    #[map_prop(default = 1)]
    pub tier: i32,
}

/// Item entity
#[derive(Component, MapEntity, Debug, Clone)]
#[map_entity(type_name = "Item")]
pub struct Item {
    #[map_prop]
    pub name: String,
    #[map_prop(default = 1)]
    pub quantity: i32,
    #[map_prop(default = false)]
    pub quest_item: bool,
}

#[derive(Resource, Default)]
struct FilterState {
    current: usize, // 0=all, 1=NPC, 2=Chest, 3=Item
}

#[derive(Component)]
struct InfoDisplay;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, Transform::from_xyz(64.0, 64.0, 0.0)));

    // Load and spawn map - ONE LINE! Entities are auto-spawned based on registered types.
    commands.spawn(MapHandle(asset_server.load("maps/custom_entities_demo.map.json")));

    // Spawn info display
    commands.spawn((
        Text::new("Custom Entities Demo\n\nLoading..."),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            max_width: Val::Px(350.0),
            ..default()
        },
        InfoDisplay,
    ));

    info!("Custom Entities Demo Started - loading via asset system!");
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut filter: ResMut<FilterState>,
    npcs: Query<(Entity, &Npc, &Transform)>,
    chests: Query<(Entity, &Chest, &Transform)>,
    items: Query<(Entity, &Item, &Transform)>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        info!("=== Entity List ===");
        for (e, npc, t) in npcs.iter() {
            info!("{:?} at {:?}: {} HP:{} Lvl:{}", e, t.translation.xy(), npc.name, npc.health, npc.level);
        }
        for (e, chest, t) in chests.iter() {
            info!("{:?} at {:?}: {} tier:{} locked:{}", e, t.translation.xy(), chest.loot_table, chest.tier, chest.locked);
        }
        for (e, item, t) in items.iter() {
            info!("{:?} at {:?}: {} x{} quest:{}", e, t.translation.xy(), item.name, item.quantity, item.quest_item);
        }
    }

    if keyboard.just_pressed(KeyCode::Tab) {
        filter.current = (filter.current + 1) % 4;
    }
}

fn update_display(
    filter: Res<FilterState>,
    npcs: Query<&Npc>,
    chests: Query<&Chest>,
    items: Query<&Item>,
    mut display_query: Query<&mut Text, With<InfoDisplay>>,
) {
    let Ok(mut text) = display_query.single_mut() else { return };

    let filter_name = match filter.current {
        0 => "All", 1 => "NPCs", 2 => "Chests", 3 => "Items", _ => "?"
    };

    let mut display = format!(
        "Custom Entities Demo (Asset-based)\n\nSPACE: List (console)\nTAB: Filter\n\nFilter: {}\n\n",
        filter_name
    );

    if filter.current == 0 || filter.current == 1 {
        display.push_str(&format!("NPCs ({})\n", npcs.iter().count()));
        for npc in npcs.iter() {
            display.push_str(&format!("  {} HP:{} Lvl:{}\n", npc.name, npc.health, npc.level));
        }
    }

    if filter.current == 0 || filter.current == 2 {
        display.push_str(&format!("\nChests ({})\n", chests.iter().count()));
        for chest in chests.iter() {
            let lock = if chest.locked { "LOCKED" } else { "open" };
            display.push_str(&format!("  {} T{} ({})\n", chest.loot_table, chest.tier, lock));
        }
    }

    if filter.current == 0 || filter.current == 3 {
        display.push_str(&format!("\nItems ({})\n", items.iter().count()));
        for item in items.iter() {
            let quest = if item.quest_item { "[Q]" } else { "" };
            display.push_str(&format!("  {} x{} {}\n", item.name, item.quantity, quest));
        }
    }

    *text = Text::new(display);
}
