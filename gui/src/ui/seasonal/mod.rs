//! Seasonal overlay effects for the application window.
//!
//! This module provides animated overlay effects that appear during specific
//! times of the year (e.g., snow for December, Halloween effects for October).
//!
//! Effects can be toggled on/off, and the animation timer is stopped when
//! effects are disabled to save CPU/memory.

mod common;
mod halloween;
mod snow;

use crate::ui::seasonal::common::MouseContext;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, DrawingArea};
use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

pub use halloween::HalloweenEffect;
pub use snow::SnowEffect;

/// Global state for whether seasonal effects are enabled.
static EFFECTS_ENABLED: AtomicBool = AtomicBool::new(true);

/// Entry for a registered effect with its drawing area and timer control.
struct EffectEntry {
    drawing_area: Rc<DrawingArea>,
    timer_source: Rc<RefCell<Option<glib::SourceId>>>,
}

/// Global registry of active effects.
struct EffectRegistry(RefCell<Vec<EffectEntry>>);

// SAFETY: Safe because GTK operations are single-threaded (main thread only).
unsafe impl Send for EffectRegistry {}
unsafe impl Sync for EffectRegistry {}

static EFFECT_REGISTRY: std::sync::OnceLock<EffectRegistry> = std::sync::OnceLock::new();

fn get_effect_registry() -> &'static RefCell<Vec<EffectEntry>> {
    &EFFECT_REGISTRY
        .get_or_init(|| EffectRegistry(RefCell::new(Vec::new())))
        .0
}

/// Check if seasonal effects are currently enabled.
pub fn are_effects_enabled() -> bool {
    EFFECTS_ENABLED.load(Ordering::Relaxed)
}

/// Set whether seasonal effects are enabled and update visibility/timers of drawing areas.
pub fn set_effects_enabled(enabled: bool) {
    EFFECTS_ENABLED.store(enabled, Ordering::Relaxed);

    let registry = get_effect_registry();
    for entry in registry.borrow().iter() {
        entry.drawing_area.set_visible(enabled);

        if enabled {
            // Restart timer if not already running
            let mut timer_ref = entry.timer_source.borrow_mut();
            if timer_ref.is_none() {
                let drawing_area_clone = entry.drawing_area.clone();
                let source_id =
                    glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                        drawing_area_clone.queue_draw();
                        glib::ControlFlow::Continue
                    });
                *timer_ref = Some(source_id);
                info!("Seasonal effect timer restarted");
            }
        } else {
            // Stop the timer to save resources
            let mut timer_ref = entry.timer_source.borrow_mut();
            if let Some(source_id) = timer_ref.take() {
                source_id.remove();
                info!("Seasonal effect timer stopped");
            }
        }
    }
}

/// Check if any seasonal effect is currently active.
pub fn has_active_effect() -> bool {
    let effects: Vec<Box<dyn SeasonalEffect>> =
        vec![Box::new(SnowEffect), Box::new(HalloweenEffect)];

    effects.iter().any(|e| e.is_active())
}

/// Register an effect with its drawing area and timer source for lifecycle management.
pub fn register_effect(
    drawing_area: Rc<DrawingArea>,
    timer_source: Rc<RefCell<Option<glib::SourceId>>>,
) {
    let registry = get_effect_registry();
    registry.borrow_mut().push(EffectEntry {
        drawing_area,
        timer_source,
    });
}

/// Trait for seasonal effects that can be applied to application windows.
pub trait SeasonalEffect {
    /// Check if this effect should be active at the current time.
    fn is_active(&self) -> bool;

    /// Get the name of this seasonal effect (for logging).
    fn name(&self) -> &'static str;

    /// Apply this effect to the given window.
    /// The mouse_context provides mouse position if the effect needs it.
    /// Returns the drawing area if the effect was successfully applied.
    fn apply(
        &self,
        window: &ApplicationWindow,
        mouse_context: Option<&MouseContext>,
    ) -> Option<Rc<DrawingArea>>;
}

/// Apply any active seasonal effects to the window.
pub fn apply_seasonal_effects(window: &ApplicationWindow) {
    if !are_effects_enabled() {
        info!("Seasonal effects are disabled");
        return;
    }

    info!("Checking for active seasonal effects...");

    let mouse_context = common::setup_mouse_tracking(window);

    let effects: Vec<Box<dyn SeasonalEffect>> =
        vec![Box::new(SnowEffect), Box::new(HalloweenEffect)];

    for effect in effects {
        if effect.is_active() {
            info!("Active seasonal effect detected: {}", effect.name());
            if let Some(_drawing_area) = effect.apply(window, Some(&mouse_context)) {
                // Effect registers itself via register_effect()
                info!("Successfully applied {} effect", effect.name());
            } else {
                info!("Failed to apply {} effect", effect.name());
            }
        }
    }
}
