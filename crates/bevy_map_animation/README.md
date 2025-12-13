# bevy_map_animation

Sprite sheet animations for the bevy_map_editor ecosystem.

Part of [bevy_map_editor](https://github.com/jbuehler23/bevy_map_editor).

## Features

- Define sprite sheets with frame dimensions
- Multiple named animations per sheet
- Loop modes: Loop, Once, PingPong
- Frame-based timing
- Automatic sprite rect updates

## Types

| Type | Description |
|------|-------------|
| `SpriteData` | Sprite sheet definition with animations |
| `AnimationDef` | Single animation (frames, timing, loop mode) |
| `AnimatedSprite` | Component for playing animations |
| `LoopMode` | Loop, Once, or PingPong |

## Usage

### Defining Animations (Code)

```rust
use bevy_map_animation::{SpriteData, AnimationDef, LoopMode};

let mut sprite = SpriteData::new("sprites/character.png", 32, 32);

sprite.add_animation("idle", AnimationDef {
    frames: vec![0, 1, 2, 3],
    frame_duration_ms: 200,
    loop_mode: LoopMode::Loop,
});

sprite.add_animation("attack", AnimationDef {
    frames: vec![4, 5, 6, 7, 8],
    frame_duration_ms: 80,
    loop_mode: LoopMode::Once,
});
```

### Defining Animations (Editor)

Use the Animation Editor panel in bevy_map_editor to visually:
1. Import a sprite sheet image
2. Set frame dimensions
3. Define animations by selecting frame ranges
4. Preview animations in real-time

### Playing Animations

```rust
use bevy::prelude::*;
use bevy_map_animation::{AnimatedSprite, SpriteData};

fn play_animation(
    mut query: Query<&mut AnimatedSprite>,
) {
    for mut animated in query.iter_mut() {
        animated.play("idle");
    }
}
```

### Auto-Loading from Maps

Use `AnimatedSpriteHandle` for automatic loading:

```rust
use bevy_map_runtime::AnimatedSpriteHandle;

commands.spawn((
    AnimatedSpriteHandle::new(
        asset_server.load("maps/game.map.json"),
        "player_idle",
    ),
    Transform::default(),
));
```

## Plugin

Add `SpriteAnimationPlugin` for automatic animation updates:

```rust
use bevy_map_animation::SpriteAnimationPlugin;

app.add_plugins(SpriteAnimationPlugin);
```

Note: `MapRuntimePlugin` includes this automatically.

## License

MIT OR Apache-2.0
