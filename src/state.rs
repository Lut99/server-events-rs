//  STATE.rs
//    by Lut99
//
//  Created:
//    11 Apr 2024, 13:14:35
//  Last edited:
//    11 Apr 2024, 17:13:35
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines some shared state for the whole app.
//

use std::fmt::{Display, Formatter, Result as FResult};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{error, fs};

use chrono::{DateTime, Local};
use enum_debug::EnumDebug;
use log::{debug, info};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{Deserialize, Serialize};
use serializable::toml::{Error as TomlError, Serializer as Toml};
use serializable::Serializable;


/***** ERRORS *****/
/// Defines errors originating from [`AppState`]s.
#[derive(Debug)]
pub enum Error {
    /// Failed to create the config parent directory.
    ConfigDirCreate { path: PathBuf, err: std::io::Error },
    /// Failed to load the given config file.
    ConfigLoad { path: PathBuf, err: serializable::Error<TomlError> },
    /// Failed to write a config file.
    ConfigWrite { path: PathBuf, err: serializable::Error<TomlError> },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            ConfigDirCreate { path, .. } => write!(f, "Failed to create config directory '{}'", path.display()),
            ConfigLoad { path, .. } => write!(f, "Failed to load config file '{}'", path.display()),
            ConfigWrite { path, .. } => write!(f, "Failed to write config file to '{}'", path.display()),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            ConfigDirCreate { err, .. } => Some(err),
            ConfigLoad { err, .. } => Some(err),
            ConfigWrite { err, .. } => Some(err),
        }
    }
}





/***** AUXILLARY *****/
/// Describes the general config file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    /// The current mute state.
    pub muted: MuteState,
}
impl Default for ConfigFile {
    fn default() -> Self { Self { muted: MuteState::Unmuted } }
}
impl Serializable<Toml<ConfigFile>> for ConfigFile {}



/// Describes if we're muted and, if so, when we're unmuted again.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MuteState {
    /// Not muted.
    Unmuted,
    /// Resuming once the given timestamp has been passed.
    After(DateTime<Local>),
    /// Resuming next time on load.
    NextBoot,
    /// No automatic end specified, waiting for manual intervention.
    Manual,
}
impl MuteState {
    /// Checks whether this refers to some muted state.
    ///
    /// # Returns
    /// True for all variants except [`MuteState::Unmuted`].
    #[inline]
    pub fn is_muted(&self) -> bool { !matches!(self, MuteState::Unmuted) }

    /// Checks whether this refers to some unmuted state.
    ///
    /// # Returns
    /// True only for [`MuteState::Unmuted`].
    #[inline]
    pub fn is_unmuted(&self) -> bool { matches!(self, MuteState::Unmuted) }
}





/***** LIBRARY *****/
/// Defines some shared state for the whole app.
///
/// Actually just wraps [`MutableAppState`] plus some non-mutable stuff with some locks and junk.
#[derive(Clone, Debug)]
pub struct AppState {
    // Immutable part
    /// The location of all the config files.
    config_dir: PathBuf,
    /// The location of any chache files (e.g., tray icon thumbnail).
    cache_dir:  PathBuf,

    // Mutable part
    /// The mutable part of the app state behind a lock.
    mut_state: Arc<RwLock<MutableAppState>>,
}
impl AppState {
    /// Constructor for the AppState.
    ///
    /// # Arguments
    /// - `config_dir`: The directory where the app's persistent config is stored.
    ///
    /// # Returns
    /// A new AppState that is ready to be used in the app.
    ///
    /// # Errors
    /// This function may error if we failed to load any of the persistent state in the `config_dir`.
    #[inline]
    pub fn new(config_dir: PathBuf) -> Result<Self, Error> {
        info!("Initializing AppState...");

        // Generate additional paths
        let cache_dir: PathBuf = config_dir.join("cache");

        // Build the mutable state
        let mut_state: Arc<RwLock<MutableAppState>> = Arc::new(RwLock::new(MutableAppState::new(&config_dir)?));

        // OK, build self
        Ok(Self { config_dir, cache_dir, mut_state })
    }

    /// Returns the path of the persistent configuration for the app.
    ///
    /// # Returns
    /// A [`Path`] describing where to find the folder.
    #[inline]
    pub fn config_dir(&self) -> &Path { &self.config_dir }

    /// Returns the path of the temporary cache for the app.
    ///
    /// # Returns
    /// A [`Path`] describing where to find the folder.
    #[inline]
    pub fn cache_dir(&self) -> &Path { &self.cache_dir }

