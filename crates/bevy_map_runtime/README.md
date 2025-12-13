# bevy_map_runtime

Runtime map loading and rendering for Bevy 0.17 using bevy_ecs_tilemap.

Part of [bevy_map_editor](https://github.com/jbuehler23/bevy_map_editor).

## Features

- Efficient tilemap rendering via bevy_ecs_tilemap 0.17
- Asset-based map loading with hot reload support
- Custom entity spawning with `#[derive(MapEntity)]`
- Auto-loading for animations and dialogues
- Runtime tile modification

## Quick Start

```rust
use bevy::prelude::*;
use bevy_map_runtime::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MapRuntimePlugin)
        .add_systems(Startup, load_map)
        .run();
}

fn load_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(MapBundle::new(asset_server.load("maps/level.map.json")));
}
```

## Custom Entities

Register entity types to spawn game objects from map data:

```rust
use bevy::prelude::*;
use bevy_map_derive::MapEntity;
use bevy_map_runtime::MapEntityRegistry;

#[derive(Component, MapEntity)]
#[map_entity(type_name = "Chest")]
pub struct Chest {
    #[map_prop]
    pub loot_table: String,
    #[map_prop(default = false)]
    pub locked: bool,
}

fn setup(mut registry: ResMut<MapEntityRegistry>) {
    registry.register::<Chest>();
}
```

## Auto-Loading Animations

Use `AnimatedSpriteHandle` to auto-load sprite animations from a map project:

```rust
use bevy_map_runtime::AnimatedSpriteHandle;

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AnimatedSpriteHandle::new(
            asset_server.load("maps/game.map.json"),
            "player_idle",  // animation name defined in editor
        ),
        Transform::default(),
    ));
}
```

## Auto-Loading Dialogues

Use `DialogueTreeHandle` to auto-load dialogues:

```rust
use bevy_map_runtime::DialogueTreeHandle;

fn spawn_npc(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        DialogueTreeHandle::new(
            asset_server.load("maps/game.map.json"),
            "merchant_greeting",  // dialogue name defined in editor
        ),
    ));
}
```

## Manual Animation Control

For direct control over sprites, use `#[map_sprite]`:

```rust
#[derive(Component, MapEntity)]
#[map_entity(type_name = "Player")]
pub struct Player {
    #[map_prop]
    pub speed: f32,

    #[map_sprite("player_sprite")]
    pub sprite: Option<Handle<Image>>,
}
```

## Re-exported Types

This crate re-exports commonly used types:

```rust
use bevy_map_runtime::{
    // Animation
    SpriteData, AnimationDef, AnimatedSprite, LoopMode,
    // Dialogue
    DialogueTree, DialogueNode, DialogueRunner,
    StartDialogueEvent, DialogueChoiceEvent, DialogueEndEvent,
};
```

## License

MIT OR Apache-2.0
