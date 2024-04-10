//  TRAY ICON.rs
//    by Lut99
//
//  Created:
//    10 Apr 2024, 11:15:30
//  Last edited:
//    10 Apr 2024, 11:35:15
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines an abstraction over backend tray icon providers.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::Path;

use image::ImageFormat;
use tray_icon::{Icon, TrayIconBuilder};

use super::event_loop::EventLoop;


/***** ERRORS *****/
/// Defines errors originating from the [`TrayIcon`].
#[derive(Debug)]
pub enum Error {
    /// Failed to convert the icon image from whatever it was embedded as to raw image data.
    ImageConvert { err: ConvertError },
    /// Failed to create a tray icon's [`Icon`].
    IconCreate { len: usize, dims: (u32, u32), err: tray_icon::BadIcon },
    /// Failed to create the [`tray_icon::TrayIcon`] itself.
    TrayIconCreate { err: tray_icon::Error },
}
impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            ImageConvert { .. } => write!(f, "Failed to convert icon image to raw format"),
            IconCreate { len, dims, .. } => write!(f, "Failed to create icon of {} bytes ({}x{} pixels)", len, dims.0, dims1),
            TrayIconCreate { .. } => write!(f, "Failed to create backend tray icon"),
        }
    }
}
impl error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            ImageConvert { err } => Some(err),
            IconCreate { err, .. } => Some(err),
            TrayIconCreate { err, .. } => Some(err),
        }
    }
}



/// Defines errors originating from converting image formats.
#[derive(Debug)]
pub enum ConvertError {}





/***** CONSTANTS *****/
/// The raw byte string that is the icon image.
const ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon/example-256x256.png"));





/***** LIBRARY *****/
pub struct TrayIcon {
    /// The tray icon we wrap.
    icon: tray_icon::TrayIcon,
}
impl TrayIcon {
    /// Helper function that can convert image formats to raw image data.
    ///
    /// Allocates a new byte vector with the raw information.
    ///
    /// # Arguments
    /// - `img`: Some binary image to convert.
    /// - `format`: Some format of the `img`. You can omit this to have the library make educated guesses.
    ///
    /// # Returns
    /// A tuple with the raw image data and its size, as two integers (width x height),
    ///
    /// # Errors
    /// This function may fail if the input `img` is incorrect for the chosen `format`. In addition, if you omitted `format`, this function can also error if it couldn't guess the image type.
    fn convert_image_to_raw(img: &[u8], format: Option<ImageFormat>) -> Result<(Vec<u8>, (u32, u32)), ConvertError> {}
}
impl TrayIcon {
    /// Constructor for the TrayIcon.
    ///
    /// # Arguments
    /// - `config_dir`: The root config directory that the icon will be written to.
    /// - `eloop`: An [`EventLoop`] that will listen to events triggered by this icon.
    ///
    /// # Returns
    /// A new TrayIcon that is ready to listen.
    ///
    /// # Errors
    /// This function errors if we failed to create the icon used for the tray icon, or if we failed to create the backend [`tray_icon::TrayIcon`] itself.
    pub fn new(config_dir: impl AsRef<Path>, eloop: &EventLoop) -> Result<Self, Error> {
        let config_dir: &Path = config_dir.as_ref();

        // Decompress the internal image
        let (raw, dims): (Vec<u8>, (u32, u32)) = match Self::convert_image_to_raw(ICON, None) {
            Ok(res) => res,
            Err(err) => return Err(Error::ImageConvert { err }),
        };

        // Load the icon icon
        let icon: Icon = match Icon::from_rgba(raw, dims.0, dims.1) {
            Ok(icon) => icon,
            Err(err) => return Err(Error::IconCreate { len: raw.len(), dims, err }),
        };

        // Build the tray icon
        let icon: tray_icon::TrayIcon = match TrayIconBuilder::new()
                .with_title(format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
                .with_icon(icon)
                // .with_menu(menu)
                .with_menu_on_left_click(true)
                .with_temp_dir_path(config_dir)
                .build()
        {
            Ok(icon) => icon,
            Err(err) => return Err(Error::TrayIconCreate { err }),
        };

        // Done, create ourselves
        Ok(Self { icon })
    }
}
