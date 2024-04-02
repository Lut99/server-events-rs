//  WINDOW.rs
//    by Lut99
//
//  Created:
//    02 Apr 2024, 15:14:28
//  Last edited:
//    02 Apr 2024, 15:17:05
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements code for handling our [`Window`]-abstraction.
//


/***** LIBRARY *****/
/// Implements an abstraction of a Window over some backend.
///
/// Currently, only [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe) is supported.
#[derive(Debug)]
pub struct Window {
    /// The wrapper [`eframe::Window`] we actually render on.
    window: eframe::Window,
}