    /// Provides read-only access to the mutable part of the state.
    ///
    /// # Arguments
    /// - `access_fn`: Some closure that can access the contents of the mutable app state.
    ///
    /// # Errors
    /// This function errors if the given `access_fn` errors.
    #[inline]
    pub fn access<R>(&self, access_fn: impl FnOnce(&MutableAppState) -> R) -> R {
        let lock: RwLockReadGuard<MutableAppState> = self.mut_state.read();
        access_fn(&*lock)
    }

    /// Provides write access to the mutable part of the state.
    ///
    /// At the end of this function, if the closure did not error, then the disk is updated with the mutated state.
    ///
    /// # Arguments
    /// - `access_fn`: Some closure that can access the contents of the mutable app state.
    ///
    /// # Errors
    /// This function errors if the given `access_fn` errors, or then if writing the state back errors.
    #[inline]
    pub fn access_mut<R, E>(&self, access_fn: impl FnOnce(&mut MutableAppState) -> Result<R, E>) -> Result<Result<R, Error>, E> {
        // Provide mutable access, with its own, unique lock
        let res: R = {
            let mut lock: RwLockWriteGuard<MutableAppState> = self.mut_state.write();
            access_fn(&mut *lock)?
        };

        // Now sync the mutable app state back
        let config_path: PathBuf = self.config_dir.join("server_events.toml");
        if let Err(err) = self.mut_state.read().sync(&config_path) {
            return Ok(Err(err));
        }

        // OK, return the result
        Ok(Ok(res))
    }
}



/// The actual [`AppState`] without locks and all that.
#[derive(Debug)]
pub struct MutableAppState {
    /// Whether notifcations are muted or not and, if not, how to unmute ourselves.
    pub muted: MuteState,
}
impl MutableAppState {
    /// Syncs this MutableAppState back to the disk.
    ///
    /// This is recommended to be called at the end of every lock if a change occurred in order to ensure the disk is up-to-date.
    ///
    /// # Arguments
    /// - `config_path`: The path to write the config file to.
    ///
    /// # Errors
    /// This function may error if it failed to write to disk.
    fn sync(&self, config_path: &Path) -> Result<(), Error> {
        // Build a config file
        let config: ConfigFile = ConfigFile { muted: self.muted.clone() };

        // Check if the target directory exists
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                debug!("Config directory '{}' does not exist, creating...", parent.display());
                if let Err(err) = fs::create_dir_all(parent) {
                    return Err(Error::ConfigDirCreate { path: parent.into(), err });
                }
            }
        }

        // Write it to that path
        debug!("Syncing MutableAppState back to '{}'...", config_path.display());
        match config.to_path_pretty(config_path) {
            Ok(_) => {
                info!("Synced MutableAppState back to '{}'", config_path.display());
                Ok(())
            },
            Err(err) => Err(Error::ConfigWrite { path: config_path.into(), err }),
        }
    }
}
impl MutableAppState {
    /// Constructor for the MutableAppState.
    ///
    /// # Arguments
    /// - `config_dir`: The directory where the app's persistent config is stored.
    ///
    /// # Returns
    /// A new MutableAppState that is ready to be used in the app.
    ///
    /// # Errors
    /// This function may error if we failed to load any of the persistent state in the `config_dir`.
    pub fn new(config_dir: &PathBuf) -> Result<Self, Error> {
        info!("Initializing MutableAppState...");

        // Attempt to load the config file
        let config_path: PathBuf = config_dir.join("server_events.toml");
        debug!("Loading config file from '{}'...", config_path.display());
        let mut config: ConfigFile = match ConfigFile::from_path(&config_path) {
            Ok(config) => config,
            Err(serializable::Error::FileOpen { path, err }) => {
                if err.kind() == ErrorKind::NotFound {
                    // Use a default thing instead (we'll catch it on the next resync)
                    debug!("Config file '{}' not found; using default", config_path.display());
                    ConfigFile::default()
                } else {
                    return Err(Error::ConfigLoad { path: config_path, err: serializable::Error::FileOpen { path, err } });
                }
            },
            Err(err) => return Err(Error::ConfigLoad { path: config_path, err }),
        };

        // Resolve the muted state in case it was supposed to last until the last exit
        if matches!(config.muted, MuteState::NextBoot) {
            config.muted = MuteState::Unmuted;
        }

        // OK, build self
        Ok(Self { muted: config.muted })
    }
}
