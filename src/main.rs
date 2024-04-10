//  MAIN.rs
//    by Lut99
//
//  Created:
//    02 Apr 2024, 15:08:52
//  Last edited:
//    10 Apr 2024, 14:54:04
//  Auto updated?
//    Yes
//
//  Description:
//!   Entrypoint for the `server-events` binary.
//

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use eframe::EventLoopBuilder;
use error_trace::trace;
use humanlog::{DebugMode, HumanLogger};
use lazy_static::lazy_static;
use log::{debug, error, info};
use server_events::gui::tray_icon::TrayIcon;
use server_events::gui::window::Window;
use tray_icon::menu::MenuEvent;
use winit::event::{Event, StartCause};
use winit::event_loop::{EventLoop, EventLoopProxy};


/***** STATICS *****/
lazy_static! {
    /// The config dir we load once.
    static ref DEFAULT_CONFIG_DIR: OsString = dirs::config_dir().unwrap_or_else(|| "/config".into()).join("server-events").into_os_string();
}





/***** ARGUMENTS *****/
/// Defines the arguments for this binary.
#[derive(Debug, Parser)]
struct Arguments {
    /// If given, enables additional log statements (INFO- and DEBUG-levels).
    #[clap(long, global = true)]
    debug: bool,
    /// If given, enables maximum log statements (INFO-, DEBUG- and TRACE-levels).
    #[clap(long, global = true)]
    trace: bool,

    /// The persistent config directory that we use to keep settings and stuff in.
    #[clap(short, long, default_value = DEFAULT_CONFIG_DIR.as_os_str())]
    config_dir: PathBuf,
}





/***** ENTRYPOINT *****/
fn main() {
    // Parse arguments
    let args = Arguments::parse();

    // Setup the logger
    if let Err(err) = HumanLogger::terminal(DebugMode::from_flags(args.trace, args.debug)).init() {
        eprintln!("WARNING: Failed to setup logger: {err} (no logging for this session)");
    }
    info!("{} v{}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));



    /* PREPARATION */
    // Check if the config dir exists, and make it if it doesn't
    let cache_dir: PathBuf = args.config_dir.join("cache");
    if !args.config_dir.exists() {
        debug!("Config directory '{}' does not exist, creating...", args.config_dir.display());
        if let Err(err) = fs::create_dir_all(&args.config_dir) {
            error!("{}", trace!(("Failed to create config directory '{}'", args.config_dir.display()), err));
            std::process::exit(1);
        }

        // Also check some nested directories
        if !cache_dir.exists() {
            if let Err(err) = fs::create_dir_all(&cache_dir) {
                error!("{}", trace!(("Failed to create config cache directory '{}'", cache_dir.display()), err));
                std::process::exit(1);
            }
        }
    } else {
        debug!("Config directory '{}' exists", args.config_dir.display());
    }



    /* INITIALIZATION */
    // Create an EventLoop
    let eloop: EventLoop<MenuEvent> = match EventLoopBuilder::with_user_event().build() {
        Ok(eloop) => eloop,
        Err(err) => {
            error!("{}", trace!(("Failed to create event loop"), err));
            std::process::exit(1);
        },
    };

    // // Build a window
    // let window: Window = match Window::new() {
    //     Ok(window) => window,
    //     Err(err) => {
    //         error!("{}", trace!(("Failed to create window"), err));
    //         std::process::exit(1);
    //     },
    // };



    /* EVENT LOOP */
    info!("Initialization complete, running event loop...");
    let mut proxy: Option<EventLoopProxy<MenuEvent>> = Some(eloop.create_proxy());
    let mut tray_icon: Option<TrayIcon> = None;
    if let Err(err) = eloop.run(move |event, eloop| {
        match event {
            Event::NewEvents(StartCause::Init) => {
                // Build the tray icon here to implicitly register events on this event loop
                #[cfg(target_os = "linux")]
                {
                    // For Linux, this actually requires us to boot up that sweet GTK
                    let config_dir: PathBuf = args.config_dir.clone();
                    let proxy: EventLoopProxy<MenuEvent> = proxy.take().unwrap();
                    std::thread::spawn(move || {
                        debug!("Initializing GTK...");
                        if let Err(err) = gtk::init() {
                            error!("{}", trace!(("Failed to initialize GTK"), err));
                            return;
                        }

                        // Build the icon
                        let _icon: TrayIcon = match TrayIcon::new(config_dir, proxy) {
                            Ok(icon) => icon,
                            Err(err) => {
                                error!("{}", trace!(("Failed to create new TrayIcon"), err));
                                return;
                            },
                        };

                        // Then delegate the rest to GTK's main
                        debug!("Running GTK main loop");
                        gtk::main();
                    });
                }

                // Else, populate the tray icon
                #[cfg(not(target_os = "linux"))]
                {
                    tray_icon = Some(match TrayIcon::new(&args.config_dir, proxy.take().unwrap()) {
                        Ok(icon) => icon,
                        Err(err) => {
                            error!("{}", trace!(("Failed to create new TrayIcon"), err));
                            eloop.exit();
                            return;
                        },
                    });
                }
            },

            // Window events
            // Event::Window() => {},

            // Tray events
            Event::UserEvent(event) => {
                debug!("Got event from '{:?}'", event);
            },

            // Rest we ignore, for now
            _ => return,
        }
    }) {
        error!("{}", trace!(("Failed to run event loop"), err));
        std::process::exit(1);
    }
    info!("Bye.");
}
