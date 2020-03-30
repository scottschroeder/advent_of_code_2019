use anyhow::{Result, Error, anyhow as ah};
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::{Ordering, Ord};
use std::ops::{Add, Sub, AddAssign, SubAssign};

lazy_static! {
    static ref RE_D3: Regex = Regex::new(r##"<x= *(?P<x>-?\d+), y= *(?P<y>-?\d+), z= *(?P<z>-?\d+)>"##).unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct D3 {
    pub(crate) x: i64,
    pub(crate) y: i64,
    pub(crate) z: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum D3Dimm {
    X,
    Y,
    Z,
}

#[inline]
fn delta(a: i64, b: i64) -> i64 {
    match a.cmp(&b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

impl Add for D3 {
    type Output = D3;

    fn add(self, rhs: Self) -> Self::Output {
        D3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for D3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}
impl SubAssign for D3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Sub for D3 {
    type Output = D3;

    fn sub(self, rhs: Self) -> Self::Output {
        D3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Default for D3 {
    fn default() -> Self {
        D3 {
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl D3 {
    pub(crate) fn compare(self, other: &D3) -> D3 {
        D3 {
            x: delta(self.x, other.x),
            y: delta(self.y, other.y),
            z: delta(self.z, other.z),
        }
    }
    pub(crate) fn abs(self) -> D3 {
        D3 {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }
    pub(crate) fn total(self) -> i64 {
        self.x + self.y + self.z
    }
    pub(crate) fn get(self, d: D3Dimm) -> i64 {
        match d {
            D3Dimm::X => self.x,
            D3Dimm::Y => self.y,
            D3Dimm::Z => self.z,
        }
    }
}

fn cap_to_d3(cap: &regex::Captures) -> Result<D3> {
    let d3 = D3 {
        x: cap["x"].parse::<i64>()?,
        y: cap["y"].parse::<i64>()?,
        z: cap["z"].parse::<i64>()?,
    };
    Ok(d3)
}

pub(crate) fn parse(s: &str) -> Result<Vec<D3>> {
    let mut data = vec![];
    for caps in RE_D3.captures_iter(s) {
        let d3 = cap_to_d3(&caps)
            .map_err(|e| ah!("{}: {:?}", e, caps))?;
        data.push(d3);
    }
    Ok(data)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    const D12_EX1_STEP1: &str = "pos=<x= 2, y=-1, z= 1>, vel=<x= 3, y=-1, z=-1>
        pos=<x= 3, y=-7, z=-4>, vel=<x= 1, y= 3, z= 3>
        pos=<x= 1, y=-7, z= 5>, vel=<x=-3, y= 1, z=-3>
        pos=<x= 2, y= 2, z= 0>, vel=<x=-1, y=-3, z= 1>";

    #[test]
    fn parse_d3() {
        let expected = vec![
            D3 { x: -1, y: 0, z: 2 },
            D3 { x: 2, y: -10, z: -7 },
            D3 { x: 4, y: -8, z: 8 },
            D3 { x: 3, y: 5, z: -1 },
        ];
        let actual = parse(DAY12_EX1).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_d3_pos_vel_pairs() {
        let expected = vec![
            D3 { x: 2, y: -1, z: 1 },
            D3 { x: 3, y: -1, z: -1 },
            D3 { x: 3, y: -7, z: -4 },
            D3 { x: 1, y: 3, z: 3 },
            D3 { x: 1, y: -7, z: 5 },
            D3 { x: -3, y: 1, z: -3 },
            D3 { x: 2, y: 2, z: 0 },
            D3 { x: -1, y: -3, z: 1 }
        ];
        let actual = parse(D12_EX1_STEP1).unwrap();
        assert_eq!(actual, expected);
    }
}
