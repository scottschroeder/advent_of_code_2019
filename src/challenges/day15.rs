use crate::challenges::day15::o2repair::Robot;
use crate::display::ImageNormal;
use crate::intcode::IntCode;
use crate::util::parse_intcode;
use anyhow::Result;

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let mut ic = IntCode::new_from_device(intcode, Robot::new());
    let _ = ic.run_till_end();
    let (_, robot) = ic.emit();

    let img = ImageNormal::create(&robot.map.inner);
    //img.display_grid(true);
    log::info!("{}", img);
    let o2 = robot.map.o2system().unwrap();
    let path = robot.map.path((0, 0).into(), o2)?;
    Ok(format!("{}", path.len() - 1))
}

pub fn part2(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let mut ic = IntCode::new_from_device(intcode, Robot::new());
    let _ = ic.run_till_end();
    let (_, robot) = ic.emit();

    let o2 = robot.map.o2system().unwrap();
    let path = robot.map.longest_path(o2)?;
    Ok(format!("{}", path.len() - 1))
}

mod o2repair {
    use crate::display::Point;
    use crate::intcode::intcode_io::{Input, Output};
    use anyhow::{anyhow as ah, Result};
    use std::collections::HashMap;
    use std::fmt;

    type Graph = petgraph::graphmap::GraphMap<Point, (), petgraph::Undirected>;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Tile {
        Empty,
        Wall,
        O2System,
        Unknown,
    }

