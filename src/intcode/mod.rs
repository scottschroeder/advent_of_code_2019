use crate::intcode::incode_io::{Input, NullIO, Output, VecIO};
use crate::intcode::opcodes::{parse_instruction, Instruction, ParameterMode, ParameterModes};
use anyhow::{anyhow, Result};

type Int = i64;

mod opcodes;

mod incode_io {
    use super::Int;
    use anyhow::{anyhow, Result};

    pub trait Input {
        fn input(&mut self) -> Result<Int>;
    }

    pub trait Output {
        fn output(&mut self, out: Int) -> Result<()>;
    }

    pub struct NullIO;

    impl Input for NullIO {
        fn input(&mut self) -> Result<Int> {
            Ok(0)
        }
    }

    impl Output for NullIO {
        fn output(&mut self, out: Int) -> Result<()> {
            trace!(slog_scope::logger(), "NullIO output => {}", out);
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct VecIO {
        inner: Vec<Int>,
    }

    impl VecIO {
        pub fn input(mut input: Vec<Int>) -> VecIO {
            input.reverse();
            VecIO { inner: input }
        }
        pub fn into_vec(self) -> Vec<Int> {
            self.inner
        }
    }

    impl Input for VecIO {
        fn input(&mut self) -> Result<Int> {
            self.inner.pop().ok_or_else(|| anyhow!("no more input"))
        }
    }

    impl Output for VecIO {
        fn output(&mut self, out: Int) -> Result<()> {
            trace!(slog_scope::logger(), "VecIO output => {}", out);
            self.inner.push(out);
            Ok(())
        }
    }
}

pub fn run_intcode(intcode: Vec<Int>, input: Vec<Int>) -> Result<(Vec<Int>, Vec<Int>)> {
    let mut ic = IntCode::new(intcode, VecIO::input(input), VecIO::default());
    ic.run_till_end()?;
    let output = ic.output.clone();
    Ok((ic.emit(), output.into_vec()))
}

#[derive(Debug, Clone)]
pub struct IntCode<I, O> {
    inner: Vec<Int>,
    pc: usize,
    halt: bool,
    input: I,
    output: O,
}

impl<I: Input, O: Output> IntCode<I, O> {
    pub fn new(intcode: Vec<Int>, input: I, output: O) -> IntCode<I, O> {
        IntCode {
            inner: intcode,
            pc: 0,
            halt: false,
            input,
            output,
        }
    }

    fn get_arg(&mut self, nth: usize, modes: ParameterModes) -> &mut Int {
        let idx = self.pc + 1 + nth;
        match modes.inner[nth] {
            ParameterMode::Position => {
                let pos = self.inner[idx] as usize;
                &mut self.inner[pos]
            },
            ParameterMode::Immediate => &mut self.inner[idx],
        }
    }

    #[inline]
    fn get_pc_offset(&mut self, offset: usize) -> &mut Int {
        let pos = self.inner[self.pc + offset] as usize;
        &mut self.inner[pos]
    }

    pub fn run_one(&mut self) -> Result<()> {
        let (instr, modes) = parse_instruction(self.inner[self.pc])?;
        match instr {
            Instruction::Add => {
                let lhs = self.get_arg(0, modes).clone();
                let rhs = self.get_arg(1, modes).clone();
                let dst = self.get_arg(2, modes);
                *dst = lhs + rhs;
            },
            Instruction::Mul => {
                let lhs = self.get_arg(0, modes).clone();
                let rhs = self.get_arg(1, modes).clone();
                let dst = self.get_arg(2, modes);
                *dst = lhs * rhs;
            },
            Instruction::Input => {
                let input = self.input.input()?.clone();
                let dst = self.get_arg(0, modes);
                *dst = input;
            },
            Instruction::Output => {
                let src = self.get_arg(0, modes).clone();
                self.output.output(src)?;
            },
            Instruction::Halt => {
                self.halt = true;
            },
        }
        self.pc += 1 + instr.arity();
        Ok(())
    }

    pub fn run_till_end(&mut self) -> Result<()> {
        while !self.halt {
            self.run_one()?
        }
        Ok(())
    }

    pub fn emit(self) -> Vec<Int> {
        self.inner
    }
}

#[cfg(test)]
mod test {
    use super::{Int, IntCode};
    use crate::intcode::incode_io::{NullIO, VecIO};

    fn assert_intcode(before: Vec<Int>, after: Vec<Int>) {
        let mut ic = IntCode::new(before, NullIO, NullIO);
        ic.run_till_end().unwrap();
        assert_eq!(ic.inner, after);
    }

    #[test]
    fn add_instr() {
        let before = vec![1, 1, 2, 0, 99];
        let after = vec![3, 1, 2, 0, 99];
        assert_intcode(before, after);
    }

    #[test]
    fn mul_instr() {
        let before = vec![2, 0, 2, 0, 99];
        let after = vec![4, 0, 2, 0, 99];
        assert_intcode(before, after);
    }

    #[test]
    fn chain_instr() {
        let before = vec![1, 3, 4, 3, 2, 3, 9, 7, 99, 11];
        let after = vec![1, 3, 4, 5, 2, 3, 9, 55, 99, 11];
        assert_intcode(before, after);
    }

    #[test]
    fn advent_examples() {
        assert_intcode(
            vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50],
            vec![3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50],
        );
        assert_intcode(vec![1, 0, 0, 0, 99], vec![2, 0, 0, 0, 99]);
        assert_intcode(vec![2, 3, 0, 3, 99], vec![2, 3, 0, 6, 99]);
        assert_intcode(vec![2, 4, 4, 5, 99, 0], vec![2, 4, 4, 5, 99, 9801]);
        assert_intcode(
            vec![1, 1, 1, 4, 99, 5, 6, 0, 99],
            vec![30, 1, 1, 4, 2, 5, 6, 0, 99],
        );
    }

    #[test]
    fn input_to_output() {
        let code = vec![3, 0, 4, 0, 99];
        for i in 0..10 {
            let mut ic = IntCode::new(code.clone(), VecIO::input(vec![i]), VecIO::default());
            ic.run_till_end().unwrap();
            assert_eq!(ic.output.into_vec()[0], i);
        }
    }

    #[test]
    fn modes_and_negatives() {
        assert_intcode(vec![1101, 100, -1, 4, 0], vec![1101, 100, -1, 4, 99])
    }
}
