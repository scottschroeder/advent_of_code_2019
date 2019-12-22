type Int = u64;

pub fn run_intcode(intcode: Vec<Int>) -> Vec<Int> {
    let mut ic = IntCode::new(intcode);
    ic.run_till_end();
    ic.emit()
}

#[derive(Debug, Clone)]
pub struct IntCode {
    inner: Vec<Int>,
    pc: usize,
}

#[derive(Debug, Clone, Copy)]
enum BinOp {
    Add,
    Mul,
}

impl BinOp {
    #[inline]
    fn run(self, lhs: Int, rhs: Int) -> Int {
        match self {
            BinOp::Add => lhs + rhs,
            BinOp::Mul => lhs * rhs,
        }
    }
}

impl IntCode {
    pub fn new(intcode: Vec<Int>) -> IntCode {
        IntCode {
            inner: intcode,
            pc: 0,
        }
    }

    #[inline]
    fn get_pc_offset(&mut self, offset: usize) -> &mut Int {
        let pos = self.inner[self.pc + offset] as usize;
        &mut self.inner[pos]
    }

    pub fn run_one(&mut self) -> bool {
        let opcode = self.inner[self.pc];
        let op = match opcode {
            1 => BinOp::Add,
            2 => BinOp::Mul,
            99 => return false,
            _ => panic!("invalid intcode"),
        };
        let lhs = self.get_pc_offset(1).clone();
        let rhs = self.get_pc_offset(2).clone();
        let dst = self.get_pc_offset(3);
        *dst = op.run(lhs, rhs);
        self.pc += 4;
        true
    }

    pub fn run_till_end(&mut self) {
        while self.run_one() {}
    }

    pub fn emit(self) -> Vec<Int> {
        self.inner
    }
}

#[cfg(test)]
mod test {
    use super::{Int, IntCode};

    fn assert_intcode(before: Vec<Int>, after: Vec<Int>) {
        let mut ic = IntCode::new(before);
        ic.run_till_end();
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
}
