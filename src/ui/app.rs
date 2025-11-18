//! Application setup and initialization functionality.

use crate::ui::tabs;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    gio, Application, ApplicationWindow, Box as GtkBox, Builder, CssProvider, Paned, Stack,
};
use log::{info, warn};

/// Main application context with UI elements.
#[derive(Clone)]
pub struct AppContext {
    pub ui: UiComponents,
}

/// UI components grouped by functionality.
#[derive(Clone)]
pub struct UiComponents {
    pub stack: Stack,
    pub tabs_container: GtkBox,
    #[allow(dead_code)]
    pub main_paned: Paned,
}

/// Initialize and set up main application UI.
pub fn setup_application_ui(app: &Application) {
    info!("Initializing application components");

    setup_resources_and_theme();

    // Create single builder for all UI components
    let builder = Builder::from_resource("/xyz/xerolinux/xero-toolkit/ui/main.ui");
    let window = create_main_window(app, &builder);

    window.show();

    info!("Loading individual page UI components");
    load_page_contents(&builder);

    let ctx = setup_ui_components(&builder);

    // Setup UI components by category
    tabs::setup_tabs(&ctx.ui.tabs_container, &ctx.ui.stack);

    info!("Setting initial view to main page");
    ctx.ui.stack.set_visible_child_name("main_page");
    info!("Xero Toolkit application startup complete");
}

