// Marching ants selection border shader
// Renders an animated dashed border around selected tiles

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct MarchingAntsUniform {
    time: f32,
    border_width: f32,
    dash_length: f32,
    tile_size: f32,
}

@group(2) @binding(0) var<uniform> material: MarchingAntsUniform;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // Calculate distance from each edge (0 = at edge, 0.5 = center)
    let dist_left = uv.x;
    let dist_right = 1.0 - uv.x;
    let dist_bottom = uv.y;
    let dist_top = 1.0 - uv.y;

    // Find minimum distance to any edge
    let min_dist = min(min(dist_left, dist_right), min(dist_bottom, dist_top));

    // Border threshold - thinner border for cleaner look
    // border_width is in pixels, tile_size converts to UV space
    let border_threshold = material.border_width / material.tile_size;

    // Only draw on the border area
    if min_dist > border_threshold {
        discard;
    }

    // Calculate position along the perimeter for marching animation
    // This creates a continuous line that wraps around the rectangle
    var perimeter_pos: f32 = 0.0;

    // Determine which edge we're on and calculate position along perimeter
    if dist_bottom <= border_threshold {
        // Bottom edge - left to right
        perimeter_pos = uv.x;
    } else if dist_right <= border_threshold {
        // Right edge - bottom to top
        perimeter_pos = 1.0 + uv.y;
    } else if dist_top <= border_threshold {
        // Top edge - right to left
        perimeter_pos = 2.0 + (1.0 - uv.x);
    } else if dist_left <= border_threshold {
        // Left edge - top to bottom
        perimeter_pos = 3.0 + (1.0 - uv.y);
    }

    // Animate the pattern along the perimeter
    // dash_length controls how many dashes per tile
    let animated_pos = perimeter_pos * material.dash_length - material.time * 3.0;

    // Create alternating black/white pattern
    let pattern = sin(animated_pos * 3.14159);

    if pattern > 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black
    } else {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0); // White
    }
}
