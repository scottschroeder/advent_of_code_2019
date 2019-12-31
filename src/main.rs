use anyhow::{anyhow as ah, Result};

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
    use anyhow::{anyhow as ah, Context, Result};
    use std::fmt::Display;
    use std::io::Read;
    use std::str::FromStr;
    use std::{fs, path};

    pub fn parse_str<T>(s: &str) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        T::from_str(s).map_err(|e| ah!("{}", e))
    }

    pub fn read_to_string<P: AsRef<path::Path>>(path: P) -> Result<String> {
        slog_scope::trace!("Reading content of file: {}", path.as_ref().display());
        let mut f = fs::File::open(&path)
            .with_context(|| format!("Unable to open path: {}", path.as_ref().display()))?;

        let mut result = String::new();

        f.read_to_string(&mut result)?;
        Ok(result)
    }

    pub fn parse_int_lines(input: &str) -> Result<Vec<u64>> {
        input.lines().map(|l| parse_str::<u64>(l)).collect()
    }

    pub fn parse_intcode(input: &str) -> Result<Vec<i64>> {
        input
            .lines()
            .flat_map(|l| l.split(','))
            .map(|ns| parse_str::<i64>(ns))
            .collect()
    }
}

pub mod intcode;
pub mod orbital_data;

pub mod challenges {
    pub mod day1;
    pub mod day2;
    pub mod day3;
    pub mod day4;
    pub mod day5;
    pub mod day6;
    pub mod day7;
    pub mod day8;
    pub mod day9;
    pub mod day10;

    #[cfg(test)]
    mod test {
        pub const DAY1_INPUT: &str = include_str!("../input/day1");
        pub const DAY2_INPUT: &str = include_str!("../input/day2");
        pub const DAY3_INPUT: &str = include_str!("../input/day3");
        pub const DAY4_INPUT: &str = include_str!("../input/day4");
        pub const DAY5_INPUT: &str = include_str!("../input/day5");
        pub const DAY6_INPUT: &str = include_str!("../input/day6");
        pub const DAY6_EXAMPLE_INPUT: &str = include_str!("../input/day6_ex");
        pub const DAY7_INPUT: &str = include_str!("../input/day7");
        pub const DAY8_INPUT: &str = include_str!("../input/day8");
        pub const DAY8_PART2_OUTPUT: &str = include_str!("../input/day8_part2_output");
        pub const DAY9_INPUT: &str = include_str!("../input/day9");
        pub const DAY10_INPUT: &str = include_str!("../input/day10");
    }
}

fn run(args: &ArgMatches) -> Result<()> {
    let log = slog_scope::logger();

    trace!(log, "Args: {:?}", args);

    match args.subcommand() {
        ("day1", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day1::day1_part1(&input)?);
        },
        ("day1-part2", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day1::day1_part2(&input)?);
        },
        ("day2", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day2::day2_part1(&input)?);
        },
        ("day2-part2", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day2::day2_part2(&input)?);
        },
        ("day3", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day3::day3_part1(&input)?);
        },
        ("day3-part2", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day3::day3_part2(&input)?);
        },
        ("day4", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day4::day4_part1(&input)?);
        },
        ("day4-part2", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day4::day4_part2(&input)?);
        },
        ("day5", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day5::day5_part1(&input)?);
        },
        ("day5-part2", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day5::day5_part2(&input)?);
        },
        ("day6", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day6::day6_part1(&input)?);
        },
        ("day6-part2", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day6::day6_part2(&input)?);
        },
        ("day7", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day7::day7_part1(&input)?);
        },
        ("day7-part2", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day7::day7_part2(&input)?);
        },
        ("day8", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day8::day8_part1(&input)?);
        },
        ("day8-part2", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day8::day8_part2(&input)?);
        },
        ("day9", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day9::day9_part1(&input)?);
        },
        ("day9-part2", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day9::day9_part2(&input)?);
        },
        ("day10", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day10::part1(&input)?);
        },
        ("day10-part2", Some(sub_m)) => {
            let input = crate::util::read_to_string(sub_m.value_of("input").unwrap())?;
            println!("{}", crate::challenges::day10::part2(&input)?);
        },
        ("", _) => return Err(ah!("Please provide a command:\n{}", args.usage())),
        subc => return Err(ah!("Unknown command: {:?}\n{}", subc, args.usage())),
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
    let drain = std::sync::Mutex::new(drain).fuse();
    //    let drain = slog_async::Async::new(drain)
    //        .chan_size(1 << 10)
    //        .build()
    //        .fuse();
    slog::Logger::root(Arc::new(drain), o!())
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
        .subcommand(
            SubCommand::with_name("day2-part2")
                .about("solve inputs for gravity assist")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day3")
                .about("find closest wire crossing")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day3-part2")
                .about("find closest wire crossing by signal distance")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day4")
                .about("how many valid passwords")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day4-part2")
                .about("how many valid passwords part2")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day5")
                .about("TEST diagnostic - AC")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day5-part2")
                .about("TEST diagnostic - thermal regulator")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day6")
                .about("Count orbits in data")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day6-part2")
                .about("Shortest path orbital transfer")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day7")
                .about("Max amplifier settings")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day7-part2")
                .about("Max amplifier settings with feedback")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day8")
                .about("Space Image Format Checksum")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day8-part2")
                .about("Space Image Format Printing")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day9")
                .about("")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day9-part2")
                .about("")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day10")
                .about("")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(
            SubCommand::with_name("day10-part2")
                .about("")
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(SubCommand::with_name("test"))
        .get_matches()
}
