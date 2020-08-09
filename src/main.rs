use anyhow::{anyhow as ah, Result};

#[macro_use]
extern crate clap;

use clap::{Arg, ArgMatches, SubCommand};

extern crate log;

pub mod util;
pub mod graph;

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
    pub mod day17;
    pub mod day18;
    pub mod day19;

    pub mod day20;

    use anyhow::Result;
    use clap::ArgMatches;

    pub(crate) fn do_challenge(args: &ArgMatches) -> Result<()> {
        let day = args.value_of("day").unwrap().parse::<u32>()?;
        let part = args.value_of("part").unwrap().parse::<u32>()?;
        let input = crate::util::read_to_string(args.value_of("input").unwrap())?;

        log::debug!("running day {}:{}", day, part);
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
            (17, 1) => day17::part1(&input)?,
            (17, 2) => day17::part2(&input)?,
            (17, 3) => day17::part2_map(&input)?,
            (18, 1) => day18::part1(&input)?,
            (18, 2) => day18::part2(&input)?,
            (19, 1) => day19::part1(&input)?,
            (19, 2) => day19::part2(&input)?,
            (20, 1) => day20::part1(&input)?,
            (20, 2) => day20::part2(&input)?,
            (20, 3) => day20::part3(&input)?,
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
        pub const DAY17_INPUT: &str = include_str!("../input/day17");
        pub const DAY18_INPUT: &str = include_str!("../input/day18");
        pub const DAY18_EX1: &str = include_str!("../input/day18_ex1");
        pub const DAY18_EX2: &str = include_str!("../input/day18_ex2");
        pub const DAY18_EX3: &str = include_str!("../input/day18_ex3");
        pub const DAY18_EX4: &str = include_str!("../input/day18_ex4");
        pub const DAY18_EX5: &str = include_str!("../input/day18_ex5");
        pub const DAY18_EX6: &str = include_str!("../input/day18_ex6");
        pub const DAY18_EX7: &str = include_str!("../input/day18_ex7");
        pub const DAY18_EX8: &str = include_str!("../input/day18_ex8");
        pub const DAY18_EX9: &str = include_str!("../input/day18_ex9");
        pub const DAY19_INPUT: &str = include_str!("../input/day19");
        pub const DAY20_INPUT: &str = include_str!("../input/day20");
        pub const DAY20_EX1: &str = include_str!("../input/day20_ex1");
        pub const DAY20_EX2: &str = include_str!("../input/day20_ex2");
        pub const DAY20_EX3: &str = include_str!("../input/day20_ex3");
    }
}

fn run(args: &ArgMatches) -> Result<()> {
    log::trace!("Args: {:?}", args);

    match args.subcommand() {
        ("challenge", Some(sub_m)) => crate::challenges::do_challenge(sub_m)?,
        ("", _) => return Err(ah!("Please provide a command:\n{}", args.usage())),
        subc => return Err(ah!("Unknown command: {:?}\n{}", subc, args.usage())),
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = get_args();
    setup_logger(args.occurrences_of("verbosity"));
    run(&args)
}

fn setup_logger(level: u64) {
    let noisy_modules = &[
        "hyper",
        "mio",
        "tokio_core",
        "tokio_reactor",
        "tokio_threadpool",
        "fuse::request",
        "rusoto_core",
        "want",
    ];

    let log_level = match level {
        //0 => log::Level::Error,
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    let mut builder = pretty_env_logger::formatted_timed_builder();
    if level > 1 && level < 4 {
        for module in noisy_modules {
            builder.filter_module(module, log::LevelFilter::Info);
        }
    }

    builder.filter_level(log_level);
    builder.format_module_path(false);
    //builder.format_timestamp_millis();
    //builder.format(|buf, record| writeln!(buf, "{}", record.args()));

    builder.init();
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
