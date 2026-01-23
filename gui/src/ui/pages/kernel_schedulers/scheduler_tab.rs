//! SCX Scheduler page handlers.
//!
//! Manages sched-ext BPF CPU schedulers via scxctl.

use crate::ui::dialogs::warning::show_warning_confirmation;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::{
    extract_widget, get_combo_row_value, is_service_enabled, path_exists, run_command,
};
use adw::prelude::*;
use gtk4::glib;
use gtk4::{ApplicationWindow, Builder, Button, Image, Label, StringList};
use log::{info, warn};
use std::cell::RefCell;
use std::rc::Rc;

const SCHED_EXT_PATH: &str = "/sys/kernel/sched_ext";

/// Shared state for the scheduler page
#[derive(Default)]
struct State {
    schedulers: Vec<String>,
    kernel_supported: bool,
    is_active: bool,
}

pub fn setup_handlers(builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    let state = Rc::new(RefCell::new(State::default()));

    init_kernel_support(builder, &state);
    setup_buttons(builder, window, &state);
    setup_persistence(builder, window);

    // Initial scan
    let b = builder.clone();
    let s = Rc::clone(&state);
    glib::idle_add_local_once(move || refresh_state(&b, &s));

    // Status monitor
    let b = builder.clone();
    let s = Rc::clone(&state);
    glib::timeout_add_seconds_local(3, move || {
        update_status(&b, &s);
        glib::ControlFlow::Continue
    });
}

fn init_kernel_support(builder: &Builder, state: &Rc<RefCell<State>>) {
    let version = run_command("uname", &["-r"]).unwrap_or_else(|| "Unknown".to_string());
    let supported = path_exists(SCHED_EXT_PATH);

    state.borrow_mut().kernel_supported = supported;

    let icon = extract_widget::<Image>(builder, "kernel_status_icon");
    let label = extract_widget::<Label>(builder, "kernel_version_label");

    if supported {
        icon.set_icon_name(Some("circle-check"));
        icon.add_css_class("success");
        label.set_text(&version);
        label.remove_css_class("warning");
    } else {
        icon.set_icon_name(Some("circle-xmark"));
        icon.add_css_class("error");
        label.set_text(&format!("{} (no sched-ext)", version));
        label.add_css_class("warning");
    }

    // Hidden label for compatibility
    extract_widget::<Label>(builder, "kernel_support_label").set_text(if supported {
        "Supported"
    } else {
        "Not supported"
    });
}

fn setup_buttons(builder: &Builder, window: &ApplicationWindow, state: &Rc<RefCell<State>>) {
    // Refresh button
    let b = builder.clone();
    let s = Rc::clone(state);
    extract_widget::<Button>(builder, "btn_refresh_schedulers").connect_clicked(move |btn| {
        btn.set_sensitive(false);
        refresh_state(&b, &s);
        btn.set_sensitive(true);
    });

    // Switch button
    let b = builder.clone();
    let w = window.clone();
    let s = Rc::clone(state);
    extract_widget::<Button>(builder, "btn_switch_scheduler").connect_clicked(move |_| {
        let scheduler =
            get_combo_row_value(&extract_widget::<adw::ComboRow>(&b, "scheduler_combo"));
        let mode = get_combo_row_value(&extract_widget::<adw::ComboRow>(&b, "mode_combo"))
            .unwrap_or_else(|| "auto".to_string());

        let Some(sched_display) = scheduler else {
            warn!("No valid scheduler selected");
            return;
        };

        let sched_name = sched_display.to_lowercase();
        let sched = format!("scx_{}", sched_name);
        let cmd = if s.borrow().is_active {
            "switch"
        } else {
            "start"
        };

        info!("{cmd}ing scheduler {sched_name} with mode {mode}");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("scxctl")
                    .args(&[cmd, "--sched", &sched_name, "--mode", &mode])
                    .description(&format!(
                        "{}ing {} ({} mode)...",
                        if cmd == "switch" { "Switch" } else { "Start" },
                        sched,
                        mode
                    ))
                    .build(),
            )
            .build();

        task_runner::run(
            w.upcast_ref(),
            commands,
            if cmd == "switch" {
                "Switch Scheduler"
            } else {
                "Start Scheduler"
            },
        );
    });

    // Stop button
    let w = window.clone();
    extract_widget::<Button>(builder, "btn_stop_scheduler").connect_clicked(move |_| {
        let wc = w.clone();
        show_warning_confirmation(
            w.upcast_ref(),
            "Stop Scheduler",
            "Stop the current scheduler and fall back to EEVDF?",
            move || {
                task_runner::run(
                    wc.upcast_ref(),
                    CommandSequence::new()
                        .then(
                            Command::builder()
                                .normal()
                                .program("scxctl")
                                .args(&["stop"])
                                .description("Stopping scheduler...")
                                .build(),
                        )
                        .build(),
                    "Stop Scheduler",
                );
            },
        );
    });
}

