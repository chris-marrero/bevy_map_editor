# bevy_map_editor

Visual map editor for Bevy 0.17 games. Create tilemaps, place entities, design dialogue trees, and define animations.

Part of [bevy_map_editor](https://github.com/jbuehler23/bevy_map_editor).

<!-- TODO: Add screenshot -->
![Editor Screenshot](../../docs/images/editor_screenshot.png)

## Features

- Project management (new, open, save)
- Multi-level support with hierarchical view
- Layer system (tile and object layers)
- Tileset management with multi-image support
- Terrain painting with autotile
- Entity placement and property editing
- Dialogue tree editor with visual node graph
- Animation/sprite sheet editor
- Undo/redo support
- Keyboard shortcuts

## Usage

```rust
use bevy::prelude::*;
use bevy_map_editor::EditorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EditorPlugin)
        .run();
}
```

## Feature Flags

| Flag | Description |
|------|-------------|
| `runtime` | Enable viewport rendering via bevy_ecs_tilemap (recommended) |

```toml
[dependencies]
bevy_map_editor = { version = "0.1", features = ["runtime"] }
```

## UI Panels

| Panel | Purpose |
|-------|---------|
| Menu Bar | File, Edit, View, Project menus |
| Toolbar | Tool selection (Select, Paint, Erase, Fill, Entity) |
| Project Tree | Hierarchical view of levels, layers, dialogues, animations |
| Inspector | Property editing for selected items |
| Terrain Palette | Terrain set and terrain selection for autotiling |
| Tileset Panel | Tile selection from loaded tilesets |
| Viewport | Map preview and editing canvas |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New Project |
| `Ctrl+O` | Open Project |
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+C` | Copy |
| `Ctrl+V` | Paste |
| `Ctrl+X` | Cut |
| `G` | Toggle Grid |

## License

MIT OR Apache-2.0
