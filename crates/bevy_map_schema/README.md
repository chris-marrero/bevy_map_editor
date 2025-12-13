# bevy_map_schema

Schema validation for entity properties in the bevy_map_editor ecosystem.

Part of [bevy_map_editor](https://github.com/jbuehler23/bevy_map_editor).

## Features

- JSON-based schema definitions
- Type validation (String, Int, Float, Bool, Color, Enum)
- Required/optional properties with defaults
- Custom enum definitions
- Numeric constraints (min/max)

## Property Types

| Type | Description |
|------|-------------|
| `string` | Text value |
| `int` | Integer number |
| `float` | Decimal number |
| `bool` | True/false |
| `color` | Hex color (#RRGGBB) |
| `enum` | Custom enum type |

## Schema Format

Define entity types in your map project:

```json
{
  "schema": {
    "data_types": {
      "Enemy": {
        "color": "#FF0000",
        "placeable": true,
        "marker_size": 16,
        "properties": [
          { "name": "name", "type": "string", "required": true },
          { "name": "health", "type": "int", "default": 100 },
          { "name": "speed", "type": "float", "default": 1.0 },
          { "name": "aggressive", "type": "bool", "default": true }
        ]
      }
    },
    "enums": {
      "Direction": ["North", "South", "East", "West"]
    }
  }
}
```

## Usage

```rust
use bevy_map_schema::{Schema, TypeDef, PropertyDef};

// Schemas are typically loaded from map project files
// The editor validates properties against the schema
```

## Integration

The schema is embedded in `.map.json` files and used by:
- **Editor**: Shows appropriate UI controls for each property type
- **Runtime**: Validates entity properties when loading maps

## License

MIT OR Apache-2.0
