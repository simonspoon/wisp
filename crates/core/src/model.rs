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

/// Partial layout for edits — only overwrite fields that are present.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PartialLayout {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
}

impl Layout {
    /// Merge a partial layout into this layout, only overwriting present fields.
    pub fn merge(&mut self, partial: &PartialLayout) {
        if let Some(x) = partial.x {
            self.x = x;
        }
        if let Some(y) = partial.y {
            self.y = y;
        }
        if let Some(width) = partial.width {
            self.width = width;
        }
        if let Some(height) = partial.height {
            self.height = height;
        }
    }
}

impl Style {
    /// Merge another style into this one, only overwriting present fields.
    pub fn merge(&mut self, other: &Style) {
        if other.fill.is_some() {
            self.fill.clone_from(&other.fill);
        }
        if other.stroke.is_some() {
            self.stroke.clone_from(&other.stroke);
        }
        if other.stroke_width.is_some() {
            self.stroke_width = other.stroke_width;
        }
        if other.corner_radius.is_some() {
            self.corner_radius = other.corner_radius;
        }
        if other.opacity.is_some() {
            self.opacity = other.opacity;
        }
    }
}

impl Typography {
    /// Merge another typography into this one, only overwriting present fields.
    pub fn merge(&mut self, other: &Typography) {
        if other.content.is_some() {
            self.content.clone_from(&other.content);
        }
        if other.font_family.is_some() {
            self.font_family.clone_from(&other.font_family);
        }
        if other.font_size.is_some() {
            self.font_size = other.font_size;
        }
        if other.font_weight.is_some() {
            self.font_weight = other.font_weight;
        }
        if other.line_height.is_some() {
            self.line_height = other.line_height;
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_layout_merge_only_set_fields() {
        let mut layout = Layout {
            x: 100.0,
            y: 200.0,
            width: 300.0,
            height: 400.0,
        };
        let partial = PartialLayout {
            x: Some(50.0),
            y: None,
            width: None,
            height: None,
        };
        layout.merge(&partial);
        assert_eq!(layout.x, 50.0);
        assert_eq!(layout.y, 200.0);
        assert_eq!(layout.width, 300.0);
        assert_eq!(layout.height, 400.0);
    }

    #[test]
    fn style_merge_only_set_fields() {
        let mut style = Style {
            fill: Some("#ff0000".to_string()),
            stroke: Some("#000000".to_string()),
            stroke_width: Some(2.0),
            corner_radius: Some(8.0),
            opacity: Some(1.0),
        };
        let partial = Style {
            fill: Some("#00ff00".to_string()),
            stroke: None,
            stroke_width: None,
            corner_radius: None,
            opacity: None,
        };
        style.merge(&partial);
        assert_eq!(style.fill.as_deref(), Some("#00ff00"));
        assert_eq!(style.stroke.as_deref(), Some("#000000"));
        assert_eq!(style.stroke_width, Some(2.0));
        assert_eq!(style.corner_radius, Some(8.0));
        assert_eq!(style.opacity, Some(1.0));
    }

    #[test]
    fn typography_merge_only_set_fields() {
        let mut typo = Typography {
            content: Some("Hello".to_string()),
            font_family: Some("Inter".to_string()),
            font_size: Some(16.0),
            font_weight: Some(400),
            line_height: Some(1.5),
        };
        let partial = Typography {
            content: None,
            font_family: None,
            font_size: Some(24.0),
            font_weight: None,
            line_height: None,
        };
        typo.merge(&partial);
        assert_eq!(typo.content.as_deref(), Some("Hello"));
        assert_eq!(typo.font_family.as_deref(), Some("Inter"));
        assert_eq!(typo.font_size, Some(24.0));
        assert_eq!(typo.font_weight, Some(400));
        assert_eq!(typo.line_height, Some(1.5));
    }
}
