use crate::intcode::intcode_io::{FusedIO, Input, Output, VecIO};
use crate::intcode::intcode_mem::Memory;
use crate::intcode::opcodes::{parse_instruction, Instruction, ParameterMode, ParameterModes};
use anyhow::Result;

pub type Int = i64;

mod opcodes;

pub(crate) mod intcode_io;

pub fn run_intcode(intcode: Vec<Int>, input: Vec<Int>) -> Result<(Vec<Int>, Vec<Int>)> {
    let mut ic = IntCode::new(intcode, VecIO::input(input), VecIO::default());
    ic.run_till_end()?;
    let (mem, FusedIO { output, .. }) = ic.emit();
    Ok((mem, output.into_vec()))
}

mod intcode_mem {
    use super::Int;
    use std::ops::{Index, IndexMut};
    #[derive(Debug, Clone)]
    pub struct Memory {
        inner: Vec<Int>,
    }

    impl Memory {
        #[inline]
        #[allow(dead_code)]
        fn len(&self) -> usize {
            self.inner.len()
        }

        pub fn into_inner(self) -> Vec<Int> {
            self.inner
        }
    }

    impl From<Vec<Int>> for Memory {
        fn from(v: Vec<Int>) -> Self {
            Memory { inner: v }
        }
    }

    impl<T: AsRef<[Int]>> From<&T> for Memory {
        fn from(v: &T) -> Self {
            Memory {
                inner: v.as_ref().to_vec(),
            }
        }
    }

    impl Index<usize> for Memory {
        type Output = Int;
        fn index(&self, index: usize) -> &Self::Output {
            if index >= self.inner.len() {
                // log::warn!("trying to read beyond mem: {}/{}", index, self.inner.len());
                &0
            } else {
                &self.inner[index]
            }
        }
    }

    impl IndexMut<usize> for Memory {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            if index >= self.inner.len() {
                self.inner.resize(index + 1, 0);
            }
            &mut self.inner[index]
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        #[test]
        fn read_without_extend() {
            let m = Memory::from(&[]);
            assert_eq!(m[0], 0);
            assert_eq!(m[10], 0);
            assert_eq!(m[100], 0);
            assert_eq!(m[1000], 0);
            assert_eq!(m[10000000], 0);
            assert_eq!(m.len(), 0)
        }

        #[test]
        fn read_mut() {
            let mut m = Memory::from(&[]);
            m[2] = 10;
            assert_eq!(m[1], 0);
            assert_eq!(m[2], 10);
            assert_eq!(m[5], 0);
            assert_eq!(m.len(), 3);
            {
                let dst = &mut m[5];
                *dst = 50;
            }
            assert_eq!(m[1], 0);
            assert_eq!(m[2], 10);
            assert_eq!(m[5], 50);
            assert_eq!(m.len(), 6);
        }
    }
}

#[derive(Clone)]
pub struct IntCode<IO> {
    inner: Memory,
    pc: usize,
    relative_base: Int,
    pub halt: bool,
    io_device: IO,
}

impl<IO> std::fmt::Debug for IntCode<IO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntCode")
            // .field("inner", &self.inner)
            .field("pc", &self.pc)
            .field("relative_base", &self.relative_base)
            .field("halt", &self.halt)
            .finish()
    }
}

impl<I: Input, O: Output> IntCode<FusedIO<I, O>> {
    pub fn new(intcode: Vec<Int>, input: I, output: O) -> IntCode<FusedIO<I, O>> {
        IntCode {
            inner: Memory::from(intcode),
            pc: 0,
            relative_base: 0,
            halt: false,
            io_device: FusedIO { input, output },
        }
    }
}

impl<IO: Input + Output> IntCode<IO> {
    pub fn new_from_device(intcode: Vec<Int>, io_device: IO) -> IntCode<IO> {
        IntCode {
            inner: Memory::from(intcode),
            pc: 0,
            relative_base: 0,
            halt: false,
            io_device,
        }
    }

    fn get_arg(&mut self, nth: usize, modes: ParameterModes) -> &mut Int {
        let idx = self.pc + 1 + nth;
        match modes.inner[nth] {
            ParameterMode::Position => {
                let pos = self.inner[idx] as usize;
                &mut self.inner[pos]
            }
            ParameterMode::Immediate => &mut self.inner[idx],
            ParameterMode::Relative => {
                let pos = self.relative_base + self.inner[idx];
                &mut self.inner[pos as usize]
            }
        }
    }

    pub fn run_one(&mut self) -> Result<()> {
        let (instr, modes) = parse_instruction(self.inner[self.pc])?;
        let mut update_pc = true;
        // log::trace!("{:?} {:?} {:?}", instr, modes, self);
        match instr {
            Instruction::Add => {
                let lhs = *self.get_arg(0, modes);
                let rhs = *self.get_arg(1, modes);
                let dst = self.get_arg(2, modes);
                *dst = lhs + rhs;
            }
            Instruction::Mul => {
                let lhs = *self.get_arg(0, modes);
                let rhs = *self.get_arg(1, modes);
                let dst = self.get_arg(2, modes);
                *dst = lhs * rhs;
            }
            Instruction::Input => {
                let input = self.io_device.input()?;
                let dst = self.get_arg(0, modes);
                *dst = input;
            }
            Instruction::Output => {
                let src = *self.get_arg(0, modes);
                self.io_device.output(src)?;
            }
            Instruction::Halt => {
                self.halt = true;
            }
            Instruction::JumpTrue => {
                let cond = *self.get_arg(0, modes);
                if cond != 0 {
                    self.pc = *self.get_arg(1, modes) as usize;
                    update_pc = false;
                }
            }
            Instruction::JumpFalse => {
                let cond = *self.get_arg(0, modes);
                if cond == 0 {
                    self.pc = *self.get_arg(1, modes) as usize;
                    update_pc = false;
                }
            }
            Instruction::LessThan => {
                let lhs = *self.get_arg(0, modes);
                let rhs = *self.get_arg(1, modes);
                let dst = self.get_arg(2, modes);
                *dst = if lhs < rhs { 1 } else { 0 };
            }
            Instruction::EqualTo => {
                let lhs = *self.get_arg(0, modes);
                let rhs = *self.get_arg(1, modes);
                let dst = self.get_arg(2, modes);
                *dst = if lhs == rhs { 1 } else { 0 };
            }
            Instruction::SetBase => {
                let offset = *self.get_arg(0, modes);
                self.relative_base += offset;
                debug_assert!(self.relative_base >= 0);
            }
        }
        if update_pc {
            self.pc += 1 + instr.arity();
        }
        Ok(())
    }

