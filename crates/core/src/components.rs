use uuid::Uuid;

use crate::model::{Layout, NodeType, Style, Typography};
use crate::store::{NodeStore, StoreError};

/// A component template definition.
#[derive(Debug, Clone)]
pub struct ComponentTemplate {
    pub name: String,
    pub description: String,
    /// The function that instantiates this template under the given parent.
    /// Returns the IDs of all created nodes (root first).
    pub instantiate: fn(&mut NodeStore, Uuid) -> Result<Vec<Uuid>, StoreError>,
}

/// Built-in component library.
pub struct ComponentLibrary {
    templates: Vec<ComponentTemplate>,
}

impl ComponentLibrary {
    pub fn new() -> Self {
        Self {
            templates: vec![
                ComponentTemplate {
                    name: "stat-card".to_string(),
                    description: "Statistics card with label, value, and change indicator"
                        .to_string(),
                    instantiate: instantiate_stat_card,
                },
                ComponentTemplate {
                    name: "nav-item".to_string(),
                    description: "Navigation menu item with icon placeholder and label".to_string(),
                    instantiate: instantiate_nav_item,
                },
                ComponentTemplate {
                    name: "button".to_string(),
                    description: "Rounded button with label text".to_string(),
                    instantiate: instantiate_button,
                },
                ComponentTemplate {
                    name: "chart-bar".to_string(),
                    description: "Single bar chart element with label".to_string(),
                    instantiate: instantiate_chart_bar,
                },
            ],
        }
    }

    pub fn list(&self) -> &[ComponentTemplate] {
        &self.templates
    }

    pub fn get(&self, name: &str) -> Option<&ComponentTemplate> {
        self.templates.iter().find(|t| t.name == name)
    }
}

impl Default for ComponentLibrary {
    fn default() -> Self {
        Self::new()
    }
}

fn instantiate_stat_card(store: &mut NodeStore, parent_id: Uuid) -> Result<Vec<Uuid>, StoreError> {
    let mut ids = Vec::new();

    // Card container
    let card_id = store.add("Stat Card", NodeType::Frame, parent_id)?;
    {
        let card = store.get_mut(card_id)?;
        card.layout = Layout {
            x: 0.0,
            y: 0.0,
            width: 280.0,
            height: 140.0,
        };
        card.style = Style {
            fill: Some("#ffffff".to_string()),
            corner_radius: Some(12.0),
            ..Style::default()
        };
    }
    ids.push(card_id);

    // Label
    let label_id = store.add("Label", NodeType::Text, card_id)?;
    {
        let label = store.get_mut(label_id)?;
        label.layout = Layout {
            x: 20.0,
            y: 16.0,
            width: 0.0,
            height: 0.0,
        };
        label.typography = Typography {
            content: Some("Metric Name".to_string()),
            font_size: Some(13.0),
            ..Typography::default()
        };
    }
    ids.push(label_id);

    // Value
    let value_id = store.add("Value", NodeType::Text, card_id)?;
    {
        let val = store.get_mut(value_id)?;
        val.layout = Layout {
            x: 20.0,
            y: 48.0,
            width: 0.0,
            height: 0.0,
        };
        val.typography = Typography {
            content: Some("0".to_string()),
            font_size: Some(32.0),
            font_weight: Some(700),
            ..Typography::default()
        };
    }
    ids.push(value_id);

    // Change indicator
    let change_id = store.add("Change", NodeType::Text, card_id)?;
    {
        let change = store.get_mut(change_id)?;
        change.layout = Layout {
            x: 20.0,
            y: 100.0,
            width: 0.0,
            height: 0.0,
        };
        change.typography = Typography {
            content: Some("+0% from last period".to_string()),
            font_size: Some(12.0),
            ..Typography::default()
        };
    }
    ids.push(change_id);

    Ok(ids)
}

fn instantiate_nav_item(store: &mut NodeStore, parent_id: Uuid) -> Result<Vec<Uuid>, StoreError> {
    let mut ids = Vec::new();

    let item_id = store.add("Nav Item", NodeType::Frame, parent_id)?;
    {
        let item = store.get_mut(item_id)?;
        item.layout = Layout {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 40.0,
        };
    }
    ids.push(item_id);

    // Icon placeholder
    let icon_id = store.add("Icon", NodeType::Rectangle, item_id)?;
    {
        let icon = store.get_mut(icon_id)?;
        icon.layout = Layout {
            x: 12.0,
            y: 8.0,
            width: 24.0,
            height: 24.0,
        };
        icon.style = Style {
            fill: Some("#94a3b8".to_string()),
            corner_radius: Some(4.0),
            ..Style::default()
        };
    }
    ids.push(icon_id);

    // Label
    let label_id = store.add("Label", NodeType::Text, item_id)?;
    {
        let label = store.get_mut(label_id)?;
        label.layout = Layout {
            x: 48.0,
            y: 10.0,
            width: 0.0,
            height: 0.0,
        };
        label.typography = Typography {
            content: Some("Menu Item".to_string()),
            font_size: Some(14.0),
            ..Typography::default()
        };
    }
    ids.push(label_id);

    Ok(ids)
}

