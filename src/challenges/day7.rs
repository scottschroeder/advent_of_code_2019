use crate::challenges::day7::amplifier::AmplifierCircut;
use crate::intcode::Int;
use crate::util::parse_intcode;
use anyhow::{anyhow as ah, Result};
use itertools::Itertools;

pub fn day7_part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let m = (0..5)
        .permutations(5)
        .map(|phases| {
            let c = AmplifierCircut::new(&intcode, 0, &phases);
            c.run().unwrap()[0]
        })
        .max()
        .unwrap();
    Ok(format!("{}", m))
}

pub fn day7_part2(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let m = (5..10)
        .permutations(5)
        .map(|phases| {
            let c = AmplifierCircut::new(&intcode, 0, &phases);
            let out = c.run().unwrap();
            *out.last().unwrap()
        })
        .max()
        .unwrap();
    Ok(format!("{}", m))
}

mod amplifier {
    use crate::intcode::intcode_io::{
        create_stream_io, create_stream_tap, Input, MultiIO, Output, VecIO,
    };
    use crate::intcode::{Int, IntCode};
    use anyhow::Result;

    pub struct Amplifier {
        intcode: Vec<Int>,
        phase: i64,
        input: Box<dyn Input + Send>,
        output: Box<dyn Output + Send>,
    }

    impl Amplifier {
        pub fn new<I: Input + 'static + Send, O: Output + 'static + Send>(
            intcode: Vec<Int>,
            phase: Int,
            input: I,
            output: O,
        ) -> Amplifier {
            let input = Box::new(input);
            let output = Box::new(output);
            Amplifier {
                intcode,
                phase,
                input,
                output,
            }
        }

        pub fn run(self) -> Result<()> {
            let fuse = MultiIO::new(VecIO::input(vec![self.phase]), self.input);
            let mut ic = IntCode::new(self.intcode, fuse, self.output);
            ic.run_till_end()?;
            Ok(())
        }
    }

    pub struct AmplifierCircut {
        amps: Vec<Amplifier>,
        out: std::sync::mpsc::Receiver<Int>,
    }

    impl AmplifierCircut {
        pub fn new(intcode: &[i64], input: Int, phases: &[i64]) -> AmplifierCircut {
            let (sys_in, sys_out, out) = create_sys_io(input);
            let (second_in, first_out) = create_stream_io();
            let mut input = Some(second_in);

            let mut amps = Vec::with_capacity(phases.len());

            amps.push(Amplifier::new(
                intcode.to_vec(),
                phases[0],
                sys_in,
                first_out,
            ));

            for phase in &phases[1..phases.len() - 1] {
                let (next_in, this_out) = create_stream_io();
                let this_in = input.replace(next_in).unwrap();
                amps.push(Amplifier::new(intcode.to_vec(), *phase, this_in, this_out));
            }

            amps.push(Amplifier::new(
                intcode.to_vec(),
                phases[phases.len() - 1],
                input.take().unwrap(),
                sys_out,
            ));

            AmplifierCircut { amps, out }
        }
        pub fn run(self) -> Result<Vec<Int>> {
            let mut threads = Vec::new();
            for (idx, amp) in self.amps.into_iter().enumerate() {
                let amp: Amplifier = amp;
                let join = std::thread::spawn(move || {
                    let log = slog_scope::logger();
                    debug!(log, "Amplifier {} start", idx);
                    if let Err(e) = amp.run() {
                        error!(log, "Amplifier {} errored: {}", idx, e)
                    } else {
                        debug!(log, "Amplifier {} finished", idx);
                    }
                });
                threads.push(join);
            }
            for j in threads {
                j.join().unwrap();
            }
            Ok(self.out.into_iter().collect())
        }
    }

    fn create_sys_io(init: Int) -> (impl Input, impl Output, std::sync::mpsc::Receiver<Int>) {
        let (feedback_input, feedback_output) = create_stream_io();
        let (system_output, system_tap) = create_stream_tap();
        let sys_in = MultiIO::new(VecIO::input(vec![init]), feedback_input);
        let sys_out = MultiIO::new(feedback_output, system_output);
        (sys_in, sys_out, system_tap)
    }

    #[cfg(test)]
    mod test {
        use super::*;

        fn test_amp_circut(intcode: &[i64], phases: &[i64], signal: i64) {
            let c = AmplifierCircut::new(intcode, 0, phases);
            let out = c.run().unwrap()[0];
            assert_eq!(out, signal);
        }

        #[test]
        fn example_amp_circuts() {
            test_amp_circut(
                &[
                    3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
                ],
                &[4, 3, 2, 1, 0],
                43210,
            );
            test_amp_circut(
                &[
                    3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23,
                    23, 4, 23, 99, 0, 0,
                ],
                &[0, 1, 2, 3, 4],
                54321,
            );
            test_amp_circut(
                &[
                    3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7,
                    33, 1, 33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
                ],
                &[1, 0, 4, 3, 2],
                65210,
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day7part1() {
        assert_eq!(day7_part1(DAY7_INPUT).unwrap().as_str(), "11828")
    }

    #[test]
    fn day7part2() {
        assert_eq!(day7_part2(DAY7_INPUT).unwrap().as_str(), "1714298")
    }
}
