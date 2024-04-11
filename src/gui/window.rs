//  WINDOW.rs
//    by Lut99
//
//  Created:
//    02 Apr 2024, 15:14:28
//  Last edited:
//    11 Apr 2024, 18:15:39
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements code for handling our [`Window`]-abstraction.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};

use egui::ViewportId;
use egui_winit::winit::event::WindowEvent;
use egui_winit::{egui, winit, EventResponse};
use log::{debug, info, trace};
use tray_icon::menu::MenuEvent;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{WindowBuilder, WindowId};

use crate::gui::gui::Gui;


/***** ERRORS *****/
/// Defines errors originating from a [`Window`].
#[derive(Debug)]
pub enum Error {
    /// Failed to create a new [`winit::Window`](winit::window::Window).
    WindowCreate { title: String, err: winit::error::OsError },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            WindowCreate { title, .. } => write!(f, "Failed to create new Window with title '{title}'"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            WindowCreate { err, .. } => Some(err),
        }
    }
}





/***** LIBRARY *****/
/// Implements an abstraction of a Window over some backend.
///
/// Currently, only [`winit`] (through [`egui_winit`]) is supported.
pub struct Window {
    /// The wrapper [`eframe::Window`] we actually render on.
    window: winit::window::Window,
    /// Some [`egui_winit::State`] that we can use to wrap the window.
    egui_state: egui_winit::State,
    /// The [`Gui`] that we will draw in this window.
    gui: Gui,
}
impl Window {
    /// Constructor for the Window.
    ///
    /// # Arguments
    /// - `event_loop`: Some [`EventLoop`] that we use to listen to this window's events.
    /// - `title`: The (initial) title of the window.
    ///
    /// # Returns
    /// A new Window.
    ///
    /// # Errors
    /// This function errors if it fails to build a new [`winit::Window`](winit::window::Window).
    pub fn new(event_loop: &EventLoopWindowTarget<MenuEvent>, title: impl AsRef<str>) -> Result<Self, Error> {
        let title: &str = title.as_ref();
        info!("Initializing Window '{title}'...");

        // Build the eframe window
        debug!("Building backend window...");
        let window: winit::window::Window = match WindowBuilder::new().with_title(title).build(event_loop) {
            Ok(win) => win,
            Err(err) => return Err(Error::WindowCreate { title: title.into(), err }),
        };

        // Create an egui context and then an egui-winit state
        debug!("Building egui state...");
        let egui_ctx: egui::Context = egui::Context::default();
        let egui_state: egui_winit::State = egui_winit::State::new(egui_ctx, ViewportId::ROOT, &window, None, None);

        // Build the context
        debug!("Building GUI...");
        let gui: Gui = Gui {};

        // Done, build self
        Ok(Self { window, egui_state, gui })
    }

    /// Handles [`WindowEvent`]s with the builtin egui state.
    ///
    /// # Arguments
    /// - `event`: The [`WindowEvent`] to handle.
    ///
    /// # Returns
    /// The same WindowEvent if egui is OK with it. Else, egui consumed it.
    #[inline]
    pub fn handle_event(&mut self, event: WindowEvent) -> Option<WindowEvent> {
        let response: EventResponse = self.egui_state.on_window_event(&self.window, &event);
        if response.repaint {
            self.paint();
        }
        if !response.consumed { Some(event) } else { None }
    }

    /// Renders the given egui application to the screen.
    #[inline]
    pub fn paint(&mut self) {
        trace!("Painting Window");
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let output = self.egui_state.egui_ctx().run(raw_input, |egui_ctx| self.gui.ui(egui_ctx));
        self.egui_state.handle_platform_output(&self.window, output.platform_output);
    }

    /// Makes this already existing window active.
    #[inline]
    pub fn focus(&self) { self.window.focus_window(); }

    /// Returns the ID of this Window.
    ///
    /// # Returns
    /// A [`WindowId`] allowing us to recognize the window.
    #[inline]
    pub fn id(&self) -> WindowId { self.window.id() }

    /// Returns the whole inner [`winit::Window`](winit::window::Window).
    #[inline]
    pub fn inner(&self) -> &winit::window::Window { &self.window }
}
