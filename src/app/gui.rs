//  GUI.rs
//    by Lut99
//
//  Created:
//    11 Apr 2024, 17:56:50
//  Last edited:
//    01 May 2024, 19:44:01
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the actual user interface renderer.
//

use egui::Context;
use egui_winit::egui;


/***** LIBRARY *****/
pub struct Gui;
impl Gui {
    /// Builds the UI from the group up.
    ///
    /// # Arguments
    /// - `ctx`: The egui [`Context`] that we draw to.
    pub fn ui(&mut self, ctx: &Context) {
        // Just draw some wacky colour for now
        egui::CentralPanel::default().show(ctx, |ui| ui.add(egui::Label::new("Hello there!")));
    }
}
