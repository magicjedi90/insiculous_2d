//! Menu bar and dropdown menu system for the editor.
//!
//! Provides a standard menu bar with File, Edit, View menus and
//! support for keyboard shortcuts.

use glam::Vec2;
use ui::{Color, Rect, UIContext};

/// Menu dropdown layout constants
const DROPDOWN_ITEM_HEIGHT: f32 = 24.0;
const DROPDOWN_ITEM_PADDING: f32 = 8.0;
const DROPDOWN_WIDTH: f32 = 200.0;

/// A single menu item (can be an action or separator).
#[derive(Debug, Clone)]
pub enum MenuItem {
    /// A clickable action
    Action {
        /// Display label
        label: String,
        /// Keyboard shortcut hint (e.g., "Ctrl+S")
        shortcut: Option<String>,
        /// Whether the item is enabled
        enabled: bool,
    },
    /// A separator line
    Separator,
    /// A submenu
    Submenu {
        /// Display label
        label: String,
        /// Submenu items
        items: Vec<MenuItem>,
    },
}

impl MenuItem {
    /// Create a new action item.
    pub fn action(label: impl Into<String>) -> Self {
        MenuItem::Action {
            label: label.into(),
            shortcut: None,
            enabled: true,
        }
    }

    /// Create a new action item with a shortcut.
    pub fn action_with_shortcut(label: impl Into<String>, shortcut: impl Into<String>) -> Self {
        MenuItem::Action {
            label: label.into(),
            shortcut: Some(shortcut.into()),
            enabled: true,
        }
    }

    /// Create a separator.
    pub fn separator() -> Self {
        MenuItem::Separator
    }

    /// Create a submenu.
    pub fn submenu(label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        MenuItem::Submenu {
            label: label.into(),
            items,
        }
    }

    /// Set whether the item is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        if let MenuItem::Action { enabled: e, .. } = &mut self {
            *e = enabled;
        }
        self
    }

    /// Get the label for this item (if any).
    pub fn label(&self) -> Option<&str> {
        match self {
            MenuItem::Action { label, .. } => Some(label),
            MenuItem::Submenu { label, .. } => Some(label),
            MenuItem::Separator => None,
        }
    }
}

/// A dropdown menu containing menu items.
#[derive(Debug, Clone)]
pub struct Menu {
    /// Menu title
    pub title: String,
    /// Menu items
    pub items: Vec<MenuItem>,
    /// Whether the menu is currently open
    pub open: bool,
    /// Menu bounds (set during render)
    pub bounds: Rect,
}

impl Menu {
    /// Create a new menu.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
            open: false,
            bounds: Rect::default(),
        }
    }

    /// Add an item to the menu.
    pub fn add_item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple items to the menu.
    pub fn with_items(mut self, items: Vec<MenuItem>) -> Self {
        self.items = items;
        self
    }

    /// Get the number of visible items (excluding separators for sizing).
    pub fn visible_item_count(&self) -> usize {
        self.items.iter().filter(|i| !matches!(i, MenuItem::Separator)).count()
    }
}

