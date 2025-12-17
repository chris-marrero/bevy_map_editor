//! Collision data structures for tiles and entities
//!
//! This module provides the core collision types used by the editor and runtime:
//! - `CollisionShape` - Shape types (None, Full, Rectangle, Circle, Polygon)
//! - `CollisionData` - Full collision configuration including layers and one-way
//! - `PhysicsBody` - Body type (Static, Dynamic, Kinematic)
//! - `OneWayDirection` - One-way platform direction

use serde::{Deserialize, Serialize};

/// Collision shape types supported by the editor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum CollisionShape {
    /// No collision
    None,
    /// Full tile/entity bounding box
    Full,
    /// Rectangle with optional offset and size
    Rectangle {
        /// Offset from center [x, y] (0-1 normalized for tiles)
        #[serde(default)]
        offset: [f32; 2],
        /// Size [width, height] (0-1 normalized for tiles)
        #[serde(default = "default_full_size")]
        size: [f32; 2],
    },
    /// Circle collider
    Circle {
        /// Center offset [x, y] (0-1 normalized)
        #[serde(default)]
        offset: [f32; 2],
        /// Radius (0-1 normalized, 0.5 = full tile)
        #[serde(default = "default_radius")]
        radius: f32,
    },
    /// Polygon collider (convex hull)
    Polygon {
        /// Vertices in local coordinates (0-1 normalized)
        points: Vec<[f32; 2]>,
    },
}

fn default_full_size() -> [f32; 2] {
    [1.0, 1.0]
}

fn default_radius() -> f32 {
    0.5
}

impl Default for CollisionShape {
    fn default() -> Self {
        CollisionShape::None
    }
}

impl CollisionShape {
    /// Check if this shape has collision
    pub fn has_collision(&self) -> bool {
        !matches!(self, CollisionShape::None)
    }

    /// Create a rectangle shape with offset and size
    pub fn rectangle(offset: [f32; 2], size: [f32; 2]) -> Self {
        CollisionShape::Rectangle { offset, size }
    }

    /// Create a circle shape with offset and radius
    pub fn circle(offset: [f32; 2], radius: f32) -> Self {
        CollisionShape::Circle { offset, radius }
    }

    /// Create a polygon shape from points
    pub fn polygon(points: Vec<[f32; 2]>) -> Self {
        CollisionShape::Polygon { points }
    }

    /// Get the display name of this shape type
    pub fn name(&self) -> &'static str {
        match self {
            CollisionShape::None => "None",
            CollisionShape::Full => "Full",
            CollisionShape::Rectangle { .. } => "Rectangle",
            CollisionShape::Circle { .. } => "Circle",
            CollisionShape::Polygon { .. } => "Polygon",
        }
    }
}

/// Physics body type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PhysicsBody {
    #[default]
    Static,
    Dynamic,
    Kinematic,
}

impl PhysicsBody {
    /// Get the display name of this body type
    pub fn name(&self) -> &'static str {
        match self {
            PhysicsBody::Static => "Static",
            PhysicsBody::Dynamic => "Dynamic",
            PhysicsBody::Kinematic => "Kinematic",
        }
    }
}

/// Direction for one-way platforms
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum OneWayDirection {
    /// No one-way behavior (solid from all sides)
    #[default]
    None,
    /// Pass through from below (standard platformer)
    Top,
    /// Pass through from above
    Bottom,
    /// Pass through from right
    Left,
    /// Pass through from left
    Right,
}

impl OneWayDirection {
    /// Check if this is a one-way platform
    pub fn is_one_way(&self) -> bool {
        !matches!(self, OneWayDirection::None)
    }

    /// Get the display name of this direction
    pub fn name(&self) -> &'static str {
        match self {
            OneWayDirection::None => "None (Solid)",
            OneWayDirection::Top => "Top (Pass from below)",
            OneWayDirection::Bottom => "Bottom (Pass from above)",
            OneWayDirection::Left => "Left (Pass from right)",
            OneWayDirection::Right => "Right (Pass from left)",
        }
    }
}

/// Collision data for a tile or entity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollisionData {
    /// The collision shape
    #[serde(default)]
    pub shape: CollisionShape,
    /// Physics body type (Static, Dynamic, Kinematic)
    #[serde(default)]
    pub body_type: PhysicsBody,
    /// One-way platform direction (None = solid from all sides)
    #[serde(default)]
    pub one_way: OneWayDirection,
    /// Collision layer (0-31)
    #[serde(default)]
    pub layer: u8,
    /// Collision mask (which layers to collide with)
    #[serde(default = "default_mask")]
    pub mask: u32,
}

fn default_mask() -> u32 {
    0xFFFFFFFF
}

impl CollisionData {
    /// Create new collision data with a shape
    pub fn new(shape: CollisionShape) -> Self {
        Self {
            shape,
            body_type: PhysicsBody::Static,
            one_way: OneWayDirection::None,
            layer: 0,
            mask: default_mask(),
        }
    }

    /// Create collision data for a full solid tile
    pub fn full() -> Self {
        Self::new(CollisionShape::Full)
    }

    /// Create collision data with no collision
    pub fn none() -> Self {
        Self::new(CollisionShape::None)
    }

    /// Check if this has collision
    pub fn has_collision(&self) -> bool {
        self.shape.has_collision()
    }

    /// Check if this is a one-way platform
    pub fn is_one_way(&self) -> bool {
        self.one_way.is_one_way()
    }

    /// Set the shape
    pub fn with_shape(mut self, shape: CollisionShape) -> Self {
        self.shape = shape;
        self
    }

    /// Set the body type
    pub fn with_body_type(mut self, body_type: PhysicsBody) -> Self {
        self.body_type = body_type;
        self
    }

    /// Set the one-way direction
    pub fn with_one_way(mut self, one_way: OneWayDirection) -> Self {
        self.one_way = one_way;
        self
    }

    /// Set the collision layer
    pub fn with_layer(mut self, layer: u8) -> Self {
        self.layer = layer;
        self
    }

    /// Set the collision mask
    pub fn with_mask(mut self, mask: u32) -> Self {
        self.mask = mask;
        self
    }

    /// Check if this collision data is effectively empty (no collision)
    pub fn is_empty(&self) -> bool {
        !self.has_collision()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_shape_default() {
        let shape = CollisionShape::default();
        assert!(!shape.has_collision());
        assert!(matches!(shape, CollisionShape::None));
    }

    #[test]
    fn test_collision_data_full() {
        let data = CollisionData::full();
        assert!(data.has_collision());
        assert!(!data.is_one_way());
        assert_eq!(data.layer, 0);
        assert_eq!(data.mask, 0xFFFFFFFF);
    }

    #[test]
    fn test_one_way_direction() {
        assert!(!OneWayDirection::None.is_one_way());
        assert!(OneWayDirection::Top.is_one_way());
        assert!(OneWayDirection::Bottom.is_one_way());
        assert!(OneWayDirection::Left.is_one_way());
        assert!(OneWayDirection::Right.is_one_way());
    }

    #[test]
    fn test_collision_shape_serialization() {
        let shape = CollisionShape::Rectangle {
            offset: [0.1, 0.2],
            size: [0.8, 0.6],
        };
        let json = serde_json::to_string(&shape).unwrap();
        let parsed: CollisionShape = serde_json::from_str(&json).unwrap();
        assert_eq!(shape, parsed);
    }
}
