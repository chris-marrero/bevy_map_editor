//! Legacy 47-tile blob autotile support
//!
//! This module provides backward compatibility for the old 47-tile blob format
//! used in earlier versions.

use crate::config::LegacyTerrainType;

/// Legacy neighbor direction flags for bitmask calculation
pub mod neighbors {
    pub const N: u8 = 0b0000_0001;  // North
    pub const NE: u8 = 0b0000_0010; // Northeast (corner)
    pub const E: u8 = 0b0000_0100;  // East
    pub const SE: u8 = 0b0000_1000; // Southeast (corner)
    pub const S: u8 = 0b0001_0000;  // South
    pub const SW: u8 = 0b0010_0000; // Southwest (corner)
    pub const W: u8 = 0b0100_0000;  // West
    pub const NW: u8 = 0b1000_0000; // Northwest (corner)
}

/// Apply corner optimization to a bitmask (legacy)
pub fn optimize_bitmask(bitmask: u8) -> u8 {
    use neighbors::*;

    let mut result = bitmask;

    // NW corner requires N and W
    if (bitmask & (N | W)) != (N | W) {
        result &= !NW;
    }
    // NE corner requires N and E
    if (bitmask & (N | E)) != (N | E) {
        result &= !NE;
    }
    // SE corner requires S and E
    if (bitmask & (S | E)) != (S | E) {
        result &= !SE;
    }
    // SW corner requires S and W
    if (bitmask & (S | W)) != (S | W) {
        result &= !SW;
    }

    result
}

/// Calculate the neighbor bitmask for a tile (legacy)
pub fn calculate_bitmask<F>(x: i32, y: i32, is_same_terrain: F) -> u8
where
    F: Fn(i32, i32) -> bool,
{
    use neighbors::*;

    let mut bitmask = 0u8;

    if is_same_terrain(x, y - 1) {
        bitmask |= N;
    }
    if is_same_terrain(x + 1, y - 1) {
        bitmask |= NE;
    }
    if is_same_terrain(x + 1, y) {
        bitmask |= E;
    }
    if is_same_terrain(x + 1, y + 1) {
        bitmask |= SE;
    }
    if is_same_terrain(x, y + 1) {
        bitmask |= S;
    }
    if is_same_terrain(x - 1, y + 1) {
        bitmask |= SW;
    }
    if is_same_terrain(x - 1, y) {
        bitmask |= W;
    }
    if is_same_terrain(x - 1, y - 1) {
        bitmask |= NW;
    }

    optimize_bitmask(bitmask)
}

/// Apply autotiling to a region of tiles (legacy)
#[allow(clippy::too_many_arguments)]
pub fn apply_autotile_to_region<F>(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    region_x: i32,
    region_y: i32,
    region_w: i32,
    region_h: i32,
    terrain: &LegacyTerrainType,
    is_terrain_tile: F,
) where
    F: Fn(Option<u32>) -> bool,
{
    let min_x = (region_x - 1).max(0) as u32;
    let min_y = (region_y - 1).max(0) as u32;
    let max_x = ((region_x + region_w + 1) as u32).min(width);
    let max_y = ((region_y + region_h + 1) as u32).min(height);

    let mut updates: Vec<(usize, u32)> = Vec::new();

    for y in min_y..max_y {
        for x in min_x..max_x {
            let idx = (y * width + x) as usize;
            if let Some(Some(_)) = tiles.get(idx) {
                if is_terrain_tile(tiles[idx]) {
                    let bitmask = calculate_bitmask(x as i32, y as i32, |nx, ny| {
                        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                            return false;
                        }
                        let nidx = (ny as u32 * width + nx as u32) as usize;
                        is_terrain_tile(tiles.get(nidx).copied().flatten())
                    });
                    updates.push((idx, terrain.get_tile(bitmask)));
                }
            }
        }
    }

    for (idx, new_tile) in updates {
        tiles[idx] = Some(new_tile);
    }
}

/// Paint a single tile with autotiling and update neighbors (legacy)
pub fn paint_autotile<F>(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    terrain: &LegacyTerrainType,
    is_terrain_tile: F,
) where
    F: Fn(Option<u32>) -> bool + Copy,
{
    let idx = (y * width + x) as usize;
    if idx < tiles.len() {
        tiles[idx] = Some(terrain.base_tile);
    }

    apply_autotile_to_region(
        tiles,
        width,
        height,
        x as i32,
        y as i32,
        1,
        1,
        terrain,
        is_terrain_tile,
    );
}

/// Erase a tile and update autotiling for neighbors (legacy)
pub fn erase_autotile<F>(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    terrain: &LegacyTerrainType,
    is_terrain_tile: F,
) where
    F: Fn(Option<u32>) -> bool + Copy,
{
    let idx = (y * width + x) as usize;
    if idx < tiles.len() {
        tiles[idx] = None;
    }

    apply_autotile_to_region(
        tiles,
        width,
        height,
        x as i32,
        y as i32,
        1,
        1,
        terrain,
        is_terrain_tile,
    );
}
