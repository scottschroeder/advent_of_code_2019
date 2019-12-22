use anyhow::{anyhow, Context, Result};

#[macro_use]
extern crate clap;

use clap::{Arg, ArgMatches, SubCommand};

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use slog::Drain;
use std::sync::Arc;

pub mod util {
    use anyhow::{anyhow, Context, Result};
    use std::fmt::Display;
    use std::str::FromStr;
    use std::{fs, io, io::BufRead, path};

    pub fn read_lines_of<T, P>(path: P) -> Result<Vec<T>>
    where
        T: FromStr,
        P: AsRef<path::Path>,
        <T as std::str::FromStr>::Err: Display,
    {
        slog_scope::trace!("Reading content of file: {}", path.as_ref().display());
        let mut f = fs::File::open(&path)
            .with_context(|| format!("Unable to open path: {}", path.as_ref().display()))?;

        let mut result = Vec::new();

        for line in io::BufReader::new(f).lines() {
            let text = line?;
            let t = T::from_str(&text).map_err(|e| anyhow!("{}", e))?;
            result.push(t);
        }

        Ok(result)
    }

    pub fn read_intcode<P: AsRef<path::Path>>(path: P) -> Result<Vec<u64>> {
        slog_scope::trace!("Reading content of file: {} as IntCode", path.as_ref().display());
        let mut f = fs::File::open(&path)
            .with_context(|| format!("Unable to open path: {}", path.as_ref().display()))?;

        let mut result = Vec::new();

        for line in io::BufReader::new(f).lines() {
            let text = line?;
            for ns in text.split(",") {
                let int = u64::from_str(ns).map_err(|e| anyhow!("{}", e))?;
                result.push(int)
            }
        }
        Ok(result)
    }
}

mod subcommand {
    use anyhow::{Context, Result};
    use clap::ArgMatches;

    pub fn day1(args: &ArgMatches) -> Result<()> {
        let modules: Vec<u64> = crate::util::read_lines_of(args.value_of("input").unwrap())?;
        let fuel = crate::challenges::day1::total_fuel(modules.into_iter());
        println!("{}", fuel);
        Ok(())
    }

    pub fn day1p2(args: &ArgMatches) -> Result<()> {
        let modules: Vec<u64> = crate::util::read_lines_of(args.value_of("input").unwrap())?;
        let fuel = crate::challenges::day1::total_fuel_recursive(modules.into_iter());
        println!("{}", fuel);
        Ok(())
    }

    pub fn day2(args: &ArgMatches) -> Result<()> {
        let mut intcode: Vec<u64> = crate::util::read_intcode(args.value_of("input").unwrap())?;
        intcode[1] = 12;
        intcode[2] = 02;
        let finished = crate::challenges::day2::run_intcode(intcode);
        println!("{}", finished[0]);
        Ok(())
    }

    pub fn test(args: &ArgMatches) -> Result<()> {
        Ok(())
    }
}

pub mod challenges {
    pub mod day1;
    pub mod day2;
}

fn run(args: &ArgMatches) -> Result<()> {
    let log = slog_scope::logger();

    trace!(log, "Args: {:?}", args);

    match args.subcommand() {
        ("day1", Some(sub_m)) => subcommand::day1(sub_m)?,
        ("day1-part2", Some(sub_m)) => subcommand::day1p2(sub_m)?,
        ("day2", Some(sub_m)) => subcommand::day2(sub_m)?,
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
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day1-part2")
                .about("Calculate fuel required: fuel requires more fuel")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day2")
                .about("1202 Program Alarm")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(SubCommand::with_name("test"))
        .get_matches()
}
