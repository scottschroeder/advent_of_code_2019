#[derive(Debug, Clone, Copy)]
pub(crate) enum ShuffleMethod {
    Stack,
    Cut(i64),
    Increment(usize),
}

impl ShuffleMethod {
    pub fn index(self, idx: usize, size: usize) -> usize {
        let desired = [0, 7, 4, 1, 8, 5, 2, 9, 6, 3];
        match self {
            ShuffleMethod::Stack => size - idx - 1,
            ShuffleMethod::Cut(c) => {
                let bidx = (size + idx) as i64;
                let cidx = (bidx + c) as usize;
                cidx % size
            }
            ShuffleMethod::Increment(inc) => {
                for round in 0.. {
                    let super_index = round * size + idx;
                    if super_index % inc == 0 {
                        log::trace!(
                            "idx:{} div:{} rem:{} round:{}",
                            idx,
                            idx / inc,
                            idx % inc,
                            round
                        );
                        return super_index / inc;
                    }
                }
                0
            }
        }
    }
}

fn inverse_mod_wiki2(a: usize, n: usize) -> Option<usize> {
    let mut t = 0;
    let mut newt = 1;
    let mut r = n as i64;
    let mut newr = a as i64;

    while newr != 0 {
        let quotient = r / newr;

        r = newr;
        t = newt;

        newr = r - quotient * newr;
        newt = t - quotient * newt;
    }

    if r > 1 {
        return None
    }
    if t < 0 {
        t += n as i64;
    }
    Some(t as usize)
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

fn inverse_mod_wiki(a: usize, n: usize) -> Option<usize> {
    let mut x = 0;
    let mut lastx = 1;
    let mut lastremainder = n as i64;
    let mut remainder = a as i64;

    while remainder != 0 {
        let quotient = lastremainder / remainder;

        lastremainder = remainder;
        x = lastx;

        remainder = lastremainder - quotient * remainder;
        // lastx = x - quotient * lastx;
        lastx = x - quotient * lastx;
    }

    if lastremainder > 1 {
        return None
    }
    if x < 0 {
        x += n as i64;
    }
    Some(x as usize)
}

fn inc_loop(idx: usize, inc: usize, size: usize) -> anyhow::Result<usize> {
    for round in 0..inc {
        let super_index = round * size + idx;
        if super_index % inc == 0 {
            // log::trace!(
            //     "loop: idx:{} div:{} rem:{} round:{}",
            //     idx,
            //     idx / inc,
            //     idx % inc,
            //     round
            // );
            return Ok(super_index / inc);
        }
    }
    Err(anyhow::anyhow!("invalid"))
}

fn inc_exact_v1(pos: usize, inc: usize, size: usize) -> anyhow::Result<usize> {
    // args = 1, 4, 5
    let x = size + (pos / inc); // 5
    let rem = ((pos % inc) * inc) % size; // 1
    let x2 = x - rem; // 4
    Ok(x2 % size) // 2
}

/*

 3/5 = 2
 4/5 = 4
 3/10 = 1
*/

fn inc_exact_v2(pos: usize, inc: usize, size: usize) -> anyhow::Result<usize> {
    let offset = dbg!(size % inc);
    let backset = size - inc;
    let x = backset * pos;
    Ok((size - (x % size)) % size)
}

fn inc_exact_8_9(idx: usize, inc: usize, size: usize) -> anyhow::Result<usize> {
    let offset = size % inc;
    let backoffset = inc - offset;
    if inc % backoffset == 0 {
        return Err(anyhow::anyhow!("invalid"));
    }
    let rem = idx % inc;
    let round = rem * backoffset;
    let round_fit = round % inc;
    let super_index = round_fit * size + idx;
    let r = (super_index) % size;
    log::trace!(
        "exact: idx:{} rem:{} offset:{} round:{}, super:{}, r:{}",
        idx,
        rem,
        offset,
        round,
        super_index,
        r
    );
    Ok(r)
}
fn inc_exact(idx: usize, inc: usize, deck: usize) -> anyhow::Result<usize> {
    let inv = inverse_mod(inc, deck).ok_or_else(|| anyhow::anyhow!("invalid"))?;
    log::trace!("inv({}, {}) -> {}", inc, deck, inv);
    //Ok(((idx * inc * inv) % (inc * deck)) / inc)
    Ok((idx * inv) % deck)
}

fn inc_exact_8_10(idx: usize, inc: usize, deck: usize) -> anyhow::Result<usize> {
    return inc_exact_8_9(idx, inc, deck);
    let offset = deck % inc;
    let backoffset = inc - offset;
    if inc % backoffset == 0 {
        return Err(anyhow::anyhow!("invalid"));
    }
    let i = idx;
    let offmod = ((offset - 1 + i) % offset) + 1;

    let block = (i + inc - 1) / inc;
    let offset_block = (offset - 1 + block) / offset;
    let skip = offmod;
    let rank = offset_block * skip * inc;
    let round = (rank - i)/ offset;
    let super_index = round * deck + i;
    let r = (super_index) / inc;
    // let rem = idx % inc;
    // let round = rem * backoffset;
    // let round_fit = round % inc;
    // let super_index = round_fit * deck + idx;
    // let r = (super_index) % deck;
    log::trace!(
        "exact: idx:{} offset:{} round:{}, super:{}, r:{}",
        idx,
        offset,
        round,
        super_index,
        r
    );
    Ok(r)
}

fn test_inc_shuf_all(deck: usize, inc: usize) {
    log::info!("TEST deck:{} inc:{}", deck, inc);
    for idx in 0..deck {
        let r_exact = match inc_exact(idx, inc, deck) {
            Ok(r) => r,
            Err(_) => {
                log::debug!("SKIP inc:{} deck:{}", inc, deck);
                return;
            }
        };
        match inc_loop(idx, inc, deck) {
            Ok(r_loop) => {
                //let r_exact = inc_exact(idx, inc, deck);
                if r_loop != r_exact {
                    // log::warn!(
                    //     "FAIL idx:{} inc:{} deck:{} loop:{} exact:{}",
                    //     idx,
                    //     inc,
                    //     deck,
                    //     r_loop,
                    //     r_exact
                    // );
                    // return;
                    log::error!("FAIL idx:{} correct:{} actual:{}", idx, r_loop, r_exact);
                } else {
                    log::trace!("PASS idx:{} correct:{} actual:{}", idx, r_loop, r_exact);
                }
            }
            Err(_) => {
                log::trace!("SKIP inc:{} deck:{}", inc, deck);
                return;
            }
        }
    }
    return;
    let offset = deck % inc;
    let backoffset = inc - offset;
    let backoffsize = deck - offset;
    let down_slope = deck / offset;
    log::debug!("o:{} b:{} s:{} down:{}", offset, backoffset, backoffsize, down_slope);
    let mut vindex = vec![];
    let mut vrem = vec![];
    let mut vremback = vec![];
    let mut voffmod = vec![];
    for i in 0..deck {
        vindex.push(i);
        let rem = i % inc;
        vrem.push(rem);
        let remback = (inc - rem) % inc;
        vremback.push(remback);
        // let offmod = ((offset - 1 + i) % offset) + 1;
        let offmod = i % offset;
        // let offmod = remback / offset;
        // let offmod = 1;
        voffmod.push(offmod);
    }
    for v in &[&vindex, &vremback, &voffmod] {
        for i in *v {
            print!("{:>2}", i);
        }
        println!("");
    }

    let mut v = vec![];
    let mut c = 0;
    for i in 0..(deck * inc) {
        if i % deck == 0 && i > 0 {
            println!();
        }
        if i % inc == 0 {
            print!("{:>2}", c);
            c += 1;
            v.push(i);
        } else {
            print!("  ");
        }
    }
    println!();
    for i in 0..deck {
        let block = (i + inc - 1) / inc;
        let offset_block = (offset - 1 + block) / offset;
        let skip = voffmod[i];
        let rank = (offset_block * skip * inc);
        // let round = (rank - i)/ offset;
        // let super_index = round * deck + i;
        // let r = (super_index) / inc;
        // println!(
        //     "{}: r:{} or:{} s:{} sn:{} round:{} super_idx:{} idx:{}",
        //     i, block, offset_block, skip, rank, round, super_index, r
        // );
        println!(
            "{}: block:{} offset_block:{} skip:{} rank:{}",
            i, block, offset_block, skip, rank
        );
    }
}

pub fn test_inc(max: usize) {
    let tc = &[
        (6, 4),
        (10, 3),
        (5, 3),
        (5, 4),
        (7, 5),
        (17, 8),
        (21, 5),
        (21, 10),
        (21, 17),
        (8, 5),
    ];
    // for (deck, inc) in tc {
    //     test_inc_shuf_all(*deck, *inc);
    // }
    // return;

    for deck in 0..max {
        for inc in 1..deck {
            test_inc_shuf_all(deck, inc)
        }
    }
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
    deck: Deck,
    scratch: Deck,
}

impl Shuffle {
    pub(crate) fn new(deck: Deck) -> Shuffle {
        Shuffle {
            scratch: deck.clone(),
            deck,
        }
    }
    pub(crate) fn finalize(self) -> Deck {
        self.deck
    }

    pub(crate) fn do_sequence(&mut self, methods: Vec<ShuffleMethod>) {
        for m in methods {
            self.do_method(m)
        }
    }

    fn do_method(&mut self, method: ShuffleMethod) {
        match method {
            ShuffleMethod::Stack => {}
            //ShuffleMethod::Cut(_) => {}
            //ShuffleMethod::ReverseCut(_) => {}
            //ShuffleMethod::Increment(_) => {}
            _ => todo!("method {:?}", method),
        }
    }
}
