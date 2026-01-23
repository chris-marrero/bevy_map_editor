# bevy_map_codegen

Rust code generation from map editor data.

Part of [bevy_map_editor](https://github.com/jbuehler23/bevy_map_editor).

## Features

| Feature | Description |
|---------|-------------|
| Entity Structs | Generate Bevy component structs from entity type definitions |
| Enum Definitions | Generate Rust enums from schema enum types |
| Behavior Stubs | Create placeholder functions for entity behaviors |
| Movement Systems | Generate input-driven movement code from Input profiles |

## Usage

This crate is used internally by bevy_map_editor. Generated code is written to your game project's `src/generated/` directory.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
