//  TRAY ICON.rs
//    by Lut99
//
//  Created:
//    10 Apr 2024, 11:15:30
//  Last edited:
//    10 Apr 2024, 13:57:32
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines an abstraction over backend tray icon providers.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::Path;

use error_trace::trace;
use image::{DynamicImage, GenericImageView, ImageFormat};
use log::{debug, info, warn};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use winit::event_loop::EventLoopProxy;


/***** ERRORS *****/
/// Defines errors originating from the [`TrayIcon`].
#[derive(Debug)]
pub enum Error {
    /// Failed to convert the icon image from whatever it was embedded as to raw image data.
    ImageConvert { err: ConvertError },
    /// Failed to create a tray icon's [`Icon`].
    IconCreate { len: usize, dims: (u32, u32), err: tray_icon::BadIcon },
    /// Failed to create a menu item.
    MenuCreateItem { item: usize, err: tray_icon::menu::Error },
    /// Failed to create the [`tray_icon::TrayIcon`] itself.
    TrayIconCreate { err: tray_icon::Error },
    /// Failed to make the tray icon visible.
    TrayIconVisible { err: tray_icon::Error },
}
impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            ImageConvert { .. } => write!(f, "Failed to convert icon image to raw format"),
            IconCreate { len, dims, .. } => write!(f, "Failed to create icon of {} bytes ({}x{} pixels)", len, dims.0, dims.1),
            MenuCreateItem { item, .. } => write!(f, "Failed to create menu item {item}"),
            TrayIconCreate { .. } => write!(f, "Failed to create backend tray icon"),
            TrayIconVisible { .. } => write!(f, "Failed make tray icon visible"),
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
            MenuCreateItem { err, .. } => Some(err),
            TrayIconCreate { err } => Some(err),
            TrayIconVisible { err } => Some(err),
        }
    }
}



/// Defines errors originating from converting image formats.
#[derive(Debug)]
pub struct ConvertError {
    format: Option<ImageFormat>,
    err:    image::error::ImageError,
}
impl Display for ConvertError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(
            f,
            "Failed to convert image{} to raw format",
            if let Some(format) = self.format { format!(" with format {format:?}") } else { String::new() }
        )
    }
}
impl error::Error for ConvertError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> { Some(&self.err) }
}





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
    /// A [`DynamicImage`] encoding the raw data.
    ///
    /// # Errors
    /// This function may fail if the input `img` is incorrect for the chosen `format`. In addition, if you omitted `format`, this function can also error if it couldn't guess the image type.
    fn convert_image_to_raw(img: &[u8], format: Option<ImageFormat>) -> Result<DynamicImage, ConvertError> {
        match format {
            Some(format) => {
                debug!("Loading image with format '{format:?}'...");
                match image::load_from_memory_with_format(img, format) {
                    Ok(img) => Ok(img),
                    Err(err) => Err(ConvertError { format: Some(format), err }),
                }
            },
            None => {
                debug!("Loading image with auto format...");
                match image::load_from_memory(img) {
                    Ok(img) => Ok(img),
                    Err(err) => Err(ConvertError { format: None, err }),
                }
            },
        }
    }
}
impl TrayIcon {
    /// Constructor for the TrayIcon.
    ///
    /// # Arguments
    /// - `config_dir`: The root config directory that the icon will be written to.
    /// - `eloop`: An [`EventLoopProxy`] that acts as a handle to the main event proxy.
    ///
    /// # Returns
    /// A new TrayIcon that is ready to listen.
    ///
    /// # Errors
    /// This function errors if we failed to create the icon used for the tray icon, or if we failed to create the backend [`tray_icon::TrayIcon`] itself.
    pub fn new(config_dir: impl AsRef<Path>, eloop: EventLoopProxy<MenuEvent>) -> Result<Self, Error> {
        let config_dir: &Path = config_dir.as_ref();
        info!("Initializing TrayIcon...");

        // Decompress the internal image
        debug!("Loading tray icon icon...");
        let raw: DynamicImage = match Self::convert_image_to_raw(ICON, None) {
            Ok(res) => res,
            Err(err) => return Err(Error::ImageConvert { err }),
        };

        // Load the icon icon
        let len: usize = raw.as_bytes().len();
        let dims: (u32, u32) = raw.dimensions();
        let icon: Icon = match Icon::from_rgba(raw.into_bytes(), dims.0, dims.1) {
            Ok(icon) => icon,
            Err(err) => return Err(Error::IconCreate { len, dims, err }),
        };

        // Build the menu
        debug!("Building tray icon menu...");
        let menu: Menu = Menu::new();
        if let Err(err) = menu.append(&MenuItem::new("Exit", true, None)) {
            return Err(Error::MenuCreateItem { item: 0, err });
        };

        // Build the tray icon
        debug!("Building backend TrayIcon...");
        let icon: tray_icon::TrayIcon = match TrayIconBuilder::new()
            .with_title(format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
            .with_icon(icon)
            .with_tooltip("server-events client")
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(true)
            .with_temp_dir_path(config_dir)
            .build()
        {
            Ok(icon) => icon,
            Err(err) => return Err(Error::TrayIconCreate { err }),
        };

        // Register the handler in the event loop
        debug!("Registering event handler...");
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let id: MenuId = event.id.clone();
            if let Err(err) = eloop.send_event(event) {
                warn!("{}", trace!(("Failed to raise MenuEvent with ID '{id:?}'"), err));
            }
        }));

        // Set some final options
        icon.set_show_menu_on_left_click(true);
        if let Err(err) = icon.set_visible(true) {
            return Err(Error::TrayIconVisible { err });
        }

        // Done, create ourselves
        Ok(Self { icon })
    }
}