fn setup_persistence(builder: &Builder, window: &ApplicationWindow) {
    let switch = extract_widget::<adw::SwitchRow>(builder, "persist_switch");
    switch.set_active(is_service_enabled("scx.service"));

    let b = builder.clone();
    let w = window.clone();
    switch.connect_active_notify(move |sw| {
        if sw.is_active() {
            let scheduler =
                get_combo_row_value(&extract_widget::<adw::ComboRow>(&b, "scheduler_combo"));
            let mode = get_combo_row_value(&extract_widget::<adw::ComboRow>(&b, "mode_combo"))
                .unwrap_or_else(|| "auto".to_string());

            let Some(sched_display) = scheduler else {
                warn!("No valid scheduler selected for persistence");
                sw.set_active(false);
                return;
            };

            let sched_name = sched_display.to_lowercase();
            let sched = format!("scx_{}", sched_name);
            let template_path = crate::config::paths::systemd().join("scx.service.in");

            let Ok(content) = std::fs::read_to_string(&template_path) else {
                warn!("Failed to read service template");
                sw.set_active(false);
                return;
            };

            let service = content
                .replace("@SCHEDULER@", &sched)
                .replace("@SCHEDULER_NAME@", &sched_name)
                .replace("@MODE@", &mode);

            if std::fs::write("/tmp/scx.service", &service).is_err() {
                sw.set_active(false);
                return;
            }

            task_runner::run(
                w.upcast_ref(),
                CommandSequence::new()
                    .then(
                        Command::builder()
                            .privileged()
                            .program("cp")
                            .args(&["/tmp/scx.service", "/etc/systemd/system/scx.service"])
                            .description("Installing service...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .privileged()
                            .program("systemctl")
                            .args(&["daemon-reload"])
                            .description("Reloading systemd...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .privileged()
                            .program("systemctl")
                            .args(&["enable", "scx.service"])
                            .description("Enabling service...")
                            .build(),
                    )
                    .build(),
                "Enable Persistence",
            );
        } else {
            task_runner::run(
                w.upcast_ref(),
                CommandSequence::new()
                    .then(
                        Command::builder()
                            .privileged()
                            .program("systemctl")
                            .args(&["disable", "scx.service"])
                            .description("Disabling service...")
                            .build(),
                    )
                    .build(),
                "Disable Persistence",
            );
        }
    });
}

fn refresh_state(builder: &Builder, state: &Rc<RefCell<State>>) {
    let schedulers = get_schedulers();
    let (is_active, name, mode) = get_status();
    let kernel_supported = path_exists(SCHED_EXT_PATH);

    {
        let mut s = state.borrow_mut();
        s.schedulers = schedulers.clone();
        s.kernel_supported = kernel_supported;
        s.is_active = is_active;
    }

    // Populate dropdown
    let combo = extract_widget::<adw::ComboRow>(builder, "scheduler_combo");
    let display_names: Vec<String> = schedulers.iter().map(|s| humanize_name(s)).collect();
    let list = StringList::new(&display_names.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    combo.set_model(Some(&list));
    if !schedulers.is_empty() {
        combo.set_selected(0);
    }

    // Update status display
    update_status_labels(builder, is_active, &name, &mode);

    // Update buttons
    let can_switch = kernel_supported && !schedulers.is_empty();
    extract_widget::<Button>(builder, "btn_switch_scheduler").set_sensitive(can_switch);
    extract_widget::<Button>(builder, "btn_stop_scheduler").set_sensitive(is_active);

    // Update persistence
    extract_widget::<adw::SwitchRow>(builder, "persist_switch")
        .set_active(is_service_enabled("scx.service"));

    info!(
        "Found {} schedulers, active={}",
        schedulers.len(),
        is_active
    );
}

fn update_status(builder: &Builder, state: &Rc<RefCell<State>>) {
    let (is_active, name, mode) = get_status();
    state.borrow_mut().is_active = is_active;

    update_status_labels(builder, is_active, &name, &mode);
    extract_widget::<Button>(builder, "btn_stop_scheduler").set_sensitive(is_active);
}

fn update_status_labels(builder: &Builder, is_active: bool, name: &str, mode: &str) {
    let active_label = extract_widget::<Label>(builder, "active_scheduler_label");
    let mode_label = extract_widget::<Label>(builder, "mode_label");

    if is_active {
        active_label.set_text(&humanize_name(name));
        active_label.remove_css_class("dim-label");
        active_label.add_css_class("accent");
        mode_label.set_text(mode);
        mode_label.remove_css_class("dim-label");
    } else {
        active_label.set_text("EEVDF (Default)");
        active_label.remove_css_class("accent");
        active_label.add_css_class("dim-label");
        mode_label.set_text("N/A");
        mode_label.add_css_class("dim-label");
    }
}

fn get_schedulers() -> Vec<String> {
    run_command("scxctl", &["list"])
        .and_then(|out| {
            out.find("supported schedulers:")
                .and_then(|i| out[i + 21..].find('[').map(|j| i + 21 + j))
                .and_then(|start| {
                    out[start..]
                        .find(']')
                        .map(|end| &out[start + 1..start + end])
                })
                .map(|list| {
                    list.split(',')
                        .map(|s| format!("scx_{}", s.trim().trim_matches('"')))
                        .filter(|s| s.len() > 4)
                        .collect()
                })
        })
        .unwrap_or_default()
}

fn get_status() -> (bool, String, String) {
    run_command("scxctl", &["get"])
        .map(|out| {
            let lower = out.to_lowercase();
            if lower.contains("not running") || out.is_empty() {
                return (false, String::new(), String::new());
            }
            if lower.starts_with("running") {
                let parts: Vec<&str> = out.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = format!("scx_{}", parts[1].to_lowercase());
                    let mode = out
                        .split(" in ")
                        .nth(1)
                        .and_then(|s| s.split(" mode").next())
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|| "N/A".to_string());
                    return (true, name, mode);
                }
            }
            (false, String::new(), String::new())
        })
        .unwrap_or((false, String::new(), String::new()))
}

fn humanize_name(name: &str) -> String {
    let name = name.strip_prefix("scx_").unwrap_or(name);
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
