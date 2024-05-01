//  MAIN.rs
//    by Lut99
//
//  Created:
//    02 Apr 2024, 15:08:52
//  Last edited:
//    01 May 2024, 19:51:49
//  Auto updated?
//    Yes
//
//  Description:
//!   Entrypoint for the `server-events` binary.
//

use std::ffi::OsString;
use std::path::PathBuf;

use clap::Parser;
use error_trace::ErrorTrace as _;
use humanlog::{DebugMode, HumanLogger};
use lazy_static::lazy_static;
use log::{error, info};
use server_events::app::app::App;


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

    // Build the app
    let app: App = match App::new(args.config_dir) {
        Ok(app) => app,
        Err(err) => {
            error!("{}", err.trace());
            std::process::exit(1);
        },
    };

    // Then run it for all it's worth
    if let Err(err) = app.run() {
        error!("{}", err.trace());
        std::process::exit(1);
    }

    // Done
    info!("Bye.");
}
