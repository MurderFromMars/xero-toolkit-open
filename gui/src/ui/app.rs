//! Application setup and initialization.

use crate::config;
use crate::config::user::Config;
use crate::core;
use crate::ui::context::AppContext;
use crate::ui::context::UiComponents;
use crate::ui::navigation;
use crate::ui::utils::extract_widget;
use adw::prelude::*;
use adw::Application;
use gtk4::glib;
use gtk4::{gio, ApplicationWindow, Builder, CssProvider, Stack};
use log::{error, info, warn};
use std::cell::RefCell;
use std::rc::Rc;

/// Initialize and set up main application UI.
pub fn setup_application_ui(app: &Application) {
    info!("Initializing application components");

    setup_resources_and_theme();

    let config = Rc::new(RefCell::new(Config::load()));
    info!("User configuration loaded");

    // Persist configuration once on application shutdown to avoid IO during interaction.
    {
        let config_for_shutdown = Rc::clone(&config);
        app.connect_shutdown(move |_| {
            if let Err(e) = config_for_shutdown.borrow().save() {
                eprintln!("Failed to save config on shutdown: {e}");
            } else {
                info!("Configuration saved on shutdown");
            }
        });
    }

    let builder = Builder::from_resource(config::resources::MAIN_UI);
    let window = create_main_window(app, &builder);

    window.present();

    info!("Initializing environment variables");
    if let Err(e) = config::env::init() {
        error!("Failed to initialize environment variables: {}", e);
        crate::ui::dialogs::error::show_error(
            &window,
            &format!(
                "Failed to initialize environment variables: {}\n\nRequired environment variables (USER, HOME) are not set.",
                e
            ),
        );
        return;
    }

    let distribution_name = core::get_distribution_name()
        .unwrap_or_else(|| "Unknown".to_string())
        .to_lowercase();
    match distribution_name.as_str() {
        "xerolinux" => info!("Running on XeroLinux"),
        _ => {
            warn!(
                "Not running on XeroLinux - current distribution: {}",
                distribution_name
            );
            warn!("Some features may not work correctly on non-XeroLinux systems");

            if !config.borrow().warnings.dismissed_generic_distro_notice {
                core::system_check::show_generic_distro_notice(
                    &window,
                    config.clone(),
                    distribution_name.clone(),
                );
            }
        }
    }

    let tabs_container = extract_widget(&builder, "tabs_container");

    let stack = navigation::create_stack_and_tabs(&tabs_container, &builder);

    let ctx = setup_ui_components(&builder, stack, &window, config.clone());

    info!("Setting initial view to first page");
    if let Some(first_page) = navigation::PAGES.first() {
        ctx.navigate_to_page(first_page.id);
    }

    crate::ui::seasonal::apply_seasonal_effects(&window);

    info!("Running dependency checks");
    let dependency_result = core::check_dependencies();
    if dependency_result.has_missing_dependencies() {
        core::show_dependency_error_dialog(&window, &dependency_result);
        return;
    }

    if core::aur::init() {
        info!("AUR helper initialized successfully");
    } else {
        warn!("No AUR helper detected");
    }

    info!("Xero Toolkit application startup complete");
}

fn setup_resources_and_theme() {
    info!("Setting up resources and theme");

    gio::resources_register_include!("xyz.xerolinux.xero-toolkit.gresource")
        .expect("Failed to register gresources");

    if let Some(display) = gtk4::gdk::Display::default() {
        info!("Setting up UI theme and styling");

        let theme = gtk4::IconTheme::for_display(&display);
        theme.set_search_path(&[]);
        theme.add_resource_path(config::resources::ICONS);
        info!("Icon theme paths configured");

        let css_provider = CssProvider::new();
        css_provider.load_from_resource(config::resources::CSS);
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        info!("UI theme and styling loaded successfully");
    } else {
        warn!("No default display found - UI theming may not work properly");
    }
}

fn create_main_window(app: &Application, builder: &Builder) -> ApplicationWindow {
    let window: ApplicationWindow = extract_widget(builder, "app_window");

    window.set_application(Some(app));
    info!("Setting window icon to xero-toolkit");
    window.set_icon_name(Some("xero-toolkit"));
    info!("Main application window created from UI resource");

    window
}

fn setup_ui_components(
    builder: &Builder,
    stack: Stack,
    window: &ApplicationWindow,
    config: Rc<RefCell<Config>>,
) -> AppContext {
    let tabs_container = extract_widget(builder, "tabs_container");
    let main_split_view = extract_widget(builder, "main_split_view");
    let sidebar_toggle = extract_widget(builder, "sidebar_toggle_button");

    setup_autostart_toggle(builder, config.clone());
    setup_about_button(builder, window);
    setup_seasonal_effects_toggle(builder, window);

    info!("All UI components successfully initialized from UI builder");

    let ui = UiComponents::new(stack, tabs_container, main_split_view, sidebar_toggle);

    ui.configure_sidebar(config::sidebar::MIN_WIDTH, config::sidebar::MAX_WIDTH);

    AppContext::new(ui, config)
}

fn setup_autostart_toggle(builder: &Builder, config: Rc<RefCell<Config>>) {
    let switch = extract_widget::<gtk4::Switch>(builder, "switch_autostart");
    switch.set_active(config.borrow().general.autostart);

    let config_clone = Rc::clone(&config);
    switch.connect_state_set(move |_switch, state| {
        info!("Autostart toggle changed to: {}", state);

        // Update in-memory config; actual persistence happens on app shutdown.
        config_clone.borrow_mut().general.autostart = state;

        let result = if state {
            core::autostart::enable()
        } else {
            core::autostart::disable()
        };

        if let Err(e) = result {
            warn!(
                "Failed to {} autostart: {}",
                if state { "enable" } else { "disable" },
                e
            );
            // Prevent the switch from updating its state on failure
            return glib::Propagation::Stop;
        }

        glib::Propagation::Proceed
    });
}

fn setup_about_button(builder: &Builder, window: &ApplicationWindow) {
    use crate::ui::dialogs::about;

    let button = extract_widget::<gtk4::Button>(builder, "about_button");
    let window_clone = window.clone();
    button.connect_clicked(move |_| {
        info!("About button clicked");
        about::show_about_dialog(window_clone.upcast_ref());
    });
}

fn setup_seasonal_effects_toggle(builder: &Builder, _window: &ApplicationWindow) {
    use crate::ui::seasonal;

    let toggle = extract_widget::<gtk4::ToggleButton>(builder, "seasonal_effects_toggle");

    let has_active = seasonal::has_active_effect();
    toggle.set_visible(has_active);
    toggle.set_active(seasonal::are_effects_enabled());

    toggle.connect_toggled(move |btn| {
        let enabled = btn.is_active();
        seasonal::set_effects_enabled(enabled);
        info!(
            "Seasonal effects {}",
            if enabled { "enabled" } else { "disabled" }
        );
    });
}
