//  TRAY ICON.rs
//    by Lut99
//
//  Created:
//    10 Apr 2024, 11:15:30
//  Last edited:
//    11 Apr 2024, 17:29:03
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines an abstraction over backend tray icon providers.
//

use std::fmt::{Display, Formatter, Result as FResult};
use std::path::{Path, PathBuf};
use std::{error, fs};

use egui_winit::winit;
use enum_debug::EnumDebug;
use error_trace::trace;
use image::{DynamicImage, GenericImageView, ImageFormat};
use log::{debug, info, warn};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use winit::event_loop::EventLoopProxy;

use crate::state::AppState;


/***** CONSTANTS *****/
/// The raw byte string that is the icon image.
const ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon/example-256x256.png"));

/// The number of check update commands that can be buffered.
#[cfg(target_os = "linux")]
const CHECK_BUFFER_LEN: usize = 32;





/***** ERRORS *****/
/// Defines errors originating from the [`TrayIcon`].
#[derive(Debug)]
pub enum Error {
    /// Failed to create the cache directory for the tray icon.
    CacheDirCreate { path: PathBuf, err: std::io::Error },
    /// Failed to convert the icon image from whatever it was embedded as to raw image data.
    ImageConvert { err: ConvertError },
    /// Failed to create a tray icon's [`Icon`].
    IconCreate { len: usize, dims: (u32, u32), err: tray_icon::BadIcon },
    /// Failed to create a menu item.
    MenuAppendItem { item: TrayIconMenuItem, err: tray_icon::menu::Error },
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
            CacheDirCreate { path, .. } => write!(f, "Failed to create cache directory '{}'", path.display()),
            ImageConvert { .. } => write!(f, "Failed to convert icon image to raw format"),
            IconCreate { len, dims, .. } => write!(f, "Failed to create icon of {} bytes ({}x{} pixels)", len, dims.0, dims.1),
            MenuAppendItem { item, .. } => write!(f, "Failed to append menu item {item} to menu"),
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
            CacheDirCreate { err, .. } => Some(err),
            ImageConvert { err } => Some(err),
            IconCreate { err, .. } => Some(err),
            MenuAppendItem { err, .. } => Some(err),
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





/***** AUXILLARY *****/
/// Defines the possible menu items in the [`TrayIcon`].
#[derive(Clone, Copy, Debug, EnumDebug, Eq, Hash, PartialEq)]
pub enum TrayIconMenuItem {
    /// It's the first item, `Open`.
    Open,
    /// It's the second item, `Exit`.
    Exit,
}
impl Display for TrayIconMenuItem {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        match self {
            Self::Open => write!(f, "Open"),
            Self::Exit => write!(f, "Exit"),
        }
    }
}



/// A handle for a TrayIcon that either visits it on another thread, or just wraps itself.
///
/// This version wraps itself.
#[cfg(not(target_os = "linux"))]
pub struct TrayIconHandle(TrayIcon);
#[cfg(not(target_os = "linux"))]
impl TrayIconHandle {
    /// Constructor for the TrayIconHandle that builds it from a direct TrayIcon.
    ///
    /// # Arguments
    /// - `tray_icon`: The [`TrayIcon`] to build ourselves from.
    ///
    /// # Returns
    /// A new TrayIconHandle that can do stuff.
    #[inline]
    pub fn new(tray_icon: TrayIcon) -> Self { Self(tray_icon) }

    /// Returns the [`MenuId`] of one of the given menu items.
    ///
    /// # Arguments
    /// - `item`: The item to get the ID of.
    ///
    /// # Returns
    /// Returns a [`MenuId`] that represents how to recognize this specific item.
    pub const fn get_id_of_item(&self, item: TrayIconMenuItem) -> &MenuId {
        match item {
            TrayIconMenuItem::Open => &self.0.ids[0],
            TrayIconMenuItem::Exit => &self.0.ids[1],
        }
    }
}





/***** LIBRARY *****/
pub struct TrayIcon {
    /// The tray icon we wrap.
    _icon: tray_icon::TrayIcon,
    /// The IDs for all of the menu items.
    ids:   [MenuId; 2],
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
    /// - `state`: The [`AppState`] shared with the window to communicate with each other.
    /// - `eloop`: An [`EventLoopProxy`] that acts as a handle to the main event proxy.
    ///
    /// # Returns
    /// A new TrayIcon that is ready to listen.
    ///
    /// # Errors
    /// This function errors if we failed to create the icon used for the tray icon, or if we failed to create the backend [`tray_icon::TrayIcon`] itself.
    pub fn new(state: AppState, eloop: EventLoopProxy<MenuEvent>) -> Result<Self, Error> {
        let config_dir: &Path = state.config_dir();
        info!("Initializing TrayIcon...");

        // Assert the cache directory exists
        if !state.cache_dir().exists() {
            debug!("Cache directory '{}' does not exist, creating...", state.cache_dir().display());
            if let Err(err) = fs::create_dir_all(&state.cache_dir()) {
                return Err(Error::CacheDirCreate { path: state.cache_dir().into(), err });
            }
        } else {
            debug!("Cache directory '{}' exists", state.config_dir().display());
        }

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

        // Build the items
        let open: MenuItem = MenuItem::new("&Open", true, None);
        let exit: MenuItem = MenuItem::new("&Exit", true, None);

        // Add them all into a menu
        let menu: Menu = Menu::new();
        menu.append(&open).map_err(|err| Error::MenuAppendItem { item: TrayIconMenuItem::Open, err })?;
        menu.append(&exit).map_err(|err| Error::MenuAppendItem { item: TrayIconMenuItem::Exit, err })?;

        // Build the tray icon
        debug!("Building backend TrayIcon...");
        let icon: tray_icon::TrayIcon = match TrayIconBuilder::new()
            .with_title(format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
            .with_icon(icon)
            .with_tooltip("server-events client")
            .with_menu(Box::new(menu))
            .with_temp_dir_path(config_dir)
            .build()
        {
            Ok(icon) => icon,
            Err(err) => return Err(Error::TrayIconCreate { err }),
        };

        // Register the handlers in the event loop
        debug!("Registering event handler for the menu...");
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let id: MenuId = event.id.clone();
            if let Err(err) = eloop.send_event(event) {
                warn!("{}", trace!(("Failed to raise MenuEvent with ID '{id:?}'"), err));
            }
        }));

        // Set some final options
        if let Err(err) = icon.set_visible(true) {
            return Err(Error::TrayIconVisible { err });
        }

        // Done, create ourselves
        Ok(Self { _icon: icon, ids: [open.into_id(), exit.into_id()] })
    }
}
