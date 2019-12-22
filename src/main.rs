use anyhow::{Context, Result, anyhow};
#[macro_use]
extern crate clap;
use clap::{Arg, SubCommand, ArgMatches};

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use slog::Drain;
use std::sync::Arc;

mod subcommand {
    use clap::ArgMatches;
    use anyhow::{Context, Result};
    pub fn day1(args: &ArgMatches) -> Result<()> {
        Ok(())
    }

    pub fn test(args: &ArgMatches) -> Result<()> {
        Ok(())
    }

}

fn run(args: &ArgMatches) -> Result<()> {
    let log = slog_scope::logger();

    trace!(log, "Args: {:?}", args);

    match args.subcommand() {
        ("day1", Some(sub_m)) => subcommand::day1(sub_m)?,
        ("test", Some(sub_m)) => subcommand::test(sub_m)?,
        ("", _) => Err(anyhow!("Please provide a command:\n{}", args.usage()))?,
        subc => Err(anyhow!("Unknown command: {:?}\n{}", subc, args.usage()))?,
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = get_args();

    // Setup logger
    let _guard = slog_scope::set_global_logger(setup_logger(args.occurrences_of("verbosity")));

    run(&args)
}

fn setup_logger(level: u64) -> slog::Logger {

    let log_level = match level {
        0 => slog::Level::Warning,
        1 => slog::Level::Info,
        2 => slog::Level::Debug,
        _ => slog::Level::Trace,
    };
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = drain.filter_level(log_level).fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(Arc::new(drain), o!("version" => "0.5"))
}

fn get_args() -> clap::ArgMatches<'static> {
    clap::App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(clap::AppSettings::DeriveDisplayOrder)
        .arg(
            clap::Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .global(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            SubCommand::with_name("day1")
                .about("Calculate fuel required")
                .arg(
                    Arg::with_name("input")
                        .required(true)
                        .help("print debug information verbosely"),
                ),
        )
        .subcommand(SubCommand::with_name("test"))
        .get_matches()
}