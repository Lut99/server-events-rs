//  WINDOW.rs
//    by Lut99
//
//  Created:
//    02 Apr 2024, 15:14:28
//  Last edited:
//    10 Apr 2024, 10:51:46
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements code for handling our [`Window`]-abstraction.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};

use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;


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
/// Currently, only [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe) is supported.
#[derive(Debug)]
pub struct Window {
    /// The wrapper [`eframe::Window`] we actually render on.
    window: winit::window::Window,
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
    pub fn new(event_loop: &EventLoop<()>, title: impl AsRef<str>) -> Result<Self, Error> {
        // Build the eframe window
        let title: &str = title.as_ref();
        let window: winit::window::Window = match WindowBuilder::new().with_title(title).build(&event_loop) {
            Ok(win) => win,
            Err(err) => return Err(Error::WindowCreate { title: title.into(), err }),
        };

        // Done, build self
        Ok(Self { window })
    }
}
