#[derive(Debug, Clone, Copy)]
pub(crate) enum ShuffleMethod {
    Stack,
    Cut(i64),
    Increment(usize),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Shuffle {
    factor: i64,
    offset: i64,
    size: i64,
}

impl Shuffle {
    pub(crate) fn new(deck: usize, methods: &[ShuffleMethod]) -> anyhow::Result<Shuffle> {
        let mut acc = Self::initilize(deck as i64);
        for m in methods.iter().rev() {
            acc = acc.add_step(*m)?;
        }
        Ok(acc)
    }

    fn initilize(size: i64) -> Self {
        Shuffle {
            factor: 1,
            offset: 0,
            size,
        }
    }

    fn add_shuffle(self, other: Shuffle) -> Shuffle {
        Shuffle {
            factor: mod_mul(self.factor, other.factor, self.size),
            offset: (mod_mul(self.offset, other.factor, self.size) + other.offset) % self.size,
            size: self.size,
        }
    }

    pub(crate) fn repeat(self, n: u64) -> Shuffle {
        let mut base = self;
        let mut exp = n;
        let mut result = Shuffle::initilize(self.size);
        while exp > 0 {
            if exp & 1 > 0 {
                result = result.add_shuffle(base);
            }
            base = base.add_shuffle(base);
            exp >>= 1;
        }
        result
    }
    fn add_step(self, s: ShuffleMethod) -> anyhow::Result<Shuffle> {
        let size = self.size;
        let mut new = match s {
            ShuffleMethod::Stack => Shuffle {
                factor: self.factor * -1,
                offset: (self.offset - size + 1) * -1,
                size,
            },
            ShuffleMethod::Cut(c) => Shuffle {
                factor: self.factor,
                offset: self.offset + c,
                size,
            },
            ShuffleMethod::Increment(inc) => {
                let inv = inverse_mod(inc as i64, size).ok_or_else(|| {
                    anyhow::anyhow!("incremnt {} not valid for deck size {}", inc, size)
                })?;
                Shuffle {
                    factor: mod_mul(self.factor, inv, size),
                    offset: mod_mul(self.offset, inv, size),
                    size,
                }
            }
        };
        new.factor = new.factor % size;
        new.offset = (new.offset + size) % size;
        Ok(new)
    }

    pub(crate) fn index(&self, idx: usize) -> usize {
        let p = mod_mul(self.factor, idx as i64, self.size);
        ((p + self.offset + 2 * self.size) % self.size) as usize
    }
    pub(crate) fn full(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.size as usize).map(move |idx| self.index(idx))
    }
}

fn mod_mul(a: i64, b: i64, n: i64) -> i64 {
    let mut res = 0;
    let mut a = a % n;
    let neg = if b < 0 { -1 } else { 1 };
    let mut b = b * neg;
    while b != 0 {
        if b & 1 == 1 {
            res = (res + a) % n;
        }
        a = (a * 2) % n;
        b >>= 1;
    }
    res * neg % n
}

fn inverse_mod(a: i64, n: i64) -> Option<i64> {
    let mut mn = (n, a);
    let mut xy = (0, 1);

    while mn.1 != 0 {
        xy = (xy.1, xy.0 - (mn.0 / mn.1) * xy.1);
        mn = (mn.1, mn.0 % mn.1);
    }

    if mn.0 > 1 {
        return None;
    }

    while xy.0 < 0 {
        xy.0 += n;
    }
    Some(xy.0)
}

#[cfg(test)]
mod tests {
    use super::super::parse;
    use super::Shuffle as TS;
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn inverse() {
        let procedure = vec![ShuffleMethod::Stack];
        let s = TS::new(10, procedure.as_slice()).unwrap();
        let actual = s.full().collect::<Vec<_>>();
        assert_eq!(actual, vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn cut_forward() {
        let procedure = vec![ShuffleMethod::Cut(3)];
        let s = TS::new(10, procedure.as_slice()).unwrap();
        let actual = s.full().collect::<Vec<_>>();
        assert_eq!(actual, vec![3, 4, 5, 6, 7, 8, 9, 0, 1, 2]);
    }
    #[test]
    fn cut_backward() {
        let procedure = vec![ShuffleMethod::Cut(-4)];
        let s = TS::new(10, procedure.as_slice()).unwrap();
        let actual = s.full().collect::<Vec<_>>();
        assert_eq!(actual, vec![6, 7, 8, 9, 0, 1, 2, 3, 4, 5,]);
    }

    fn do_increment(inc: usize, size: usize) -> Vec<usize> {
        let procedure = vec![ShuffleMethod::Increment(inc)];
        let s = TS::new(size, procedure.as_slice()).unwrap();
        s.full().collect::<Vec<_>>()
    }

    #[test]
    fn increment() {
        assert_eq!(do_increment(3, 10), vec![0, 7, 4, 1, 8, 5, 2, 9, 6, 3]);
        assert_eq!(do_increment(3, 5), vec![0, 2, 4, 1, 3]);
        assert_eq!(do_increment(4, 5), vec![0, 4, 3, 2, 1]);
        assert_eq!(do_increment(5, 7), vec![0, 3, 6, 2, 5, 1, 4]);
    }

    #[test]
    fn ex1() {
        let procedures = parse(DAY22_EX1).unwrap();
        let s = TS::new(10, procedures.as_slice()).unwrap();
        let actual = s.full().collect::<Vec<_>>();
        assert_eq!(actual, vec![0, 3, 6, 9, 2, 5, 8, 1, 4, 7]);
    }
    #[test]
    fn ex2() {
        let procedures = parse(DAY22_EX2).unwrap();
        let s = TS::new(10, procedures.as_slice()).unwrap();
        let actual = s.full().collect::<Vec<_>>();
        assert_eq!(actual, vec![3, 0, 7, 4, 1, 8, 5, 2, 9, 6]);
    }
    #[test]
    fn ex3() {
        let procedures = parse(DAY22_EX3).unwrap();
        let s = TS::new(10, procedures.as_slice()).unwrap();
        let actual = s.full().collect::<Vec<_>>();
        assert_eq!(actual, vec![6, 3, 0, 7, 4, 1, 8, 5, 2, 9]);
    }
    #[test]
    fn ex4() {
        let procedures = parse(DAY22_EX4).unwrap();
        let s = TS::new(10, procedures.as_slice()).unwrap();
        let actual = s.full().collect::<Vec<_>>();
        assert_eq!(actual, vec![9, 2, 5, 8, 1, 4, 7, 0, 3, 6]);
    }

    fn check_mod_mul_small(a: i64, b: i64, n: i64) {
        assert_eq!(mod_mul(a, b, n), (a * b) % n);
    }
    #[test]
    fn modmul_pos() {
        check_mod_mul_small(3, 4, 12);
        check_mod_mul_small(3, 4, 10);
    }
    #[test]
    fn modmul_rneg() {
        check_mod_mul_small(-3, 4, 12);
        check_mod_mul_small(-3, 4, 10);
    }
    #[test]
    fn modmul_lneg() {
        check_mod_mul_small(3, -4, 12);
        check_mod_mul_small(3, -4, 10);
    }
    #[test]
    fn modmul_double_neg() {
        check_mod_mul_small(-3, -4, 12);
        check_mod_mul_small(-3, -4, 10);
    }
}