    pub fn run_till_end(&mut self) -> Result<()> {
        while !self.halt {
            self.run_one()?
        }
        Ok(())
    }

    pub fn emit(self) -> (Vec<Int>, IO) {
        (self.inner.into_inner(), self.io_device)
    }
}

#[cfg(test)]
mod test {
    use super::{Int, IntCode};
    use crate::intcode::intcode_io::{NullIO, VecIO};
    use crate::intcode::run_intcode;
    use anyhow::Result;

    fn single_input_single_output(code: Vec<Int>, input: Int) -> Result<Int> {
        let (_, output) = run_intcode(code, vec![input])?;
        Ok(output[0])
    }

    fn assert_intcode(before: Vec<Int>, after: Vec<Int>) {
        let mut ic = IntCode::new(before, NullIO, NullIO);
        ic.run_till_end().unwrap();
        assert_eq!(ic.inner.into_inner(), after);
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
            assert_eq!(ic.io_device.output.into_vec()[0], i);
        }
    }

    #[test]
    fn modes_and_negatives() {
        assert_intcode(vec![1101, 100, -1, 4, 0], vec![1101, 100, -1, 4, 99])
    }

    #[test]
    fn is_equal_8_position_mode() {
        let code = vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_eq!(single_input_single_output(code.clone(), -7).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), -8).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), -9).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 0).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 7).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 8).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 9).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 13).unwrap(), 0);
    }

    #[test]
    fn is_lt_8_position_mode() {
        let code = vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_eq!(single_input_single_output(code.clone(), -7).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), -8).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), -9).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 0).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 7).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 8).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 9).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 13).unwrap(), 0);
    }

    #[test]
    fn is_equal_8_immediate_mode() {
        let code = vec![3, 3, 1108, -1, 8, 3, 4, 3, 99];
        assert_eq!(single_input_single_output(code.clone(), -7).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), -8).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), -9).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 0).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 7).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 8).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 9).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 13).unwrap(), 0);
    }

    #[test]
    fn is_lt_8_immediate_mode() {
        let code = vec![3, 3, 1107, -1, 8, 3, 4, 3, 99];
        assert_eq!(single_input_single_output(code.clone(), -7).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), -8).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), -9).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 0).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 7).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 8).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 9).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 13).unwrap(), 0);
    }

    #[test]
    fn jump_test_postition() {
        let code = vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        assert_eq!(single_input_single_output(code.clone(), -7).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), -8).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), -9).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 0).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 7).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 8).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 9).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 13).unwrap(), 1);
    }

    #[test]
    fn jump_test_immediate() {
        let code = vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        assert_eq!(single_input_single_output(code.clone(), -7).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), -8).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), -9).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 0).unwrap(), 0);
        assert_eq!(single_input_single_output(code.clone(), 7).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 8).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 9).unwrap(), 1);
        assert_eq!(single_input_single_output(code.clone(), 13).unwrap(), 1);
    }

    #[test]
    fn less_equal_greater_to_8() {
        let code = vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];
        assert_eq!(single_input_single_output(code.clone(), -7).unwrap(), 999);
        assert_eq!(single_input_single_output(code.clone(), -8).unwrap(), 999);
        assert_eq!(single_input_single_output(code.clone(), -9).unwrap(), 999);
        assert_eq!(single_input_single_output(code.clone(), 0).unwrap(), 999);
        assert_eq!(single_input_single_output(code.clone(), 7).unwrap(), 999);
        assert_eq!(single_input_single_output(code.clone(), 8).unwrap(), 1000);
        assert_eq!(single_input_single_output(code.clone(), 9).unwrap(), 1001);
        assert_eq!(single_input_single_output(code.clone(), 13).unwrap(), 1001);
    }

    #[test]
    fn simple_relative_base() {
        let mut code = vec![109, 2000, 109, 19, 204, -34, 99];
        code.extend(vec![0; 2000]);
        code[1985] = 923;
        assert_eq!(single_input_single_output(code, 0).unwrap(), 923);
    }

    #[test]
    fn advent_complete_intcode_copy_self() {
        let code = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];
        let (_, output) = run_intcode(code.clone(), vec![]).unwrap();
        assert_eq!(output, code);
    }

    #[test]
    fn advent_complete_intcode_16digits() {
        let code = vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0];
        let (_, output) = run_intcode(code.clone(), vec![]).unwrap();
        assert_eq!(output[0], 1219070632396864);
    }
    #[test]
    fn advent_complete_intcode_magic_number() {
        let magic = 1125899906842624;
        let code = vec![104, magic, 99];
        let (_, output) = run_intcode(code.clone(), vec![]).unwrap();
        assert_eq!(output[0], magic);
    }
}
