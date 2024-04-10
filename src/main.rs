//  MAIN.rs
//    by Lut99
//
//  Created:
//    02 Apr 2024, 15:08:52
//  Last edited:
//    10 Apr 2024, 11:04:22
//  Auto updated?
//    Yes
//
//  Description:
//!   Entrypoint for the `server-events` binary.
//

use clap::Parser;
use error_trace::trace;
use humanlog::{DebugMode, HumanLogger};
use log::{error, info};


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

    /* INITIALIZATION */
}
