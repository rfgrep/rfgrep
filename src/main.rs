//! Main entry point for rfgrep
#![allow(clippy::uninlined_format_args)]
#![allow(dead_code)]
#![allow(clippy::op_ref)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::borrowed_box)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::new_without_default)]
#![allow(unused_assignments)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::print_literal)]
mod app;
mod app_simple;
mod cli;
mod config;
mod error;
mod file_types;
mod memory;
mod output_formats;
mod plugin_cli;
mod plugin_system;
mod processor;
mod search_algorithms;
mod streaming_search;
mod tui;
mod walker;

use crate::error::{Result as RfgrepResult, RfgrepError};
use clap::Parser;
use cli::*;
use env_logger::{Builder, Env, Target};
use std::fs;
use std::time::Instant;

#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

#[cfg(unix)]
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

fn main() {
    let result = main_inner();
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn main_inner() -> RfgrepResult<()> {
    #[cfg(unix)]
    {
        let shutdown_flag = &SHUTDOWN_REQUESTED;
        ctrlc::set_handler(move || {
            shutdown_flag.store(true, AtomicOrdering::SeqCst);
            eprintln!("\nShutdown requested, finishing current operations...");
        })
        .expect("Failed to set Ctrl-C handler");
    }

    let cli = Cli::parse();
    // setup_logging(&cli)?;

    let start_time = Instant::now();

    // Auto-detect if output is being piped
    let is_piped = !is_terminal::is_terminal(&std::io::stdout());

    let suppress_verbose = cli.quiet
        || is_piped
        || matches!(&cli.command, Commands::Search { output_format, ndjson, .. } if output_format == &cli::OutputFormat::Json || *ndjson);

    let verbose = cli.verbose;

    if !suppress_verbose && verbose {
        println!("Application started with command: {:?}", cli.command);
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let app = app_simple::RfgrepApp::new_async().await?;
        app.run(cli).await
    })?;

    if !suppress_verbose && verbose {
        println!(
            "Application finished. Total elapsed time: {:.2?}",
            start_time.elapsed()
        );
    }
    Ok(())
}

fn setup_logging(cli: &Cli) -> RfgrepResult<()> {
    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));

    builder.format(|buf, record| {
        use std::io::Write;
        writeln!(
            buf,
            "{} [{}] [{}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.module_path().unwrap_or("unknown"),
            record.args()
        )
    });

    if let Some(log_path) = &cli.log {
        if let Some(parent_dir) = log_path.parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(parent_dir).map_err(RfgrepError::Io)?;
            }
        }
        let log_file = fs::File::create(log_path).map_err(RfgrepError::Io)?;
        builder.target(Target::Pipe(Box::new(log_file)));
    } else {
        builder.target(Target::Stderr);
    }

    builder
        .try_init()
        .map_err(|e| RfgrepError::Other(e.to_string()))?;
    Ok(())
}
