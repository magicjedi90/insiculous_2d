//! Menu bar and dropdown menu system for the editor.
//!
//! Provides a standard menu bar with File, Edit, View menus and
//! support for keyboard shortcuts.

use glam::Vec2;
use ui::{Rect, UIContext};

use crate::theme::EditorTheme;

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
                MenuItem::action_with_shortcut("Toggle Grid", "G"),
                MenuItem::action_with_shortcut("Toggle Colliders", "C"),
                MenuItem::separator(),
                MenuItem::action("Reset Layout"),
            ]),
        );

        // Entity menu
        bar.add_menu(
            Menu::new("Entity").with_items(vec![
                MenuItem::action("Create Empty"),
                MenuItem::separator(),
                MenuItem::action("Create Sprite"),
                MenuItem::action("Create Camera"),
                MenuItem::separator(),
                MenuItem::action("Create Static Body"),
                MenuItem::action("Create Dynamic Body"),
                MenuItem::action("Create Kinematic Body"),
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
    /// Runs the three phases in order: layout (pure geometry), title bar
    /// (drawing + click detection), and the open dropdown.
    ///
    /// Returns the label of the clicked menu item, if any.
    pub fn render(&mut self, ui: &mut UIContext, window_width: f32, theme: &EditorTheme) -> Option<String> {
        self.layout_titles(window_width);
        let toggled = self.render_title_bar(ui, theme);
        self.apply_toggle(toggled);
        self.render_open_dropdown(ui, theme)
    }

    /// Phase 1 — compute the bar bounds and each menu title's bounds.
    /// Pure geometry: no drawing, no state transitions.
    fn layout_titles(&mut self, window_width: f32) {
        let height = self.height();
        self.bounds = Rect::new(0.0, 0.0, window_width, height);

        let mut x = self.item_padding;
        for menu in &mut self.menus {
            // Wider than the text for better readability
            let title_width = menu.title.len() as f32 * 10.0 + self.item_padding * 3.0;
            menu.bounds = Rect::new(x, 0.0, title_width, height);
            x += title_width + self.item_spacing;
        }
    }

    /// Phase 2 — draw the bar background and title buttons.
    /// Returns the index of a title that was clicked this frame, if any.
    fn render_title_bar(&mut self, ui: &mut UIContext, theme: &EditorTheme) -> Option<usize> {
        ui.rect(self.bounds, theme.bg_header);

        let mut toggled = None;
        for (index, menu) in self.menus.iter().enumerate() {
            if self.open_menu == Some(index) {
                ui.rect(menu.bounds, theme.menu_open_highlight);
            }

            let id = format!("menu_{}", menu.title);
            if ui.button(id.as_str(), &menu.title, menu.bounds) {
                toggled = Some(index);
            }
        }
        toggled
    }

    /// Phase 3 — apply a title click: open the clicked menu, or close it if
    /// it was already open.
    fn apply_toggle(&mut self, toggled: Option<usize>) {
        if let Some(index) = toggled {
            self.open_menu = if self.open_menu == Some(index) {
                None
            } else {
                Some(index)
            };
        }
    }

    /// Phase 4 — render the dropdown for the open menu (if any) and handle
    /// item clicks. A click on an item — or a press outside the dropdown —
    /// closes the menu; an item click returns its label.
    fn render_open_dropdown(&mut self, ui: &mut UIContext, theme: &EditorTheme) -> Option<String> {
        let open_index = self.open_menu?;
        let menu = &self.menus[open_index];
        let dropdown_bounds = Self::dropdown_bounds(menu, menu.bounds);

        if ui.mouse_just_pressed()
            && Self::should_close_on_press(ui.mouse_pos(), dropdown_bounds, menu.bounds)
        {
            self.open_menu = None;
            return None;
        }

        // Overlay: render on top of panels/toolbar and swallow clicks so
        // they don't pass through to widgets underneath.
        ui.begin_overlay(dropdown_bounds);
        let clicked = Self::render_dropdown_static(ui, menu, dropdown_bounds, theme);
        ui.end_overlay();

        if clicked.is_some() {
            self.open_menu = None;
        }
        clicked
    }

    /// Compute the dropdown bounds for a menu anchored below its title.
    fn dropdown_bounds(menu: &Menu, anchor: Rect) -> Rect {
        let dropdown_height = menu.items.len() as f32 * DROPDOWN_ITEM_HEIGHT + 8.0;
        Rect::new(
            anchor.x,
            anchor.y + anchor.height,
            DROPDOWN_WIDTH,
            dropdown_height,
        )
    }

    /// Whether a mouse press at `mouse` should close the open dropdown.
    ///
    /// Presses inside the dropdown are item interactions; presses on the open
    /// menu's own title must NOT close here because the title's click fires on
    /// mouse *release* — closing on press would make the release re-toggle the
    /// menu open (close/reopen flicker). Pressing a different title closes
    /// here, then that title's release opens its menu.
    fn should_close_on_press(mouse: Vec2, dropdown: Rect, title: Rect) -> bool {
        !dropdown.contains(mouse) && !title.contains(mouse)
    }

    /// Render a dropdown menu (static method to avoid borrow issues).
    fn render_dropdown_static(ui: &mut UIContext, menu: &Menu, dropdown_bounds: Rect, theme: &EditorTheme) -> Option<String> {
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
                        ui.label_styled(shortcut, shortcut_pos, theme.shortcut_hint, theme.fonts.small);
                    }

                    y += DROPDOWN_ITEM_HEIGHT;
                }
                MenuItem::Separator => {
                    let sep_y = y + DROPDOWN_ITEM_HEIGHT / 2.0;
                    ui.line(
                        Vec2::new(dropdown_bounds.x + 8.0, sep_y),
                        Vec2::new(dropdown_bounds.x + dropdown_bounds.width - 8.0, sep_y),
                        theme.menu_separator,
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

    #[test]
    fn test_layout_titles_sets_bar_bounds_to_window_width() {
        let mut bar = MenuBar::editor_default();
        bar.layout_titles(1280.0);
        assert_eq!(bar.bounds.width, 1280.0);
        assert_eq!(bar.bounds.height, bar.height());
    }

    #[test]
    fn test_layout_titles_do_not_overlap() {
        let mut bar = MenuBar::editor_default();
        bar.layout_titles(1280.0);

        for pair in bar.menus.windows(2) {
            let left = pair[0].bounds;
            let right = pair[1].bounds;
            assert!(
                left.x + left.width <= right.x,
                "menu '{}' overlaps '{}'",
                pair[0].title,
                pair[1].title
            );
        }
    }

    #[test]
    fn test_layout_title_width_scales_with_title_length() {
        let mut bar = MenuBar::new();
        bar.add_menu(Menu::new("File"));
        bar.add_menu(Menu::new("MuchLongerTitle"));
        bar.layout_titles(800.0);

        assert!(bar.menus[1].bounds.width > bar.menus[0].bounds.width);
    }

    #[test]
    fn test_should_close_on_press_geometry() {
        let dropdown = Rect::new(8.0, 24.0, 200.0, 100.0);
        let title = Rect::new(8.0, 0.0, 60.0, 24.0);

        // Press far away → close
        assert!(MenuBar::should_close_on_press(Vec2::new(500.0, 300.0), dropdown, title));
        // Press inside the dropdown → keep open (item interaction)
        assert!(!MenuBar::should_close_on_press(Vec2::new(50.0, 60.0), dropdown, title));
        // Press on the open menu's own title → keep open (release toggles it)
        assert!(!MenuBar::should_close_on_press(Vec2::new(30.0, 10.0), dropdown, title));
    }

    /// Build an InputHandler with the mouse at the given position, pressed.
    fn pressed_mouse_at(x: f32, y: f32) -> input::InputHandler {
        let mut input = input::InputHandler::new();
        input.mouse_mut().update_position(x, y);
        input.mouse_mut().handle_button_press(winit::event::MouseButton::Left);
        input
    }

    #[test]
    fn test_outside_press_closes_open_menu() {
        let mut bar = MenuBar::editor_default();
        bar.layout_titles(1280.0);
        bar.open_menu = Some(0);

        let mut ui = UIContext::new();
        let input = pressed_mouse_at(900.0, 400.0); // far from menu + dropdown
        ui.begin_frame(&input, Vec2::new(1280.0, 720.0));
        let clicked = bar.render(&mut ui, 1280.0, &crate::theme::EditorTheme::default());
        ui.end_frame();

        assert!(clicked.is_none());
        assert!(bar.open_menu.is_none(), "press outside must close the dropdown");
    }

    #[test]
    fn test_press_on_open_title_keeps_menu_open_until_release() {
        let mut bar = MenuBar::editor_default();
        bar.layout_titles(1280.0);
        bar.open_menu = Some(0);
        let title_center = bar.menus[0].bounds.center();

        let mut ui = UIContext::new();
        let input = pressed_mouse_at(title_center.x, title_center.y);
        ui.begin_frame(&input, Vec2::new(1280.0, 720.0));
        bar.render(&mut ui, 1280.0, &crate::theme::EditorTheme::default());
        ui.end_frame();

        // The close happens via the title's click (on release), not on press —
        // closing here too would cause a close/reopen flicker.
        assert_eq!(bar.open_menu, Some(0));
    }

    #[test]
    fn test_open_dropdown_renders_in_overlay_band_and_blocks_input() {
        let mut bar = MenuBar::editor_default();
        bar.layout_titles(1280.0);
        bar.open_menu = Some(0);

        let mut ui = UIContext::new();
        ui.begin_frame(&input::InputHandler::new(), Vec2::new(1280.0, 720.0));
        bar.render(&mut ui, 1280.0, &crate::theme::EditorTheme::default());

        // Dropdown draws above the base UI band (950+)
        let max_depth = ui
            .draw_list()
            .commands()
            .iter()
            .map(|c| c.depth())
            .fold(f32::MIN, f32::max);
        assert!(max_depth >= 950.0, "dropdown must render in the overlay band, got {max_depth}");

        // Mouse input under the dropdown is swallowed for later widgets
        let dropdown = MenuBar::dropdown_bounds(&bar.menus[0], bar.menus[0].bounds);
        assert!(ui.is_input_blocked_at(dropdown.center()));

        // Overlay mode was properly closed: subsequent draws are base band
        let before = ui.draw_list().len();
        ui.rect(Rect::new(0.0, 0.0, 10.0, 10.0), ui::Color::WHITE);
        assert!(ui.draw_list().commands()[before].depth() < 950.0);
        ui.end_frame();
    }

    #[test]
    fn test_apply_toggle_opens_and_closes() {
        let mut bar = MenuBar::editor_default();

        bar.apply_toggle(Some(1));
        assert_eq!(bar.open_menu, Some(1));

        // Clicking the same title again closes it
        bar.apply_toggle(Some(1));
        assert!(bar.open_menu.is_none());

        // No click leaves state unchanged
        bar.apply_toggle(Some(2));
        bar.apply_toggle(None);
        assert_eq!(bar.open_menu, Some(2));
    }
}
