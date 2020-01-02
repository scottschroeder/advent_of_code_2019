use crate::util::parse_str;
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
enum WireDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
struct WireRun {
    direction: WireDirection,
    step: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct WirePoint {
    x: i64,
    y: i64,
}

impl WirePoint {
    fn add_run(&self, run: WireRun) -> WirePoint {
        let mut new = *self;
        match run.direction {
            WireDirection::Up => new.y += run.step as i64,
            WireDirection::Down => new.y -= run.step as i64,
            WireDirection::Left => new.x -= run.step as i64,
            WireDirection::Right => new.x += run.step as i64,
        }
        new
    }

    fn sum(self) -> i64 {
        self.x.abs() + self.y.abs()
    }

    #[inline]
    fn distance(self, other: WirePoint) -> u64 {
        assert!(self.x == other.x || self.y == other.y);
        let x = (self.x - other.x).abs();
        let y = (self.y - other.y).abs();
        (x + y) as u64
    }
}

#[derive(Debug, Clone, Copy)]
struct WireSegment {
    start: WirePoint,
    end: WirePoint,
}

#[derive(Debug, Clone, Copy)]
enum WireIntersect {
    None,
    Point(WirePoint),
    Run,
}

#[derive(Debug, Clone, Copy)]
enum WireAxis {
    Vertical(i64),
    Horizontal(i64),
}

#[inline]
fn in_between(a: i64, b: i64, x: i64) -> bool {
    x >= a.min(b) && x <= a.max(b)
}

impl WireSegment {
    #[inline]
    fn contains(self, x: i64, y: i64) -> bool {
        in_between(self.start.x, self.end.x, x) && in_between(self.start.y, self.end.y, y)
    }

    #[inline]
    fn axis(self) -> WireAxis {
        if self.start.y == self.end.y {
            WireAxis::Vertical(self.start.y)
        } else {
            WireAxis::Horizontal(self.start.x)
        }
    }