/// The main menu bar at the top of the editor.
#[derive(Debug, Clone)]
pub struct MenuBar {
    /// Menus in the menu bar
    menus: Vec<Menu>,
    /// Menu bar bounds
    bounds: Rect,
    /// Menu item spacing
    item_spacing: f32,
    /// Menu item padding
    item_padding: f32,
    /// Currently open menu index
    open_menu: Option<usize>,
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuBar {
    /// Create a new empty menu bar.
    pub fn new() -> Self {
        Self {
            menus: Vec::new(),
            bounds: Rect::default(),
            item_spacing: 16.0,
            item_padding: 8.0,
            open_menu: None,
        }
    }

    /// Create a default editor menu bar with File, Edit, View menus.
    pub fn editor_default() -> Self {
        let mut bar = Self::new();

        // File menu
        bar.add_menu(
            Menu::new("File").with_items(vec![
                MenuItem::action_with_shortcut("New Scene", "Ctrl+N"),
                MenuItem::action_with_shortcut("Open Scene...", "Ctrl+O"),
                MenuItem::separator(),
                MenuItem::action_with_shortcut("Save", "Ctrl+S"),
                MenuItem::action_with_shortcut("Save As...", "Ctrl+Shift+S"),
                MenuItem::separator(),
                MenuItem::action("Exit"),
            ]),
        );

        // Edit menu
        bar.add_menu(
            Menu::new("Edit").with_items(vec![
                MenuItem::action_with_shortcut("Undo", "Ctrl+Z"),
                MenuItem::action_with_shortcut("Redo", "Ctrl+Y"),
                MenuItem::separator(),
                MenuItem::action_with_shortcut("Cut", "Ctrl+X"),
                MenuItem::action_with_shortcut("Copy", "Ctrl+C"),
                MenuItem::action_with_shortcut("Paste", "Ctrl+V"),
                MenuItem::separator(),
                MenuItem::action_with_shortcut("Delete", "Del"),
                MenuItem::action_with_shortcut("Duplicate", "Ctrl+D"),
            ]),
        );

        // View menu
        bar.add_menu(
            Menu::new("View").with_items(vec![
                MenuItem::action("Scene View"),
                MenuItem::action("Inspector"),
                MenuItem::action("Hierarchy"),
                MenuItem::action("Asset Browser"),
                MenuItem::action("Console"),
                MenuItem::separator(),
                MenuItem::action("Reset Layout"),
            ]),
        );

        // Entity menu
        bar.add_menu(
            Menu::new("Entity").with_items(vec![
                MenuItem::action("Create Empty"),
                MenuItem::submenu(
                    "2D Object",
                    vec![
                        MenuItem::action("Sprite"),
                        MenuItem::action("Camera"),
                        MenuItem::action("Tilemap"),
                    ],
                ),
                MenuItem::submenu(
                    "Physics",
                    vec![
                        MenuItem::action("Static Body"),
                        MenuItem::action("Dynamic Body"),
                        MenuItem::action("Kinematic Body"),
                    ],
                ),
            ]),
        );

        bar
    }

    /// Add a menu to the menu bar.
    pub fn add_menu(&mut self, menu: Menu) {
        self.menus.push(menu);
    }

    /// Set the bounds for the menu bar.
    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    /// Get the menu bar height.
    pub fn height(&self) -> f32 {
        24.0
    }

    /// Render the menu bar and handle interactions.
    ///
    /// Returns the label of the clicked menu item, if any.
    pub fn render(&mut self, ui: &mut UIContext, window_width: f32) -> Option<String> {
        let mut clicked_item = None;
        let height = self.height();

        // Update bounds
        self.bounds = Rect::new(0.0, 0.0, window_width, height);

        // Draw menu bar background
        ui.rect(self.bounds, Color::new(0.12, 0.12, 0.12, 1.0));

        // First pass: update menu bounds and handle interactions
        let mut x = self.item_padding;
        let mut new_open_menu = self.open_menu;

        for index in 0..self.menus.len() {
            let menu = &mut self.menus[index];

            // Calculate menu title bounds (wider for better readability)
            let title_width = menu.title.len() as f32 * 10.0 + self.item_padding * 3.0;
            let menu_bounds = Rect::new(x, 0.0, title_width, height);
            menu.bounds = menu_bounds;

            // Check for menu title click
            let id = format!("menu_{}", menu.title);
            let is_open = self.open_menu == Some(index);

            // Highlight if open or hovered
            if is_open {
                ui.rect(menu_bounds, Color::new(0.2, 0.2, 0.2, 1.0));
            }

            if ui.button(id.as_str(), &menu.title, menu_bounds) {
                if is_open {
                    new_open_menu = None;
                } else {
                    new_open_menu = Some(index);
                }
            }

            x += title_width + self.item_spacing;
        }

        self.open_menu = new_open_menu;

        // Second pass: render dropdown for open menu
        if let Some(open_index) = self.open_menu {
            let menu = &self.menus[open_index];
            let menu_bounds = menu.bounds;

            if let Some(item) = Self::render_dropdown_static(ui, menu, menu_bounds) {
                clicked_item = Some(item);
                self.open_menu = None;
            }
        }

        clicked_item
    }

    /// Render a dropdown menu (static method to avoid borrow issues).
    fn render_dropdown_static(ui: &mut UIContext, menu: &Menu, anchor: Rect) -> Option<String> {
        let item_count = menu.items.len();
        let dropdown_height = item_count as f32 * DROPDOWN_ITEM_HEIGHT + 8.0;
        let dropdown_bounds = Rect::new(
            anchor.x,
            anchor.y + anchor.height,
            DROPDOWN_WIDTH,
            dropdown_height,
        );

        // Draw dropdown background
        ui.panel(dropdown_bounds);

        // Draw items
        let mut y = dropdown_bounds.y + 4.0;
        for (i, item) in menu.items.iter().enumerate() {
            match item {
                MenuItem::Action { label, shortcut, enabled } => {
                    let item_bounds = Rect::new(
                        dropdown_bounds.x + 4.0,
                        y,
                        dropdown_bounds.width - 8.0,
                        DROPDOWN_ITEM_HEIGHT,
                    );

                    let id = format!("menu_item_{}_{}", menu.title, i);

                    if ui.button_styled(id.as_str(), label, item_bounds, *enabled) {
                        return Some(label.clone());
                    }

                    // Draw shortcut if present
                    if let Some(shortcut) = shortcut {
                        let shortcut_pos = Vec2::new(
                            item_bounds.x + item_bounds.width - DROPDOWN_ITEM_PADDING - shortcut.len() as f32 * 6.0,
                            item_bounds.center().y,
                        );
                        ui.label_styled(shortcut, shortcut_pos, Color::new(0.5, 0.5, 0.5, 1.0), 12.0);
                    }

                    y += DROPDOWN_ITEM_HEIGHT;
                }
                MenuItem::Separator => {
                    let sep_y = y + DROPDOWN_ITEM_HEIGHT / 2.0;
                    ui.line(
                        Vec2::new(dropdown_bounds.x + 8.0, sep_y),
                        Vec2::new(dropdown_bounds.x + dropdown_bounds.width - 8.0, sep_y),
                        Color::new(0.3, 0.3, 0.3, 1.0),
                        1.0,
                    );
                    y += DROPDOWN_ITEM_HEIGHT;
                }
                MenuItem::Submenu { label, .. } => {
                    // For now, just render the label with an arrow indicator
                    let item_bounds = Rect::new(
                        dropdown_bounds.x + 4.0,
                        y,
                        dropdown_bounds.width - 8.0,
                        DROPDOWN_ITEM_HEIGHT,
                    );

                    let id = format!("menu_submenu_{}_{}", menu.title, i);
                    ui.button(id.as_str(), &format!("{} >", label), item_bounds);
                    y += DROPDOWN_ITEM_HEIGHT;
                }
            }
        }

        None
    }

    /// Close any open menus.
    pub fn close_all(&mut self) {
        self.open_menu = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_action() {
        let item = MenuItem::action("Test");
        assert_eq!(item.label(), Some("Test"));
    }

    #[test]
    fn test_menu_item_with_shortcut() {
        let item = MenuItem::action_with_shortcut("Save", "Ctrl+S");
        if let MenuItem::Action { label, shortcut, enabled } = item {
            assert_eq!(label, "Save");
            assert_eq!(shortcut, Some("Ctrl+S".to_string()));
            assert!(enabled);
        } else {
            panic!("Expected Action variant");
        }
    }

    #[test]
    fn test_menu_item_separator() {
        let item = MenuItem::separator();
        assert!(matches!(item, MenuItem::Separator));
        assert!(item.label().is_none());
    }

    #[test]
    fn test_menu_item_submenu() {
        let item = MenuItem::submenu("Create", vec![MenuItem::action("Sprite")]);
        if let MenuItem::Submenu { label, items } = item {
            assert_eq!(label, "Create");
            assert_eq!(items.len(), 1);
        } else {
            panic!("Expected Submenu variant");
        }
    }

    #[test]
    fn test_menu_item_with_enabled() {
        let item = MenuItem::action("Test").with_enabled(false);
        if let MenuItem::Action { enabled, .. } = item {
            assert!(!enabled);
        } else {
            panic!("Expected Action variant");
        }
    }

    #[test]
    fn test_menu_new() {
        let menu = Menu::new("File");
        assert_eq!(menu.title, "File");
        assert!(menu.items.is_empty());
        assert!(!menu.open);
    }

    #[test]
    fn test_menu_add_item() {
        let menu = Menu::new("File")
            .add_item(MenuItem::action("New"))
            .add_item(MenuItem::separator())
            .add_item(MenuItem::action("Exit"));

        assert_eq!(menu.items.len(), 3);
    }

    #[test]
    fn test_menu_with_items() {
        let menu = Menu::new("Edit").with_items(vec![
            MenuItem::action("Undo"),
            MenuItem::action("Redo"),
        ]);

        assert_eq!(menu.items.len(), 2);
    }

    #[test]
    fn test_menu_visible_item_count() {
        let menu = Menu::new("File").with_items(vec![
            MenuItem::action("New"),
            MenuItem::separator(),
            MenuItem::action("Exit"),
        ]);

        // Separators don't count
        assert_eq!(menu.visible_item_count(), 2);
    }

    #[test]
    fn test_menu_bar_new() {
        let bar = MenuBar::new();
        assert!(bar.menus.is_empty());
        assert!(bar.open_menu.is_none());
    }

    #[test]
    fn test_menu_bar_add_menu() {
        let mut bar = MenuBar::new();
        bar.add_menu(Menu::new("File"));
        bar.add_menu(Menu::new("Edit"));

        assert_eq!(bar.menus.len(), 2);
    }

    #[test]
    fn test_menu_bar_editor_default() {
        let bar = MenuBar::editor_default();

        // Should have File, Edit, View, Entity menus
        assert_eq!(bar.menus.len(), 4);
        assert_eq!(bar.menus[0].title, "File");
        assert_eq!(bar.menus[1].title, "Edit");
        assert_eq!(bar.menus[2].title, "View");
        assert_eq!(bar.menus[3].title, "Entity");
    }

    #[test]
    fn test_menu_bar_height() {
        let bar = MenuBar::new();
        assert_eq!(bar.height(), 24.0);
    }

    #[test]
    fn test_menu_bar_close_all() {
        let mut bar = MenuBar::new();
        bar.add_menu(Menu::new("File"));
        bar.open_menu = Some(0);

        bar.close_all();
        assert!(bar.open_menu.is_none());
    }
}
