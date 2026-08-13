use glam::Vec2;
use ui::Rect;

use crate::layout::HEADER_HEIGHT;

use super::render::resized_size;
use super::*;

#[test]
fn test_dock_panel_content_bounds() {
    let mut panel = DockPanel::new(PanelId::INSPECTOR, "Test", DockPosition::Right);
    panel.bounds = Rect::new(100.0, 50.0, 200.0, 400.0);

    let content = panel.content_bounds();
    assert_eq!(content.x, 100.0);
    assert_eq!(content.y, 74.0); // 50 + 24 header
    assert_eq!(content.width, 200.0);
    assert_eq!(content.height, 376.0); // 400 - 24 header
}

#[test]
fn test_collapsed_panel_has_zero_content_bounds() {
    let mut panel = DockPanel::new(PanelId::INSPECTOR, "Test", DockPosition::Right);
    panel.bounds = Rect::new(100.0, 50.0, 200.0, 400.0);
    panel.collapsed = true;

    let content = panel.content_bounds();
    assert_eq!(content.width, 0.0);
    assert_eq!(content.height, 0.0);
}

#[test]
fn test_collapsible_only_for_edge_positions() {
    let edge = DockPanel::new(PanelId::HIERARCHY, "H", DockPosition::Left);
    assert!(edge.is_collapsible());
    let bottom = DockPanel::new(PanelId::ASSET_BROWSER, "A", DockPosition::Bottom);
    assert!(bottom.is_collapsible());
    let center = DockPanel::new(PanelId::SCENE_VIEW, "S", DockPosition::Center);
    assert!(!center.is_collapsible());
}

#[test]
fn test_effective_size_collapsed_is_header_height() {
    let mut panel = DockPanel::new(PanelId::HIERARCHY, "H", DockPosition::Left).with_size(200.0);
    assert_eq!(panel.effective_size(), 200.0);
    panel.collapsed = true;
    assert_eq!(panel.effective_size(), HEADER_HEIGHT);
}

#[test]
fn test_effective_size_center_ignores_collapsed_flag() {
    let mut panel = DockPanel::new(PanelId::SCENE_VIEW, "S", DockPosition::Center).with_size(300.0);
    panel.collapsed = true;
    assert_eq!(panel.effective_size(), 300.0);
}

#[test]
fn test_dock_area_add_panel() {
    let mut area = DockArea::new();
    area.add_panel(DockPanel::new(PanelId::INSPECTOR, "Inspector", DockPosition::Right));
    area.add_panel(DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left));

    assert_eq!(area.panels().len(), 2);
}

#[test]
fn test_dock_area_get_panel() {
    let mut area = DockArea::new();
    area.add_panel(DockPanel::new(PanelId::INSPECTOR, "Inspector", DockPosition::Right));

    let panel = area.get_panel(PanelId::INSPECTOR);
    assert!(panel.is_some());
    assert_eq!(panel.unwrap().title, "Inspector");

    let missing = area.get_panel(PanelId::HIERARCHY);
    assert!(missing.is_none());
}

#[test]
fn test_dock_area_layout_left() {
    let mut area = DockArea::new();
    area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
    area.add_panel(
        DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left).with_size(200.0),
    );
    area.layout();

    let panel = area.get_panel(PanelId::HIERARCHY).unwrap();
    assert_eq!(panel.bounds.x, 0.0);
    assert_eq!(panel.bounds.y, 0.0);
    assert_eq!(panel.bounds.width, 200.0);
    assert_eq!(panel.bounds.height, 800.0);
}

#[test]
fn test_dock_area_layout_right() {
    let mut area = DockArea::new();
    area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
    area.add_panel(
        DockPanel::new(PanelId::INSPECTOR, "Inspector", DockPosition::Right).with_size(250.0),
    );
    area.layout();

    let panel = area.get_panel(PanelId::INSPECTOR).unwrap();
    assert_eq!(panel.bounds.x, 750.0); // 1000 - 250
    assert_eq!(panel.bounds.y, 0.0);
    assert_eq!(panel.bounds.width, 250.0);
    assert_eq!(panel.bounds.height, 800.0);
}

#[test]
fn test_dock_area_layout_center_gets_remaining() {
    let mut area = DockArea::new();
    area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
    area.add_panel(
        DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left).with_size(200.0),
    );
    area.add_panel(
        DockPanel::new(PanelId::INSPECTOR, "Inspector", DockPosition::Right).with_size(250.0),
    );
    area.add_panel(DockPanel::new(PanelId::SCENE_VIEW, "Scene", DockPosition::Center));
    area.layout();

    let center = area.get_panel(PanelId::SCENE_VIEW).unwrap();
    assert_eq!(center.bounds.x, 200.0);
    assert_eq!(center.bounds.y, 0.0);
    assert_eq!(center.bounds.width, 550.0); // 1000 - 200 - 250
    assert_eq!(center.bounds.height, 800.0);
}

#[test]
fn test_dock_area_layout_hidden_panel() {
    let mut area = DockArea::new();
    area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
    area.add_panel({
        let mut panel =
            DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left).with_size(200.0);
        panel.visible = false;
        panel
    });
    area.add_panel(DockPanel::new(PanelId::SCENE_VIEW, "Scene", DockPosition::Center));
    area.layout();

    // Center should get full width since left panel is hidden
    let center = area.get_panel(PanelId::SCENE_VIEW).unwrap();
    assert_eq!(center.bounds.x, 0.0);
    assert_eq!(center.bounds.width, 1000.0);
}

