use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The type of a node in the document tree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Frame,
    Text,
    Rectangle,
    Ellipse,
    Group,
}

/// CSS-like layout properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Layout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Visual style properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Style {
    /// Fill color as hex string, e.g. "#ff0000"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    /// Stroke color as hex string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    /// Stroke width in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,
    /// Corner radius in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f64>,
    /// Opacity from 0.0 to 1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
}

/// Text-specific properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Typography {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f64>,
}

/// A node in the document tree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub node_type: NodeType,
    pub parent_id: Option<Uuid>,
    pub children: Vec<Uuid>,
    pub layout: Layout,
    pub style: Style,
    pub typography: Typography,
}

impl Node {
    pub fn new(name: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            node_type,
            parent_id: None,
            children: Vec::new(),
            layout: Layout::default(),
            style: Style::default(),
            typography: Typography::default(),
        }
    }

    /// Create a node with a specific ID (useful for tests and deserialization).
    pub fn with_id(id: Uuid, name: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id,
            name: name.into(),
            node_type,
            parent_id: None,
            children: Vec::new(),
            layout: Layout::default(),
            style: Style::default(),
            typography: Typography::default(),
        }
    }
}
