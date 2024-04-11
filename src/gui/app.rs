//  APP.rs
//    by Lut99
//
//  Created:
//    11 Apr 2024, 13:53:31
//  Last edited:
//    11 Apr 2024, 18:09:28
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the global [`App`] which wraps the GUI elements and manages
//!   them.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::PathBuf;

use egui_winit::{egui, winit};
use error_trace::trace;
use log::{debug, error, info, warn};
use tray_icon::menu::MenuEvent;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget};
use winit::window::WindowId;

use super::tray_icon::{TrayIconHandle, TrayIconMenuItem};
use crate::gui::tray_icon::TrayIcon;
use crate::gui::window::Window;
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
    /// Failed to create a [`TrayIcon`].
    TrayIconCreate { err: crate::gui::tray_icon::Error },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            AppStateCreate { .. } => write!(f, "Failed to load app state"),
            EventLoopCreate { .. } => write!(f, "Failed to create main event loop"),
            EventLoopRun { .. } => write!(f, "Failed to run main event loop"),
            TrayIconCreate { .. } => write!(f, "Failed to create tray icon"),
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
            TrayIconCreate { err } => Some(err),
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
        let mut proxy: Option<EventLoopProxy<MenuEvent>> = Some(self.eloop.create_proxy());
        let mut tray_icon: Option<TrayIconHandle> = None;
        match self.eloop.run(|event, eloop| {
            // Received an event!
            match event {
                // Init event
                Event::NewEvents(StartCause::Init) => {
                    // Other OS' implementation
                    #[cfg(not(target_os = "linux"))]
                    {
                        // Build the tray icon
                        tray_icon = match TrayIcon::new(self.state.clone(), proxy.take().unwrap()) {
                            Ok(icon) => Some(TrayIconHandle::new(icon)),
                            Err(err) => {
                                error!("{}", trace!(("Failed to create TrayIcon"), err));
                                eloop.exit();
                                return;
                            },
                        };
                    }
                },

                // Window events
                Event::WindowEvent { window_id, event } => Self::handle_window_event(&mut self.window, window_id, event),

                // Tray events
                Event::UserEvent(event) => Self::handle_tray_event(eloop, tray_icon.as_ref(), &mut self.window, event),

                // Other events are ignored
                _ => return,
            }
        }) {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::EventLoopRun { err }),
        }
    }

    /// Handles an emitted [`WindowEvent`] from a [`Window`].
    ///
    /// # Arguments
    /// - `window`: A mutable handle to the [`Window`] object that we may potentially alter or remove.
    /// - `window_id`: The [`WindowId`] of the window that triggered the `event`.
    /// - `event`: The [`WindowEvent`] to handle.
    pub fn handle_window_event(widnow_h: &mut Option<Window>, window_id: WindowId, event: WindowEvent) {
        // Only do if our window exists
        let window: &mut Window = match widnow_h {
            Some(window) => window,
            None => {
                warn!("Received WindowEvent without owning a window");
                return;
            },
        };
        if window.id() != window_id {
            warn!("Received WindowEvent for another window");
            return;
        }

        // Let the window match events first, then its our turn
        match window.handle_event(event) {
            // Quitting
            Some(WindowEvent::CloseRequested) => {
                debug!("Received window close click");
                *widnow_h = None;
                return;
            },

            // The rest we ignore
            _ => return,
        }
    }

    /// Handles an emitted [`MenuEvent`] from a [`TrayIcon`].
    ///
    /// # Arguments
    /// - `eloop`: Some [`EventLoopWindowTarget`] that is used to influence the progression of the loop.
    /// - `tray_icon_check_sender`: Some [`Sender`] that can update the checkbox of the tray icone.
    /// - `window`: A mutable handle to the [`Window`] object that we may potentially create.
    /// - `event`: The [`MenuEvent`] to handle.
    pub fn handle_tray_event(
        eloop: &EventLoopWindowTarget<MenuEvent>,
        tray_icon_h: Option<&TrayIconHandle>,
        window: &mut Option<Window>,
        event: MenuEvent,
    ) {
        let tray_icon_h: &TrayIconHandle = match tray_icon_h {
            Some(handle) => handle,
            None => panic!("Processing tray event before tray is created; this should never happen!"),
        };

        // Check which item it is
        if &event.id == tray_icon_h.get_id_of_item(TrayIconMenuItem::Open) {
            debug!("Received 'Open' click in tray icon menu");

            // Either spawn the window or make it active
            if let Some(window) = window {
                window.focus();
            } else {
                // Build the window
                match Window::new(eloop, format!("Server Events v{}", env!("CARGO_PKG_VERSION"))) {
                    Ok(win) => *window = Some(win),
                    Err(err) => {
                        error!("{}", trace!(("Failed to build new Server Events window"), err));
                    },
                }
            }
        } else if &event.id == tray_icon_h.get_id_of_item(TrayIconMenuItem::Exit) {
            debug!("Received 'Exit' click in tray icon menu");
            eloop.exit();
        } else {
            warn!("Received event for unknown tray icon menu ID '{}'", event.id.0);
        }
    }
}
