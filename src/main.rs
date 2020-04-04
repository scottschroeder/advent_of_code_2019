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

pub mod util;

pub mod display;
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
    pub mod day11;
    pub mod day12;
    pub mod day13;
    pub mod day14;
    pub mod day15;
    pub mod day16;

    use anyhow::Result;
    use clap::ArgMatches;

    pub(crate) fn do_challenge(args: &ArgMatches) -> Result<()> {
        let day = args.value_of("day").unwrap().parse::<u32>()?;
        let part = args.value_of("part").unwrap().parse::<u32>()?;
        let input = crate::util::read_to_string(args.value_of("input").unwrap())?;

        debug!(slog_scope::logger(), "running day {}:{}", day, part);
        let result = match (day, part) {
            (1, 1) => day1::part1(&input)?,
            (1, 2) => day1::part2(&input)?,
            (2, 1) => day2::part1(&input)?,
            (2, 2) => day2::part2(&input)?,
            (3, 1) => day3::part1(&input)?,
            (3, 2) => day3::part2(&input)?,
            (4, 1) => day4::part1(&input)?,
            (4, 2) => day4::part2(&input)?,
            (5, 1) => day5::part1(&input)?,
            (5, 2) => day5::part2(&input)?,
            (6, 1) => day6::part1(&input)?,
            (6, 2) => day6::part2(&input)?,
            (7, 1) => day7::part1(&input)?,
            (7, 2) => day7::part2(&input)?,
            (8, 1) => day8::part1(&input)?,
            (8, 2) => day8::part2(&input)?,
            (9, 1) => day9::part1(&input)?,
            (9, 2) => day9::part2(&input)?,
            (10, 1) => day10::part1(&input)?,
            (10, 2) => day10::part2(&input)?,
            (11, 1) => day11::part1(&input)?,
            (11, 2) => day11::part2(&input)?,
            (12, 1) => day12::part1(&input)?,
            (12, 2) => day12::part2(&input)?,
            (13, 1) => day13::part1(&input)?,
            (13, 2) => day13::part2(&input)?,
            (14, 1) => day14::part1(&input)?,
            (14, 2) => day14::part2(&input)?,
            (15, 1) => day15::part1(&input)?,
            (15, 2) => day15::part2(&input)?,
            (16, 1) => day16::part1(&input)?,
            (16, 2) => day16::part2(&input)?,
            (d, p) => {
                return Err(anyhow::anyhow!(
                    "unimplemented challenge day {} part {}",
                    d,
                    p
                ))
            }
        };
        println!("{}", result);
        Ok(())
    }

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
        pub const DAY11_INPUT: &str = include_str!("../input/day11");
        pub const DAY11_PART2_OUTPUT: &str = include_str!("../input/day11_part2_output");
        pub const DAY12_INPUT: &str = include_str!("../input/day12");
        pub const DAY12_EX1: &str = include_str!("../input/day12_ex1");
        pub const DAY12_EX2: &str = include_str!("../input/day12_ex2");
        pub const DAY13_INPUT: &str = include_str!("../input/day13");
        pub const DAY14_INPUT: &str = include_str!("../input/day14");
        pub const DAY15_INPUT: &str = include_str!("../input/day15");
        pub const DAY16_INPUT: &str = include_str!("../input/day16");
    }
}

fn run(args: &ArgMatches) -> Result<()> {
    let log = slog_scope::logger();

    trace!(log, "Args: {:?}", args);

    match args.subcommand() {
        ("challenge", Some(sub_m)) => crate::challenges::do_challenge(sub_m)?,
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
            SubCommand::with_name("challenge")
                .about("run one of the daily challenges")
                .arg(Arg::with_name("day").required(true))
                .arg(Arg::with_name("part").required(true))
                .arg(Arg::with_name("input").required(true)),
        )
        .subcommand(SubCommand::with_name("test"))
        .get_matches()
}
