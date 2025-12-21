//! Application setup and initialization.

use crate::config;
use crate::core;
use crate::ui::context::AppContext;
use crate::ui::context::UiComponents;
use crate::ui::navigation;
use adw::prelude::*;
use adw::Application;
use gtk4::glib;
use gtk4::{gio, ApplicationWindow, Builder, CssProvider, Stack};
use log::{info, warn};

/// Initialize and set up main application UI.
pub fn setup_application_ui(app: &Application) {
    info!("Initializing application components");

    setup_resources_and_theme();

    let builder = Builder::from_resource("/xyz/xerolinux/xero-toolkit/ui/main.ui");
    let window = create_main_window(app, &builder);

    window.present();

    info!("Checking system dependencies");
    if !core::check_system_requirements(&window) {
        warn!("Dependency check failed - application will not continue");
        return;
    }

    // Initialize AUR helper after dependency checks pass
    if core::aur::init() {
        info!("AUR helper initialized successfully");
    }
    info!("Dependency check passed");

    // Extract tabs_container first for stack creation
    let tabs_container = extract_widget(&builder, "tabs_container");

    // Create dynamic stack with all pages and set up navigation tabs
    let stack = navigation::create_stack_and_tabs(&tabs_container, &builder);

    // Set up UI components with the dynamic stack
    let ctx = setup_ui_components(&builder, stack);

    info!("Setting initial view to first page");
    if let Some(first_page) = navigation::PAGES.first() {
        ctx.navigate_to_page(first_page.id);
    }
    info!("Xero Toolkit application startup complete");
}

/// Set up resources and theme.
fn setup_resources_and_theme() {
    info!("Setting up resources and theme");

    gio::resources_register_include!("xyz.xerolinux.xero-toolkit.gresource")
        .expect("Failed to register gresources");

    if let Some(display) = gtk4::gdk::Display::default() {
        info!("Setting up UI theme and styling");

        let theme = gtk4::IconTheme::for_display(&display);
        // Don't inherit system icon themes
        theme.set_search_path(&[]);
        theme.add_resource_path("/xyz/xerolinux/xero-toolkit/icons");
        info!("Icon theme paths configured");

        let css_provider = CssProvider::new();
        css_provider.load_from_resource("/xyz/xerolinux/xero-toolkit/css/style.css");
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

/// Create main application window.
fn create_main_window(app: &Application, builder: &Builder) -> ApplicationWindow {
    let window: ApplicationWindow = builder
        .object("app_window")
        .expect("Failed to get app_window");

    window.set_application(Some(app));
    info!("Setting window icon to xero-toolkit");
    window.set_icon_name(Some("xero-toolkit"));
    info!("Main application window created from UI resource");

    window
}

/// Helper to extract widgets from builder with consistent error handling.
pub fn extract_widget<T: IsA<glib::Object>>(builder: &Builder, name: &str) -> T {
    builder
        .object(name)
        .unwrap_or_else(|| panic!("Failed to get widget with id '{}'", name))
}

/// Set up UI components and return application context.
fn setup_ui_components(builder: &Builder, stack: Stack) -> AppContext {
    let tabs_container = extract_widget(builder, "tabs_container");
    let main_split_view = extract_widget(builder, "main_split_view");
    let sidebar_toggle = extract_widget(builder, "sidebar_toggle_button");

    // Set up autostart toggle in sidebar
    setup_autostart_toggle(builder);

    info!("All UI components successfully initialized from UI builder");

    let ui = UiComponents::new(stack, tabs_container, main_split_view, sidebar_toggle);

    // Configure sidebar with size constraints from config
    ui.configure_sidebar(config::sidebar::MIN_WIDTH, config::sidebar::MAX_WIDTH);

    AppContext::new(ui)
}

/// Set up the autostart toggle switch in the sidebar.
fn setup_autostart_toggle(builder: &Builder) {
    if let Some(switch) = builder.object::<gtk4::Switch>("switch_autostart") {
        // Set initial state based on whether autostart is enabled
        switch.set_active(core::autostart::is_enabled());

        switch.connect_state_set(move |_switch, state| {
            info!("Autostart toggle changed to: {}", state);

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
            }

            // Return Propagation::Proceed to allow the switch to update its state
            glib::Propagation::Proceed
        });
    }
}