fn instantiate_button(store: &mut NodeStore, parent_id: Uuid) -> Result<Vec<Uuid>, StoreError> {
    let mut ids = Vec::new();

    let btn_id = store.add("Button", NodeType::Frame, parent_id)?;
    {
        let btn = store.get_mut(btn_id)?;
        btn.layout = Layout {
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 40.0,
        };
        btn.style = Style {
            fill: Some("#3b82f6".to_string()),
            corner_radius: Some(8.0),
            ..Style::default()
        };
    }
    ids.push(btn_id);

    let label_id = store.add("Label", NodeType::Text, btn_id)?;
    {
        let label = store.get_mut(label_id)?;
        label.layout = Layout {
            x: 16.0,
            y: 10.0,
            width: 0.0,
            height: 0.0,
        };
        label.typography = Typography {
            content: Some("Click me".to_string()),
            font_size: Some(14.0),
            font_weight: Some(600),
            ..Typography::default()
        };
    }
    ids.push(label_id);

    Ok(ids)
}

fn instantiate_chart_bar(store: &mut NodeStore, parent_id: Uuid) -> Result<Vec<Uuid>, StoreError> {
    let mut ids = Vec::new();

    let group_id = store.add("Chart Bar", NodeType::Group, parent_id)?;
    {
        let group = store.get_mut(group_id)?;
        group.layout = Layout {
            x: 0.0,
            y: 0.0,
            width: 40.0,
            height: 200.0,
        };
    }
    ids.push(group_id);

    let bar_id = store.add("Bar", NodeType::Rectangle, group_id)?;
    {
        let bar = store.get_mut(bar_id)?;
        bar.layout = Layout {
            x: 0.0,
            y: 50.0,
            width: 40.0,
            height: 150.0,
        };
        bar.style = Style {
            fill: Some("#3b82f6".to_string()),
            corner_radius: Some(4.0),
            ..Style::default()
        };
    }
    ids.push(bar_id);

    let label_id = store.add("Label", NodeType::Text, group_id)?;
    {
        let label = store.get_mut(label_id)?;
        label.layout = Layout {
            x: 4.0,
            y: 180.0,
            width: 0.0,
            height: 0.0,
        };
        label.typography = Typography {
            content: Some("Jan".to_string()),
            font_size: Some(11.0),
            ..Typography::default()
        };
    }
    ids.push(label_id);

    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_has_four_templates() {
        let lib = ComponentLibrary::new();
        assert_eq!(lib.list().len(), 4);
    }

    #[test]
    fn get_template_by_name() {
        let lib = ComponentLibrary::new();
        assert!(lib.get("stat-card").is_some());
        assert!(lib.get("button").is_some());
        assert!(lib.get("nonexistent").is_none());
    }

    #[test]
    fn instantiate_stat_card_creates_4_nodes() {
        let mut store = NodeStore::new();
        let root_id = store.root_id();
        let ids = instantiate_stat_card(&mut store, root_id).unwrap();
        assert_eq!(ids.len(), 4);
        // Card + 3 text children + root = 5 total
        assert_eq!(store.len(), 5);
        // Card is child of root
        assert_eq!(store.get(ids[0]).unwrap().parent_id, Some(root_id));
    }

    #[test]
    fn instantiate_nav_item_creates_3_nodes() {
        let mut store = NodeStore::new();
        let root_id = store.root_id();
        let ids = instantiate_nav_item(&mut store, root_id).unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn instantiate_button_creates_2_nodes() {
        let mut store = NodeStore::new();
        let root_id = store.root_id();
        let ids = instantiate_button(&mut store, root_id).unwrap();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn instantiate_chart_bar_creates_3_nodes() {
        let mut store = NodeStore::new();
        let root_id = store.root_id();
        let ids = instantiate_chart_bar(&mut store, root_id).unwrap();
        assert_eq!(ids.len(), 3);
    }
}
