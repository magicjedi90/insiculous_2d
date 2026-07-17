//! GameRunner's post-update frame tail: everything the engine does after
//! the game's `update()` returns — particle stepping, line forwarding,
//! scene-defined UI elements, achievement toasts, and locale-font capture.
//!
//! Child module of `game` (like `render`) so it can reach the runner's
//! private fields without widening visibility.

use glam::Vec2;

use super::{Game, GameRunner};

impl<G: Game> GameRunner<G> {
    /// Engine-side work that runs right after `game.update()` each frame.
    pub(super) fn post_update(&mut self, delta_time: f32, window_size: Vec2, first_frame: bool) {
        // Step the particle system after the game's update — emitter
        // accumulators see the latest transforms, and pool stepping
        // happens once per frame. Scaled by time_scale so a paused game
        // (time_scale 0.0) freezes its particles with the rest of the world.
        crate::particles::ParticleSystem::update(
            &mut self.scene.world,
            &mut self.particles,
            delta_time * self.time_scale,
        );

        // Forward the line vertices the game pushed during update to the
        // renderer. Empty buffer == no lines drawn this frame.
        self.render_manager.set_lines(&self.lines);

        // Draw scene-defined UI elements (labels/panels/buttons) over the
        // game's own UI; presses buffer until the next frame's event flush.
        let ui_presses = crate::ui_element_system::draw_ui_elements(
            &self.scene.world,
            self.ui_manager.ui_context(),
            window_size,
            &self.strings,
        );
        self.pending_ui_events.extend(ui_presses);

        // Draw achievement toasts on top of whatever the game drew.
        self.achievements
            .draw_toasts(self.ui_manager.ui_context(), window_size);
        self.achievements.tick(delta_time);

        // The font the game set up in init() is the one locale switches
        // restore to — capture it once, before any locale font applies.
        if first_frame {
            self.base_font = self.ui_manager.ui_context().default_font();
        }
        // Apply a pending locale font change (from init/update set_locale).
        self.apply_locale_font();
    }
}