    fn intersection(self, other: WireSegment) -> WireIntersect {
        let _log = slog_scope::logger();

        match (self.axis(), other.axis()) {
            (WireAxis::Horizontal(x), WireAxis::Vertical(y))
            | (WireAxis::Vertical(y), WireAxis::Horizontal(x)) => {
                if self.contains(x, y) && other.contains(x, y) {
                    WireIntersect::Point(WirePoint { x, y })
                } else {
                    WireIntersect::None
                }
            },
            (WireAxis::Horizontal(x1), WireAxis::Horizontal(x2)) => {
                if x1 != x2 {
                    WireIntersect::None
                } else if self.contains(other.start.x, other.start.y)
                    || self.contains(other.end.x, other.end.y)
                {
                    WireIntersect::Run
                } else {
                    WireIntersect::None
                }
            },
            (WireAxis::Vertical(y1), WireAxis::Vertical(y2)) => {
                if y1 != y2 {
                    WireIntersect::None
                } else if self.contains(other.start.x, other.start.y)
                    || self.contains(other.end.x, other.end.y)
                {
                    WireIntersect::Run
                } else {
                    WireIntersect::None
                }
            },
        }
    }
}

#[derive(Debug)]
struct Wire {
    inner: Vec<WireSegment>,
}

impl Wire {
    fn distance_to_point(&self, p: WirePoint) -> Option<u64> {
        let mut distance = 0;
        for segment in &self.inner {
            if segment.contains(p.x, p.y) {
                distance += segment.start.distance(p);
                return Some(distance);
            } else {
                distance += segment.start.distance(segment.end);
            }
        }
        None
    }
}

fn wire_crossings(w1: &Wire, w2: &Wire) -> Vec<WirePoint> {
    let mut result = vec![];
    for w1s in &w1.inner {
        for w2s in &w2.inner {
            match w1s.intersection(*w2s) {
                WireIntersect::None => {},
                WireIntersect::Point(p) => {
                    if p != WirePoint::default() {
                        result.push(p)
                    }
                },
                WireIntersect::Run => {
                    warn!(
                        slog_scope::logger(),
                        "Run intersection: {:?}, {:?}", w1s, w2s
                    );
                },
            }
        }
    }
    result
}

fn min_wire_distance(w1: &Wire, w2: &Wire) -> Option<u64> {
    wire_crossings(w1, w2).iter().map(|p| p.sum() as u64).min()
}

fn min_wire_signal_distance(w1: &Wire, w2: &Wire) -> Option<u64> {
    wire_crossings(w1, w2)
        .iter()
        .map(|p| {
            w1.distance_to_point(*p).expect("invalid intersection")
                + w2.distance_to_point(*p).expect("invalid intersection")
        })
        .min()
}

pub fn part1(input: &str) -> Result<String> {
    let wires = parse_wires(input)?;
    let min_distance = min_wire_distance(&wires[0], &wires[1]).expect("no crossing found");
    Ok(format!("{:?}", min_distance))
}

pub fn part2(input: &str) -> Result<String> {
    let wires = parse_wires(input)?;
    let min_distance = min_wire_signal_distance(&wires[0], &wires[1]).expect("no crossing found");
    Ok(format!("{:?}", min_distance))
}

fn parse_wires(input: &str) -> Result<Vec<Wire>> {
    input.lines().map(|l| parse_wire(l)).collect()
}

fn parse_wire(input: &str) -> Result<Wire> {
    let mut start = WirePoint::default();
    let mut segments = Vec::new();
    for wr in input.split(',').map(|s| parse_wirerun(s)) {
        let end = start.add_run(wr?);
        segments.push(WireSegment { start, end });
        start = end
    }

    //.collect::<Result<_>>()?;
    Ok(Wire { inner: segments })
}

fn parse_wirerun(input: &str) -> Result<WireRun> {
    let (d, n) = input.split_at(1);
    let direction = match d {
        "U" => WireDirection::Up,
        "D" => WireDirection::Down,
        "L" => WireDirection::Left,
        "R" => WireDirection::Right,
        unk => panic!("direction '{}' is unknown", unk),
    };
    let step = parse_str::<u64>(n)?;

    let segment = WireRun { direction, step };
    //trace!(slog_scope::logger(), "{} -> {:?}", input, segment);
    Ok(segment)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn example_explanation() {
        let w1 = parse_wire("R8,U5,L5,D3").unwrap();
        let w2 = parse_wire("U7,R6,D4,L4").unwrap();
        let d = min_wire_distance(&w1, &w2).expect("no crossing found");
        assert_eq!(d, 6);
    }

    #[test]
    fn example_one() {
        let w1 = parse_wire("R75,D30,R83,U83,L12,D49,R71,U7,L72").unwrap();
        let w2 = parse_wire("U62,R66,U55,R34,D71,R55,D58,R83").unwrap();
        let d = min_wire_distance(&w1, &w2).expect("no crossing found");
        assert_eq!(d, 159);
    }

    #[test]
    fn example_two() {
        let w1 = parse_wire("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51").unwrap();
        let w2 = parse_wire("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7").unwrap();
        let d = min_wire_distance(&w1, &w2).expect("no crossing found");
        assert_eq!(d, 135);
    }

    #[test]
    fn example_explanation_signal() {
        let w1 = parse_wire("R8,U5,L5,D3").unwrap();
        let w2 = parse_wire("U7,R6,D4,L4").unwrap();
        let d = min_wire_signal_distance(&w1, &w2).expect("no crossing found");
        assert_eq!(d, 30);
    }

    #[test]
    fn example_one_signal() {
        let w1 = parse_wire("R75,D30,R83,U83,L12,D49,R71,U7,L72").unwrap();
        let w2 = parse_wire("U62,R66,U55,R34,D71,R55,D58,R83").unwrap();
        let d = min_wire_signal_distance(&w1, &w2).expect("no crossing found");
        assert_eq!(d, 610);
    }

    #[test]
    fn example_two_signal() {
        let w1 = parse_wire("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51").unwrap();
        let w2 = parse_wire("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7").unwrap();
        let d = min_wire_signal_distance(&w1, &w2).expect("no crossing found");
        assert_eq!(d, 410);
    }

    #[test]
    fn day3part1() {
        assert_eq!(part1(DAY3_INPUT).unwrap().as_str(), "221")
    }
    #[test]
    fn day3part2() {
        assert_eq!(part2(DAY3_INPUT).unwrap().as_str(), "18542")
    }
}
