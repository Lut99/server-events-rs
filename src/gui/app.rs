//  APP.rs
//    by Lut99
//
//  Created:
//    11 Apr 2024, 13:53:31
//  Last edited:
//    11 Apr 2024, 14:47:44
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
use std::sync::Arc;

use eframe::EventLoopBuilder;
use error_trace::trace;
use log::{debug, error, info, warn};
use parking_lot::{Mutex, MutexGuard};
use tray_icon::menu::MenuEvent;
use winit::event::{Event, StartCause};
use winit::event_loop::{EventLoop, EventLoopProxy, EventLoopWindowTarget};

use crate::gui::tray_icon::TrayIcon;
use crate::gui::window::Window;
use crate::state::{AppState, MuteState};


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
    state: AppState,

    /// The EventLoop that we use to handle events.
    eloop: EventLoop<MenuEvent>,

    /// The TrayIcon that forms the basis of interaction.
    ///
    /// Note that this may not live here, but instead in a separate thread where GTK does its thing.
    tray_icon: Arc<Mutex<Option<TrayIcon>>>,
    /// The Window that, when spawned, provides deeper interaction.
    window:    Option<Window>,
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
        Ok(Self { state, eloop, tray_icon: Arc::new(Mutex::new(None)), window: None })
    }

    /// Runs the app, receiving events and doing stuff based on that.
    ///
    /// # Returns
    /// This function only returns if the underlying EventLoop returns. This means that on some platforms (e.g., iOS / Android), it never returns.
    ///
    /// # Errors
    /// This function errors if _something_ went wrong while running the app.
    pub fn run(self) -> Result<(), Error> {
        // We start by running the event loop
        info!("Entering event loop...");
        let mut proxy: Option<EventLoopProxy<MenuEvent>> = Some(self.eloop.create_proxy());
        match self.eloop.run(|event, eloop| {
            // Received an event!
            match event {
                // System events (e.g., initialization)
                Event::NewEvents(event) => match event {
                    // Emitted once, at the start of the loop.
                    StartCause::Init => {
                        // Do one of two things, OS-dependent
                        #[cfg(not(target_os = "linux"))]
                        {
                            self.tray_icon = match TrayIcon::new(self.state.clone(), proxy.take().unwrap()) {
                                Ok(icon) => Arc::new(Mutex::new(Some(icon))),
                                Err(err) => {
                                    error!("{}", trace!(("Failed to create new TrayIcon"), err));
                                    eloop.exit();
                                    return;
                                },
                            };
                        }
                        #[cfg(target_os = "linux")]
                        {
                            // For Linux, this actually requires us to boot up that sweet GTK
                            let state: AppState = self.state.clone();
                            let tray_icon: Arc<Mutex<Option<TrayIcon>>> = self.tray_icon.clone();
                            let proxy: EventLoopProxy<MenuEvent> = proxy.take().unwrap();
                            std::thread::spawn(move || {
                                debug!("Initializing GTK...");
                                if let Err(err) = gtk::init() {
                                    error!("{}", trace!(("Failed to initialize GTK"), err));
                                    return;
                                }

                                // Build the icon
                                match TrayIcon::new(&state, proxy) {
                                    Ok(icon) => {
                                        let mut lock: MutexGuard<Option<TrayIcon>> = tray_icon.lock();
                                        *lock = Some(icon);
                                    },
                                    Err(err) => {
                                        error!("{}", trace!(("Failed to create new TrayIcon"), err));
                                        return;
                                    },
                                }

                                // Then delegate the rest to GTK's main
                                debug!("Running GTK main loop");
                                gtk::main();
                            });
                        }
                    },

                    // Other events are ignored
                    _ => return,
                },

                // Window events
                // TODO

                // Tray events
                Event::UserEvent(event) => Self::handle_tray_event(&self.state, eloop, event),

                // Other events are ignored
                _ => return,
            }
        }) {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::EventLoopRun { err }),
        }
    }

    /// Handles an emitted [`MenuEvent`] from a [`TrayIcon`].
    ///
    /// # Arguments
    /// - `state`: The [`AppState`] that contains any runtime state.
    /// - `eloop`: Some [`EventLoopWindowTarget`] that is used to influence the progression of the loop.
    /// - `tray_icon`: The [`TrayIcon`] itself that we want to see.
    /// - `event`: The [`MenuEvent`] to handle.
    pub fn handle_tray_event(state: &AppState, eloop: &EventLoopWindowTarget<MenuEvent>, event: MenuEvent) {
        match event.id.0.as_str() {
            "3" => debug!("Received 'Open' click in tray icon menu"),
            "4" => {
                debug!("Received 'Mute' click in tray icon menu");

                // Update the state with forever muted
                // ...or unmute it!
                if let Err(err) = state
                    .access_mut(|state| {
                        if state.muted.is_unmuted() {
                            state.muted = MuteState::Manual;
                        } else {
                            state.muted = MuteState::Unmuted;
                        }
                        Ok::<(), std::convert::Infallible>(())
                    })
                    .unwrap()
                {
                    warn!("{}", trace!(("Failed to sync app state back to disk"), err));
                }
            },
            "5" => {
                debug!("Received 'Mute until exit' click in tray icon menu");

                // Update the state accordingly
                // ...or unmute it!
                if let Err(err) = state
                    .access_mut(|state| {
                        state.muted = MuteState::NextBoot;
                        Ok::<(), std::convert::Infallible>(())
                    })
                    .unwrap()
                {
                    warn!("{}", trace!(("Failed to sync app state back to disk"), err));
                }
            },
            "6" => debug!("Received 'Mute for...' click in tray icon menu"),
            "7" => {
                debug!("Received 'Exit' click in tray icon menu");
                eloop.exit();
            },

            // Other events are ignored
            other => warn!("Received event for unknown tray icon menu ID '{other}'"),
        }
    }
}
