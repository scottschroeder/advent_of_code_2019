use super::Int;
use anyhow::{anyhow, Result};

const MAX_ARITY: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    Add,
    Mul,
    Input,
    Output,
    Halt,
}

impl Instruction {
    pub fn arity(self) -> usize {
        match self {
            Instruction::Add => 3,
            Instruction::Mul => 3,
            Instruction::Input => 1,
            Instruction::Output => 1,
            Instruction::Halt => 0,
        }
    }

    pub fn try_from_int(instr: Int) -> Result<Instruction> {
        Ok(match instr {
            01 => Instruction::Add,
            02 => Instruction::Mul,
            03 => Instruction::Input,
            04 => Instruction::Output,
            99 => Instruction::Halt,
            n => return Err(anyhow!("unknown opcode {:02}", n)),
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ParameterModes {
    pub inner: [ParameterMode; MAX_ARITY],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParameterMode {
    Position,
    Immediate,
}

impl ParameterMode {
    pub fn try_from_int(m: i64) -> Result<ParameterMode> {
        Ok(match m {
            0 => ParameterMode::Position,
            1 => ParameterMode::Immediate,
            n => return Err(anyhow!("unknown parameter mode: {}", n)),
        })
    }
}

impl Default for ParameterMode {
    fn default() -> Self {
        ParameterMode::Position
    }
}

pub fn parse_instruction(instr: Int) -> Result<(Instruction, ParameterModes)> {
    let opcode = Instruction::try_from_int(instr % 100)?;

    let mut modes = ParameterModes::default();
    let mut packed_modes = instr / 100;
    for idx in 0..MAX_ARITY {
        modes.inner[idx] = ParameterMode::try_from_int(packed_modes % 10)?;
        packed_modes /= 10;
    }
    Ok((opcode, modes))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_instr_add() {
        assert_eq!(
            parse_instruction(1).unwrap(),
            (
                Instruction::Add,
                ParameterModes {
                    inner: [
                        ParameterMode::Position,
                        ParameterMode::Position,
                        ParameterMode::Position,
                    ]
                }
            )
        )
    }
    #[test]
    fn parse_mul_with_mode() {
        assert_eq!(
            parse_instruction(1002).unwrap(),
            (
                Instruction::Mul,
                ParameterModes {
                    inner: [
                        ParameterMode::Position,
                        ParameterMode::Immediate,
                        ParameterMode::Position,
                    ]
                }
            )
        )
    }
    #[test]
    fn parse_max_mode() {
        assert_eq!(
            parse_instruction(10001).unwrap(),
            (
                Instruction::Add,
                ParameterModes {
                    inner: [
                        ParameterMode::Position,
                        ParameterMode::Position,
                        ParameterMode::Immediate,
                    ]
                }
            )
        )
    }
    #[test]
    fn parse_halt() {
        assert_eq!(
            parse_instruction(99).unwrap(),
            (
                Instruction::Halt,
                ParameterModes {
                    inner: [
                        ParameterMode::Position,
                        ParameterMode::Position,
                        ParameterMode::Position,
                    ]
                }
            )
        )
    }
}