#[test]
fn test_dock_area_layout_collapsed_left_is_slim_strip_and_center_reclaims() {
    let mut area = DockArea::new();
    area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
    area.add_panel(
        DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left).with_size(200.0),
    );
    area.add_panel(DockPanel::new(PanelId::SCENE_VIEW, "Scene", DockPosition::Center));
    area.set_panel_collapsed(PanelId::HIERARCHY, true);

    let strip = area.get_panel(PanelId::HIERARCHY).unwrap();
    assert_eq!(strip.bounds.width, HEADER_HEIGHT);
    let center = area.get_panel(PanelId::SCENE_VIEW).unwrap();
    assert_eq!(center.bounds.x, HEADER_HEIGHT);
    assert_eq!(center.bounds.width, 1000.0 - HEADER_HEIGHT);
}

#[test]
fn test_collapse_expand_round_trip_preserves_size() {
    let mut area = DockArea::new();
    area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
    area.add_panel(
        DockPanel::new(PanelId::INSPECTOR, "Inspector", DockPosition::Right).with_size(280.0),
    );
    area.layout();

    area.toggle_panel_collapsed(PanelId::INSPECTOR);
    assert!(area.get_panel(PanelId::INSPECTOR).unwrap().collapsed);
    assert_eq!(area.get_panel(PanelId::INSPECTOR).unwrap().bounds.width, HEADER_HEIGHT);

    area.toggle_panel_collapsed(PanelId::INSPECTOR);
    let panel = area.get_panel(PanelId::INSPECTOR).unwrap();
    assert!(!panel.collapsed);
    assert_eq!(panel.size, 280.0);
    assert_eq!(panel.bounds.width, 280.0);
}

#[test]
fn test_collapse_ignored_for_center_panel() {
    let mut area = DockArea::new();
    area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
    area.add_panel(DockPanel::new(PanelId::SCENE_VIEW, "Scene", DockPosition::Center));
    area.set_panel_collapsed(PanelId::SCENE_VIEW, true);
    assert!(!area.get_panel(PanelId::SCENE_VIEW).unwrap().collapsed);
}

#[test]
fn test_toggle_panel_visible_relayouts() {
    let mut area = DockArea::new();
    area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
    area.add_panel(
        DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left).with_size(200.0),
    );
    area.add_panel(DockPanel::new(PanelId::SCENE_VIEW, "Scene", DockPosition::Center));
    area.layout();
    assert_eq!(area.get_panel(PanelId::SCENE_VIEW).unwrap().bounds.x, 200.0);

    area.toggle_panel_visible(PanelId::HIERARCHY);
    assert!(!area.get_panel(PanelId::HIERARCHY).unwrap().visible);
    assert_eq!(area.get_panel(PanelId::SCENE_VIEW).unwrap().bounds.x, 0.0);

    area.toggle_panel_visible(PanelId::HIERARCHY);
    assert!(area.get_panel(PanelId::HIERARCHY).unwrap().visible);
    assert_eq!(area.get_panel(PanelId::SCENE_VIEW).unwrap().bounds.x, 200.0);
}

#[test]
fn test_resized_size_clamps_to_min_and_half_dock() {
    let dock = Rect::new(0.0, 0.0, 1000.0, 800.0);
    let panel_bounds = Rect::new(0.0, 0.0, 200.0, 800.0);

    // Dragging inward below min clamps to min
    let too_small = resized_size(
        DockPosition::Left,
        Vec2::new(10.0, 400.0),
        panel_bounds,
        100.0,
        dock,
    );
    assert_eq!(too_small, 100.0);

    // Dragging outward beyond half the dock clamps to half
    let too_big = resized_size(
        DockPosition::Left,
        Vec2::new(900.0, 400.0),
        panel_bounds,
        100.0,
        dock,
    );
    assert_eq!(too_big, 500.0);

    // In-range drag follows the mouse
    let ok = resized_size(
        DockPosition::Left,
        Vec2::new(300.0, 400.0),
        panel_bounds,
        100.0,
        dock,
    );
    assert_eq!(ok, 300.0);
}

#[test]
fn test_resized_size_right_and_bottom_measure_from_far_edge() {
    let dock = Rect::new(0.0, 0.0, 1000.0, 800.0);

    let right_bounds = Rect::new(750.0, 0.0, 250.0, 800.0);
    let right =
        resized_size(DockPosition::Right, Vec2::new(700.0, 400.0), right_bounds, 100.0, dock);
    assert_eq!(right, 300.0); // 1000 - 700

    let bottom_bounds = Rect::new(0.0, 620.0, 1000.0, 180.0);
    let bottom =
        resized_size(DockPosition::Bottom, Vec2::new(500.0, 600.0), bottom_bounds, 100.0, dock);
    assert_eq!(bottom, 200.0); // 800 - 600
}

#[test]
fn test_panel_id_for_menu_label_map() {
    assert_eq!(panel_id_for_menu_label("Inspector"), Some(PanelId::INSPECTOR));
    assert_eq!(panel_id_for_menu_label("Hierarchy"), Some(PanelId::HIERARCHY));
    assert_eq!(panel_id_for_menu_label("Asset Browser"), Some(PanelId::ASSET_BROWSER));
    // Scene view can't be hidden; Console has no panel yet.
    assert_eq!(panel_id_for_menu_label("Scene View"), None);
    assert_eq!(panel_id_for_menu_label("Console"), None);
    assert_eq!(panel_id_for_menu_label("Toggle Grid"), None);
}
