//  MOD.rs
//    by Lut99
//
//  Created:
//    02 Apr 2024, 15:13:02
//  Last edited:
//    01 May 2024, 19:51:49
//  Auto updated?
//    Yes
//
//  Description:
//!   Collects the part of the client concerned with rendering and
//!   updating the GUI, making callbacks to the more fundamental
//!   functions.
//

// Declare submodules
pub mod gui;
pub mod pipeline;
pub mod window;

// Imports
use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::PathBuf;

use egui_winit::winit;
use log::{debug, info};
use tray_icon::menu::MenuEvent;
use winit::event::Event;
use winit::event_loop::{EventLoop, EventLoopBuilder};

use crate::app::window::Window;
use crate::state::AppState;


/***** ERRORS *****/
/// Defines errors originating from running the [`App`].
#[derive(Debug)]
pub enum Error {
    /// Failed to create a new [`AppState`].
    AppStateCreate { err: crate::state::Error },
    /// Failed to create a new [`EventLoop`].
    EventLoopCreate { err: winit::error::EventLoopError },
    /// Failed to run the backend EventLoop.
    EventLoopRun { err: winit::error::EventLoopError },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            AppStateCreate { .. } => write!(f, "Failed to load app state"),
            EventLoopCreate { .. } => write!(f, "Failed to create main event loop"),
            EventLoopRun { .. } => write!(f, "Failed to run main event loop"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            AppStateCreate { err } => Some(err),
            EventLoopCreate { err } => Some(err),
            EventLoopRun { err } => Some(err),
        }
    }
}





/***** LIBRARY *****/
/// Wraps the other GUI elements and manages them.
pub struct App {
    /// The state shared between various components.
    state:  AppState,
    /// The EventLoop that we use to handle events.
    eloop:  EventLoop<MenuEvent>,
    /// The Window that, when spawned, provides deeper interaction.
    window: Option<Window>,
}
impl App {
    /// Constructor for the App that does what is necessary.
    ///
    /// # Arguments
    /// - `config_dir`: The directory to load persistent app state from.
    ///
    /// # Returns
    /// A new App, ready to rumble.
    ///
    /// # Errors
    /// This function errors if some part of the initialization failed.
    pub fn new(config_dir: PathBuf) -> Result<Self, Error> {
        info!("Initializing App...");

        // Build an app state
        let state: AppState = match AppState::new(config_dir) {
            Ok(state) => state,
            Err(err) => return Err(Error::AppStateCreate { err }),
        };

        // Build an EventLoop to match
        debug!("Initializing EventLoop...");
        let eloop: EventLoop<MenuEvent> = match EventLoopBuilder::with_user_event().build() {
            Ok(eloop) => eloop,
            Err(err) => return Err(Error::EventLoopCreate { err }),
        };

        // Done; build self
        info!("App initialization complete");
        Ok(Self { state, eloop, window: None })
    }

    /// Runs the app, receiving events and doing stuff based on that.
    ///
    /// # Returns
    /// This function only returns if the underlying EventLoop returns. This means that on some platforms (e.g., iOS / Android), it never returns.
    ///
    /// # Errors
    /// This function errors if _something_ went wrong while running the app.
    pub fn run(mut self) -> Result<(), Error> {
        // We start by running the event loop
        info!("Entering event loop...");
        match self.eloop.run(|event, eloop| {
            // Received an event!
            match event {
                // Init event
                // Event::NewEvents(StartCause::Init) => {},

                // Window events
                Event::WindowEvent { window_id, event } => {
                    let mut close: bool = false;
                    if let Some(window) = &mut self.window {
                        // See if it's about this window
                        if window_id == window.id() {
                            close = window.handle_event(event);
                        }
                    }
                    if close {
                        self.window = None;
                    }
                },

                // Other events are ignored
                _ => return,
            }
        }) {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::EventLoopRun { err }),
        }
    }
}
