//  EVENT LOOP.rs
//    by Lut99
//
//  Created:
//    10 Apr 2024, 11:03:11
//  Last edited:
//    10 Apr 2024, 11:14:52
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines a wrapper around a [`winit`] event loop to suit our own
//!   needs.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};

use tray_icon::menu::MenuEvent;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoopBuilder;


/***** ERRORS *****/
/// The errors that originate from [`EventLoop`]s.
#[derive(Debug)]
pub enum Error {
    /// Failed to create a new [`winit::EventLoop`](winit::event_loop::EventLoop).
    EventLoopCreate { err: winit::error::EventLoopError },
    /// Failed to run the [`winit::EventLoop`](winit::event_loop::EventLoop).
    EventLoopRun { err: winit::error::EventLoopError },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            EventLoopCreate { .. } => write!(f, "Failed to create new winit EventLoop"),
            EventLoopRun { .. } => write!(f, "Failed to run winit EventLoop"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            EventLoopCreate { err } => Some(err),
            EventLoopRun { err } => Some(err),
        }
    }
}





/***** LIBRARY *****/
/// Our own [`winit::event_loop::EventLoop`] wrapper.
#[derive(Debug)]
pub struct EventLoop {
    /// The loop we wrap.
    eloop: winit::event_loop::EventLoop<MenuEvent>,
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
        // Build the winit event loop
        let eloop: winit::event_loop::EventLoop<MenuEvent> = match EventLoopBuilder::with_user_event().build() {
            Ok(eloop) => eloop,
            Err(err) => return Err(Error::EventLoopCreate { err }),
        };

        // Done
        Ok(Self { eloop })
    }

    /// Runs this event loop.
    ///
    /// This will hijack the current thread to run the event loop. On some platforms (iOS and the Web), this function never returns.
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
            Err(err) => Err(Error::EventLoopRun { err }),
        }
    }
}
