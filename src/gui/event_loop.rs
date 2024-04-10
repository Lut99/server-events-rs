//  EVENT LOOP.rs
//    by Lut99
//
//  Created:
//    10 Apr 2024, 11:03:11
//  Last edited:
//    10 Apr 2024, 13:42:42
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines a wrapper around a [`winit`] event loop to suit our own
//!   needs.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};

use log::info;
use tray_icon::menu::{MenuEvent, MenuId};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoopBuilder, EventLoopProxy};


/***** ERRORS *****/
/// The errors that originate from [`EventLoop`]s.
#[derive(Debug)]
pub enum Error {
    /// Failed to create a new [`winit::EventLoop`](winit::event_loop::EventLoop).
    EventLoopCreate { err: winit::error::EventLoopError },
    /// Failed to run the [`winit::EventLoop`](winit::event_loop::EventLoop).
    Run { err: winit::error::EventLoopError },
    /// Failed to send an event across space and time.
    SendEvent { id: MenuId, err: winit::event_loop::EventLoopClosed<MenuEvent> },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            EventLoopCreate { .. } => write!(f, "Failed to create new winit EventLoop"),
            Run { .. } => write!(f, "Failed to run the EventLoop"),
            SendEvent { id, .. } => write!(f, "Failed to send MenuEvent with ID '{id:?}' to main event loop"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            EventLoopCreate { err } => Some(err),
            Run { err } => Some(err),
            SendEvent { err, .. } => Some(err),
        }
    }
}





/***** LIBRARY *****/
/// A wrapper around an [`EventLoopProxy`] to send commands to an [`EventLoop`] from another thread.
#[derive(Debug)]
pub struct EventLoopHandle {
    /// The proxy we wrap.
    proxy: EventLoopProxy<MenuEvent>,
}
impl EventLoopHandle {
    /// Raises a user event on the main loop.
    ///
    /// # Arguments
    /// - `event`: The [`MenuEvent`] to raise.
    ///
    /// # Errors
    /// This function may error if we failed to send the event, e.g., the other event loop no longer exists.
    pub fn raise(&self, event: MenuEvent) -> Result<(), Error> {
        let id: MenuId = event.id.clone();
        self.proxy.send_event(event).map_err(move |err| Error::SendEvent { id, err })
    }
}



/// Our own [`winit::event_loop::EventLoop`] wrapper.
#[derive(Debug)]
pub struct EventLoop {
    /// The loop we wrap.
    eloop: EventLoop<MenuEvent>,
}
impl EventLoop {
    /// Constructor for the EventLoop.
    ///
    /// # Returns
    /// A new EventLoop that can be used to build windows and tray icons.
    ///
    /// # Errors
    /// This function errors if we failed to build the winit's [`winit::event_loop::EventLoop`].
    pub fn new() -> Result<Self, Error> {
        info!("Initializing EventLoop...");

        // Build the winit event loop
        let eloop: winit::event_loop::EventLoop<MenuEvent> = match EventLoopBuilder::with_user_event().build() {
            Ok(eloop) => eloop,
            Err(err) => return Err(Error::EventLoopCreate { err }),
        };

        // Done
        Ok(Self { eloop })
    }

    /// Creates a handle that can be used on other threads to send commands to this EventLoop.
    ///
    /// # Returns
    /// An [`EventLoopHandle`] that is thread-safe.
    pub fn create_handle(&self) -> EventLoopHandle { EventLoopHandle { proxy: self.eloop.create_proxy() } }

    /// Runs this event loop.
    ///
    /// This will hijack the current thread to run the event loop. On some platforms (iOS and the Web), this function never returns.
    ///
    /// # Arguments
    /// - `init_fn`: Some function that will be triggered to build say
    pub fn run(self) -> Result<(), Error> {
        match self.eloop.run(|event, window| match event {
            // A window event has been called
            Event::WindowEvent { window_id, event } => match event {
                _ => return,
            },

            // A tray icon event has been called
            Event::UserEvent(event) => {},

            // Ignore other events
            _ => return,
        }) {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::Run { err }),
        }
    }
}
