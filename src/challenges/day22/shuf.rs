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

impl ShuffleActor {
    fn from_method(method: &ShuffleMethod, size: usize) -> anyhow::Result<ShuffleActor> {
        Ok(match method {
            ShuffleMethod::Stack => ShuffleActor::Stack,
            ShuffleMethod::Cut(c) => ShuffleActor::Cut(*c),
            ShuffleMethod::Increment(inc) => {
                let inv = inverse_mod(*inc, size)
                    .ok_or_else(|| anyhow::anyhow!("incremnt {} not valid for deck size {}", inc, size))?;
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
                (idx * inv) % size
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

pub(crate) struct Shuffle {
    methods: Vec<ShuffleActor>,
    deck: usize,
}

impl Shuffle {
    pub(crate) fn new(deck: usize, methods: &[ShuffleMethod]) -> anyhow::Result<Shuffle> {
        Ok(Shuffle {
            methods: methods
            .iter()
            .rev()
            .map(|m| ShuffleActor::from_method(m, deck))
            .collect::<anyhow::Result<Vec<ShuffleActor>>>()?,
            deck,
        })
    }
    pub(crate) fn index(&self, idx: usize) -> usize {
        self.methods.iter().fold(idx, |pos, actor|{
            actor.index(pos, self.deck)
        })
    }
    pub(crate) fn full(&self) -> impl Iterator<Item =usize> + '_ {
        (0..self.deck)
        .map(move |idx| self.index(idx))
    }
}