/// Set up resources and theme.
fn setup_resources_and_theme() {
    info!("Setting up resources and theme");

    // Must match the gresource name specified in build.rs
    gio::resources_register_include!("xyz.xerolinux.xero-toolkit.gresource")
        .expect("Failed to register gresources");

    if let Some(display) = gtk4::gdk::Display::default() {
        info!("Setting up UI theme and styling");

        // Add icons shipped in resources
        let theme = gtk4::IconTheme::for_display(&display);
        theme.add_resource_path("/xyz/xerolinux/xero-toolkit/icons");
        info!("Icon theme paths configured");

        // Load application CSS
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
fn setup_ui_components(builder: &Builder) -> AppContext {
    // Extract all widgets using helper
    let stack = extract_widget(builder, "stack");
    let tabs_container = extract_widget(builder, "tabs_container");
    let main_paned: Paned = extract_widget(builder, "main_paned");

    // Set up paned widget properties for better UX
    main_paned.set_wide_handle(true);

    // Set minimum and maximum position constraints
    const MIN_SIDEBAR_WIDTH: i32 = 200;
    const MAX_SIDEBAR_WIDTH: i32 = 400;

    // Add resize constraint handling with user feedback
    main_paned.connect_notify_local(Some("position"), move |paned, _| {
        let position = paned.position();

        // Enforce minimum width constraint
        if position < MIN_SIDEBAR_WIDTH {
            paned.set_position(MIN_SIDEBAR_WIDTH);
            log::debug!(
                "Sidebar resize limited to minimum width: {}",
                MIN_SIDEBAR_WIDTH
            );
            return;
        }

        // Enforce maximum width constraint
        if position > MAX_SIDEBAR_WIDTH {
            paned.set_position(MAX_SIDEBAR_WIDTH);
            log::debug!(
                "Sidebar resize limited to maximum width: {}",
                MAX_SIDEBAR_WIDTH
            );
            return;
        }

        log::debug!("Sidebar resized to width: {}", position);
    });

    // Set initial position within constraints
    if main_paned.position() < MIN_SIDEBAR_WIDTH {
        main_paned.set_position(MIN_SIDEBAR_WIDTH);
    } else if main_paned.position() > MAX_SIDEBAR_WIDTH {
        main_paned.set_position(MAX_SIDEBAR_WIDTH);
    }

    info!("All UI components successfully initialized from UI builder");

    // Assemble UI components
    let ui = UiComponents {
        stack,
        tabs_container,
        main_paned,
    };

    AppContext { ui }
}

/// Load page content from separate UI files into page containers
fn load_page_contents(main_builder: &Builder) {
    let pages = [
        (
            "main_page",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/main_page.ui",
            "page_main_page_container",
        ),
        (
            "customization",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/customization.ui",
            "page_customization_container",
        ),
        (
            "gaming_tools",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/gaming_tools.ui",
            "page_gaming_tools_container",
        ),
        (
            "containers_vms",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/containers_vms.ui",
            "page_containers_vms_container",
        ),
        (
            "multimedia_tools",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/multimedia_tools.ui",
            "page_multimedia_tools_container",
        ),
        (
            "kernel_manager_scx",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/kernel_manager_scx.ui",
            "page_kernel_manager_scx_container",
        ),
        (
            "servicing_system_tweaks",
            "/xyz/xerolinux/xero-toolkit/ui/tabs/servicing_system_tweaks.ui",
            "page_servicing_system_tweaks_container",
        ),
    ];

    for (page_name, resource_path, container_id) in pages {
        match load_page_from_resource(main_builder, page_name, resource_path, container_id) {
            Ok(_) => info!("Successfully loaded {} page", page_name),
            Err(e) => {
                warn!("Failed to load {} page: {}", page_name, e);
                // Create a fallback label for the page
                if let Some(container) = main_builder.object::<GtkBox>(container_id) {
                    let fallback_label = gtk4::Label::builder()
                        .label(format!("{} page content not available", page_name))
                        .build();
                    container.append(&fallback_label);
                }
            }
        }
    }
}

/// Load a single page from a UI resource file
fn load_page_from_resource(
    main_builder: &Builder,
    page_name: &str,
    resource_path: &str,
    container_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load the page UI from resource
    let page_builder = Builder::from_resource(resource_path);

    // Get the main page widget (first object in the UI file)
    let page_widget: gtk4::Widget = page_builder
        .object(format!("page_{}", page_name))
        .ok_or_else(|| {
            format!(
                "Could not find page_{} widget in {}",
                page_name, resource_path
            )
        })?;

    // Get the container from the main UI
    let container: GtkBox = main_builder
        .object(container_id)
        .ok_or_else(|| format!("Could not find container {} in main UI", container_id))?;

    // Add the page content to the container
    container.append(&page_widget);

    // Set up button connections for main page
    if page_name == "main_page" {
        setup_main_page_buttons(&page_builder, main_builder);
    }
    // Set up button connections for gaming tools page
    if page_name == "gaming_tools" {
        setup_gaming_tools_buttons(&page_builder);
    }
    // Set up button connections for containers/VMs page
    if page_name == "containers_vms" {
        setup_containers_vms_buttons(&page_builder);
    }

    Ok(())
}

/// Set up button functionality for the main page
fn setup_main_page_buttons(page_builder: &Builder, _main_builder: &Builder) {
    // Diamond buttons - placeholder functionality
    if let Some(btn_update_system) = page_builder.object::<gtk4::Button>("btn_update_system") {
        btn_update_system.connect_clicked(move |_| {
            info!("Main page: Update System button clicked - functionality not implemented yet");
        });
    }

    if let Some(btn_pkg_manager) = page_builder.object::<gtk4::Button>("btn_pkg_manager") {
        btn_pkg_manager.connect_clicked(move |_| {
            info!("Main page: PKG Manager GUI & others button clicked - functionality not implemented yet");
        });
    }

    if let Some(btn_parallel_downloads) =
        page_builder.object::<gtk4::Button>("btn_parallel_downloads")
    {
        btn_parallel_downloads.connect_clicked(move |_| {
            info!("Main page: Setup Parallel Downloads button clicked - functionality not implemented yet");
        });
    }

    if let Some(btn_drivers_codecs) = page_builder.object::<gtk4::Button>("btn_drivers_codecs") {
        btn_drivers_codecs.connect_clicked(move |_| {
            info!("Main page: Install Drivers/Codecs button clicked - functionality not implemented yet");
        });
    }

    // External link buttons
    if let Some(link_discord) = page_builder.object::<gtk4::Button>("link_discord") {
        link_discord.connect_clicked(move |_| {
            info!("Main page: Discord link clicked - opening https://discord.xerolinux.xyz/");
            let _ = std::process::Command::new("xdg-open")
                .arg("https://discord.xerolinux.xyz/")
                .spawn();
        });
    }

    if let Some(link_youtube) = page_builder.object::<gtk4::Button>("link_youtube") {
        link_youtube.connect_clicked(move |_| {
            info!("Main page: YouTube link clicked - opening https://www.youtube.com/@XeroLinux");
            let _ = std::process::Command::new("xdg-open")
                .arg("https://www.youtube.com/@XeroLinux")
                .spawn();
        });
    }

    if let Some(link_website) = page_builder.object::<gtk4::Button>("link_website") {
        link_website.connect_clicked(move |_| {
            info!("Main page: XeroLinux website link clicked - opening https://xerolinux.xyz/");
            let _ = std::process::Command::new("xdg-open")
                .arg("https://xerolinux.xyz/")
                .spawn();
        });
    }

    if let Some(link_donate) = page_builder.object::<gtk4::Button>("link_donate") {
        link_donate.connect_clicked(move |_| {
            info!("Main page: Donate link clicked - opening https://ko-fi.com/xerolinux");
            let _ = std::process::Command::new("xdg-open")
                .arg("https://ko-fi.com/xerolinux")
                .spawn();
        });
    }
}
/// Set up button functionality for the gaming tools page
fn setup_gaming_tools_buttons(page_builder: &Builder) {
    // Steam AiO
    if let Some(btn_steam_aio) = page_builder.object::<gtk4::Button>("btn_steam_aio") {
        btn_steam_aio.connect_clicked(move |_| {
            info!("Gaming tools: Steam AiO button clicked - functionality not implemented yet");
        });
    }

    // Controllers
    if let Some(btn_controllers) = page_builder.object::<gtk4::Button>("btn_controllers") {
        btn_controllers.connect_clicked(move |_| {
            info!("Gaming tools: Controllers button clicked - functionality not implemented yet");
        });
    }

    // Gamescope CFG
    if let Some(btn_gamescope_cfg) = page_builder.object::<gtk4::Button>("btn_gamescope_cfg") {
        btn_gamescope_cfg.connect_clicked(move |_| {
            info!("Gaming tools: Gamescope CFG button clicked - functionality not implemented yet");
        });
    }

    // LACT OC
    if let Some(btn_lact_oc) = page_builder.object::<gtk4::Button>("btn_lact_oc") {
        btn_lact_oc.connect_clicked(move |_| {
            info!("Gaming tools: LACT OC button clicked - functionality not implemented yet");
        });
    }

    // Lutris/Heroic & Bottles
    if let Some(btn_lutris_heroic_bottles) =
        page_builder.object::<gtk4::Button>("btn_lutris_heroic_bottles")
    {
        btn_lutris_heroic_bottles.connect_clicked(move |_| {
            info!("Gaming tools: Lutris/Heroic & Bottles button clicked - functionality not implemented yet");
        });
    }

    // Future reserved button removed from UI; handler no longer needed
}
/// Set up button functionality for the containers/VMs page
fn setup_containers_vms_buttons(page_builder: &Builder) {
    // Docker
    if let Some(btn_docker) = page_builder.object::<gtk4::Button>("btn_docker") {
        btn_docker.connect_clicked(move |_| {
            info!("Containers/VMs: Docker button clicked - functionality not implemented yet");
        });
    }

    // Podman
    if let Some(btn_podman) = page_builder.object::<gtk4::Button>("btn_podman") {
        btn_podman.connect_clicked(move |_| {
            info!("Containers/VMs: Podman button clicked - functionality not implemented yet");
        });
    }

    // vBox
    if let Some(btn_vbox) = page_builder.object::<gtk4::Button>("btn_vbox") {
        btn_vbox.connect_clicked(move |_| {
            info!("Containers/VMs: vBox button clicked - functionality not implemented yet");
        });
    }

    // DistroBox
    if let Some(btn_distrobox) = page_builder.object::<gtk4::Button>("btn_distrobox") {
        btn_distrobox.connect_clicked(move |_| {
            info!("Containers/VMs: DistroBox button clicked - functionality not implemented yet");
        });
    }

    // KVM
    if let Some(btn_kvm) = page_builder.object::<gtk4::Button>("btn_kvm") {
        btn_kvm.connect_clicked(move |_| {
            info!("Containers/VMs: KVM button clicked - functionality not implemented yet");
        });
    }
}
