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

    pub fn test(args: &ArgMatches) -> Result<()> {
        Ok(())
    }
}

pub mod challenges {
    pub mod day1 {
        type MassUnit = u64;

        fn fuel_from_mass(mass: MassUnit) -> u64 {
            (mass - 6) / 3
        }

        pub fn total_fuel(modules: impl Iterator<Item = MassUnit>) -> u64 {
            modules.map(fuel_from_mass).sum()
        }

        #[cfg(test)]
        mod test {
            use crate::challenges::day1::fuel_from_mass;

            #[test]
            fn advent_examples() {
                assert_eq!(fuel_from_mass(12), 2);
                assert_eq!(fuel_from_mass(14), 2);
                assert_eq!(fuel_from_mass(1969), 654);
                assert_eq!(fuel_from_mass(100756), 33583);
            }
        }
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