    impl fmt::Display for Tile {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Tile::Empty => ".",
                    Tile::Wall => "X",
                    Tile::O2System => "O",
                    Tile::Unknown => "?",
                }
            )
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum Status {
        Wall,
        Move,
        O2,
    }

    impl From<i64> for Status {
        fn from(i: i64) -> Self {
            match i {
                0 => Status::Wall,
                1 => Status::Move,
                2 => Status::O2,
                _ => unreachable!("unhandled robot status {}", i),
            }
        }
    }

    fn adjacent_points(p: Point) -> Vec<Point> {
        Direction::rose().iter().map(|d| d.moved(p)).collect()
    }

    #[derive(Debug, Clone, Copy)]
    enum Direction {
        North,
        South,
        West,
        East,
    }

    impl Direction {
        fn moved(self, p: Point) -> Point {
            match self {
                Direction::North => (p.x, p.y + 1),
                Direction::South => (p.x, p.y - 1),
                Direction::West => (p.x - 1, p.y),
                Direction::East => (p.x + 1, p.y),
            }
            .into()
        }
        fn rose() -> [Direction; 4] {
            [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ]
        }
        fn step(src: Point, dst: Point) -> Option<Direction> {
            match (dst.x - src.x, dst.y - src.y) {
                (0, 1) => Some(Direction::North),
                (0, -1) => Some(Direction::South),
                (-1, 0) => Some(Direction::West),
                (1, 0) => Some(Direction::East),
                _ => None,
            }
        }
    }

    impl From<Direction> for i64 {
        fn from(d: Direction) -> Self {
            match d {
                Direction::North => 1,
                Direction::South => 2,
                Direction::West => 3,
                Direction::East => 4,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Deck {
        pub inner: HashMap<Point, Tile>,
        graph: Graph,
    }

    impl Deck {
        fn new() -> Deck {
            let inner = HashMap::new();
            let mut d = Deck {
                inner,
                graph: Graph::new(),
            };
            d.mark((0, 0).into(), Tile::Empty);
            d
        }
        fn mark_unknown(&mut self, p: Point) -> bool {
            let t = *self.inner.entry(p).or_insert(Tile::Unknown);
            if t == Tile::Unknown {
                self.mark_traverseable(p);
                true
            } else {
                false
            }
        }
        fn mark(&mut self, p: Point, t: Tile) {
            self.inner.insert(p, t);
            match t {
                Tile::Wall => self.mark_unreachable(p),
                Tile::Empty | Tile::O2System | Tile::Unknown => self.mark_traverseable(p),
            }
        }
        fn mark_unreachable(&mut self, p: Point) {
            self.graph.remove_node(p);
        }
        fn mark_traverseable(&mut self, p: Point) {
            self.graph.add_node(p);
            for adj_p in adjacent_points(p) {
                if self.graph.contains_node(adj_p) {
                    self.graph.add_edge(p, adj_p, ());
                }
            }
        }
        fn squares_with<F: Fn(Tile) -> bool>(&self, filter: F) -> Vec<Point> {
            self.inner
                .iter()
                .filter_map(|(p, t)| if filter(*t) { Some(*p) } else { None })
                .collect()
        }
        pub fn path(&self, src: Point, dst: Point) -> Result<Vec<Point>> {
            petgraph::algo::astar(
                &self.graph,
                src,
                |n| n == dst,
                |_| 1,
                |dst| src.step_dist(&dst),
            )
            .ok_or_else(|| ah!("could not chart course from {} to {}", src, dst))
            .map(|(_, path)| path)
        }

        pub fn longest_path(&self, src: Point) -> Result<Vec<Point>> {
            let mut bfs = petgraph::visit::Bfs::new(&self.graph, src);
            let mut terminal = None;
            while let Some(p) = bfs.next(&self.graph) {
                terminal = Some(p);
            }

            terminal
                .ok_or_else(|| ah!("bfs did not find any nodes"))
                .and_then(|dst| self.path(src, dst))
        }
        pub fn o2system(&self) -> Option<Point> {
            self.squares_with(|t| t == Tile::O2System)
                .iter()
                .cloned()
                .next()
        }
    }

    #[derive(Debug)]
    pub struct Robot {
        loc: Point,
        pub map: Deck,
        cmd: Direction,
        dst: Vec<Point>,
    }

    impl Robot {
        pub fn new() -> Robot {
            Robot {
                loc: Point::new(0, 0),
                map: Deck::new(),
                cmd: Direction::North,
                dst: vec![Point::new(0, 1)],
            }
        }
        fn mark_neighbors(&mut self) {
            self.dst.clear();
            let loc = self.loc;
            let mut target = None;
            for p in adjacent_points(loc).into_iter().rev() {
                if self.map.mark_unknown(p) {
                    target = Some(p);
                }
            }
            if let Some(p) = target {
                self.dst.push(p)
            }
        }
        fn new_frontier(&self) -> Option<Point> {
            self.map
                .squares_with(|t| t == Tile::Unknown)
                .iter()
                .map(|p| (*p, self.loc.step_dist(p)))
                .fold(None, |acc: Option<(Point, i32)>, x| {
                    if let Some(prev) = acc {
                        Some(if prev.1 < x.1 { prev } else { x })
                    } else {
                        Some(x)
                    }
                })
                .map(|acc| acc.0)
        }
    }

    impl Input for Robot {
        fn input(&mut self) -> Result<i64> {
            let dst = self
                .dst
                .pop()
                .or_else(|| self.new_frontier())
                .ok_or_else(|| ah!("no destination"))?;
            if let Some(cmd) = Direction::step(self.loc, dst) {
                self.cmd = cmd;
                return Ok(cmd.into());
            }
            let mut path = self.map.path(self.loc, dst)?;
            if path.len() < 3 {
                return Err(ah!(
                    "path too short? src: {} dst: {} path: {:?}",
                    self.loc,
                    dst,
                    path
                ));
            }
            path.reverse();
            path.pop();
            let dst = path.pop().unwrap();
            let cmd = Direction::step(self.loc, dst).ok_or_else(|| {
                ah!(
                    "path first step was not adjacent? src: {} dst: {}",
                    self.loc,
                    dst
                )
            })?;
            self.cmd = cmd;
            Ok(cmd.into())
        }
    }

    impl Output for Robot {
        fn output(&mut self, out: i64) -> Result<()> {
            let status = Status::from(out);
            log::trace!(
                "Robot {} went {:?}, found: {:?}",
                self.loc,
                self.cmd,
                status
            );
            let pos = self.cmd.moved(self.loc);
            let tile = match status {
                Status::Wall => Tile::Wall,
                Status::Move => {
                    self.loc = pos;
                    Tile::Empty
                }
                Status::O2 => {
                    self.loc = pos;
                    Tile::O2System
                }
            };
            if self.dst.is_empty() {
                self.map.mark(pos, tile);
                self.mark_neighbors();
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day15part1() {
        assert_eq!(part1(DAY15_INPUT).unwrap().as_str(), "300")
    }

    #[test]
    fn day15part2() {
        assert_eq!(part2(DAY15_INPUT).unwrap().as_str(), "312")
    }
}
