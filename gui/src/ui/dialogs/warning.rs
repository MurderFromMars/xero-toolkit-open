//! Warning confirmation dialog for experimental features.

use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{Builder, Button, Label, Window};
use log::info;
use std::cell::RefCell;
use std::rc::Rc;

/// Show a warning confirmation dialog with cancel and continue buttons.
/// Calls on_confirm callback if user clicks continue.
pub fn show_warning_confirmation<F>(parent: &Window, heading: &str, on_confirm: F)
where
    F: FnOnce() + 'static,
{
    info!("Showing warning confirmation dialog: {}", heading);

    // Load the UI from resource
    let builder = Builder::from_resource(crate::config::resources::dialogs::WARNING);

    // Get the dialog window
    let dialog: Window = extract_widget(&builder, "warning_dialog");

    // Set transient parent
    dialog.set_transient_for(Some(parent));

    // Get UI elements
    let heading_label: Label = extract_widget(&builder, "dialog_heading");
    let warning_message: Label = extract_widget(&builder, "warning_message");
    let cancel_button: Button = extract_widget(&builder, "cancel_button");
    let continue_button: Button = extract_widget(&builder, "continue_button");

    // Set heading (remove emoji from heading since we have an icon now)
    heading_label.set_label(heading);

    // Set message with Pango markup for red text, centered
    warning_message.set_markup("Nix Package Manager is an <span foreground=\"red\" weight=\"bold\">EXPERIMENTAL</span> feature.\n\n\
        This is intended for <span foreground=\"red\" weight=\"bold\">EXPERIENCED USERS ONLY</span>.\n\
        <span foreground=\"red\" weight=\"bold\">Do NOT enable</span> unless you know what you are doing.\n\
        <span foreground=\"red\" weight=\"bold\">NO SUPPORT</span> will be provided for Nix-related issues.\n\n\
        Proceed at your own risk.");

    // Setup callbacks
    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        info!("Warning dialog cancelled");
        dialog_clone.close();
    });

    let dialog_clone = dialog.clone();
    let on_confirm_rc = Rc::new(RefCell::new(Some(on_confirm)));
    let on_confirm_weak = Rc::downgrade(&on_confirm_rc);
    
    continue_button.connect_clicked(move |_| {
        info!("Warning dialog confirmed");
        if let Some(on_confirm) = on_confirm_weak.upgrade().and_then(|rc| rc.borrow_mut().take()) {
            on_confirm();
        }
        dialog_clone.close();
    });

    // Show the dialog
    dialog.present();
}

