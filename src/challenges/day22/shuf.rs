use num::ToPrimitive;
#[derive(Debug, Clone, Copy)]
pub(crate) enum ShuffleMethod {
    Stack,
    Cut(i64),
    Increment(usize),
}

#[derive(Debug, Clone, Copy)]
enum ShuffleActor {
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
        let mut acc = Self::initilize(deck);
        for m in methods.iter().rev() {
            let sa = ShuffleActor::from_method(m, deck)?;
            acc = acc.add_step(sa);
        }
        Ok(acc)
    }

    fn initilize(size: usize) -> Self {
        Shuffle {
            factor: 1,
            offset: 0,
            size: size as i64,
        }
    }
    pub(crate) fn repeat(self, n: u64) -> Shuffle {
        let size = self.size;
        let fr = modpow(self.factor, n, size);
        let fr1 = modpow(self.factor, n+1, size);
        Shuffle {
            factor: fr,
            offset: (fr1 -1) / (self.factor - 1),
            size,
        }
    }
    fn add_step(self, s: ShuffleActor) -> Shuffle {
        let size = self.size;
        let mut new = match s {
            ShuffleActor::Stack => Shuffle {
                factor: self.factor * -1,
                offset: (self.offset - size + 1) * -1,
                size,
            },
            ShuffleActor::Cut(c) => Shuffle {
                factor: self.factor,
                offset: self.offset + c,
                size,
            },
            ShuffleActor::Increment(i) => Shuffle {
                factor: mod_mul(self.factor, i as u64, size),
                offset: mod_mul(self.offset ,i as u64, size),
                size,
            },
        };
        new.factor = new.factor % size;
        new.offset = (new.offset + size) % size;
        new
    }
    pub(crate) fn index(&self, idx: usize) -> usize {
        let p = mod_mul(self.factor, idx as u64, self.size);
        ((p + self.offset + self.size) % self.size) as usize
    }
    pub(crate) fn full(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.size as usize).map(move |idx| self.index(idx))
    }
}

fn mod_mul(a: i64, b: u64, n: i64) -> i64 {
    // return (a * b) % n;
    let neg = if a < 0 {
        -1
    } else {
        1
    };
    let mut a = (a * neg) % n;
    let mut b = b;
    let mut res = 0; // Initialize result
    while b > 0 {
        // If b is odd, add 'a' to result
        if b % 2 == 1 {
            res = (res + a) % n;
        }

        // Multiply 'a' with 2
        a = (a * 2) % n;

        // Divide b by 2
        b /= 2;
    }

    // Return result
    (res * neg) % n
}

fn modpow(base: i64, exp: u64, modulus: i64) -> i64{
    let mut neg = if base < 0 {
        -1
    } else {
        1
    };
    if exp % 2 == 0 {
        neg = 1;
    }


  let mut base = (base * neg % modulus) as u64;
  let mut exp = exp;
  let mut result = 1;
  while exp > 0 {
    if exp & 1 > 0 {
        result = mod_mul(result,  base, modulus);
    }
    base = mod_mul(base as i64,  base, modulus) as u64;
    exp >>= 1;
  }
  neg * result
}

impl ShuffleActor {
    fn from_method(method: &ShuffleMethod, size: usize) -> anyhow::Result<ShuffleActor> {
        Ok(match method {
            ShuffleMethod::Stack => ShuffleActor::Stack,
            ShuffleMethod::Cut(c) => ShuffleActor::Cut(*c),
            ShuffleMethod::Increment(inc) => {
                let inv = inverse_mod(*inc, size).ok_or_else(|| {
                    anyhow::anyhow!("incremnt {} not valid for deck size {}", inc, size)
                })?;
                ShuffleActor::Increment(inv)
            }
        })
    }
    fn index(self, idx: usize, size: usize) -> usize {
        match self {
            ShuffleActor::Stack => size - idx - 1,
            ShuffleActor::Cut(c) => {
                let bidx = (size + idx) as i64;
                let cidx = (bidx + c) as usize;
                cidx % size
            }
            ShuffleActor::Increment(inv) => {
                // log::trace!("idx: {}, inv: {}, size: {}", idx, inv, size);
                let bidx = num::BigUint::from(idx);
                let binv = num::BigUint::from(inv);
                let bsize = num::BigUint::from(size);
                let bnew = (bidx * binv) % bsize;
                bnew.to_u64().unwrap() as usize
                //(idx * inv) % size
                //((idx % size) * (inv % size)) % size
            }
        }
    }
}

fn inverse_mod(a: usize, n: usize) -> Option<usize> {
    let mut mn = (n as isize, a as isize);
    let mut xy = (0, 1);

    while mn.1 != 0 {
        xy = (xy.1, xy.0 - (mn.0 / mn.1) * xy.1);
        mn = (mn.1, mn.0 % mn.1);
    }

    if mn.0 > 1 {
        return None;
    }

    while xy.0 < 0 {
        xy.0 += n as isize;
    }
    Some(xy.0 as usize)
}

#[derive(Debug, Clone, Copy)]
pub struct Card(pub u32);

#[derive(Debug, Clone)]
pub struct Deck(pub Vec<Card>);

impl Deck {
    pub(crate) fn new(size: u32) -> Deck {
        Deck((0..size).map(Card).collect())
    }
}

pub(crate) struct Shuffle2 {
    methods: Vec<ShuffleActor>,
    deck: usize,
}

impl Shuffle2 {
    pub(crate) fn new(deck: usize, methods: &[ShuffleMethod]) -> anyhow::Result<Shuffle2> {
        Ok(Shuffle2 {
            methods: methods
                .iter()
                .rev()
                .map(|m| ShuffleActor::from_method(m, deck))
                .collect::<anyhow::Result<Vec<ShuffleActor>>>()?,
            deck,
        })
    }
    pub(crate) fn index(&self, idx: usize) -> usize {
        self.methods
            .iter()
            .fold(idx, |pos, actor| actor.index(pos, self.deck))
    }
    pub(crate) fn full(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.deck).map(move |idx| self.index(idx))
    }
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
}
