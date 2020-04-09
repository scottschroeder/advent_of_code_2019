use crate::challenges::day17::Tile::Scaffold;
use crate::display::Point;
use crate::intcode::run_intcode;
use crate::util::parse_intcode;
use anyhow::{anyhow as ah, Result};
use petgraph::visit::IntoEdges;
use std::collections::HashMap;
use std::sync::mpsc::RecvTimeoutError::Timeout;

type Graph = petgraph::graphmap::UnGraphMap<Point, ()>;

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let (_, out) = run_intcode(intcode, vec![])?;
    let (m, r) = Map::from_render(out.as_slice())?;
    //trace!(slog_scope::logger(), "m: {:?}, r: {:#?}", m, r);
    let intersections = m.intersections();
    trace!(slog_scope::logger(), "intersections: {:#?}", intersections);

    let s = String::from_utf8(out.iter().map(|x| *x as u8).collect::<Vec<_>>())?;
    trace!(slog_scope::logger(), "map:\n{}", s);

    Ok(format!(
        "{}",
        intersections.iter().map(|p| p.x * p.y).sum::<i32>()
    ))
}

pub fn part2(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let (_, out) = run_intcode(intcode, vec![2])?;
    program_walk(out.as_slice())?;

    Ok(format!("{}", out[0]))
}

// Take a map directly as input
pub fn part2_map(input: &str) -> Result<String> {
    let out = input
        .as_bytes()
        .iter()
        .map(|x| *x as i64)
        .collect::<Vec<_>>();
    program_walk(out.as_slice())?;

    Ok(format!("{}", out[0]))
}

fn program_walk(map_data: &[i64]) -> Result<()> {
    let (m, r) = Map::from_render(map_data)?;
    let intersections = m.intersections();
    trace!(slog_scope::logger(), "intersections: {:#?}", intersections);
    let g = m.to_graph();
    if true {
        trace!(slog_scope::logger(), "graph: {:#?}", g);
        for n in g.nodes() {
            print!("{}: ", n);
            for (src, dst, _) in g.edges(n) {
                print!("[{}->{}], ", src, dst);
            }
            println!("");
        }
    }

    let paths = ScaffoldSearcher::new(&m, &g, r.loc)
        .map(|p| {
            debug!(slog_scope::logger(), "{:?}", p);
            p
        })
        .collect::<Vec<WalkPath>>();
    debug!(slog_scope::logger(), "found {} intersections", paths.len());
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Intersection {
    Unvisited,
    Entered {
        first_approach: Direction,
        first_departure: Direction,
    },
    Completed {
        first_departure: Direction,
        second_departure: Direction,
    },
}

impl Intersection {
    fn adjust(&self, p: Point) -> Point {
        match self {
            Intersection::Unvisited => unreachable!("empty intersection has no direction"),
            Intersection::Entered {
                first_departure, ..
            } => first_departure.adjust(p),
            Intersection::Completed {
                second_departure, ..
            } => second_departure.adjust(p),
        }
    }
}

struct ScaffoldSearcher<'a> {
    stack: Vec<ScaffoldWalker<'a>>,
}

impl<'a> ScaffoldSearcher<'a> {
    fn new(map: &'a Map, graph: &'a Graph, start: Point) -> ScaffoldSearcher<'a> {
        let walker = ScaffoldWalker::new(map, graph, start);
        ScaffoldSearcher {
            stack: vec![walker],
        }
    }
}

impl<'a> Iterator for ScaffoldSearcher<'a> {
    type Item = WalkPath;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(walker) = self.stack.pop() {
            if let Some(path) = walker.walk(&mut self.stack) {
                return Some(path);
            }
        }
        None
    }
}

// Kinda like an iterator
#[derive(Clone)]
struct ScaffoldWalker<'a> {
    loc: Point,
    prev: Point,
    intersections: HashMap<Point, Intersection>,
    map: &'a Map,
    graph: &'a Graph,
}

type WalkPath = HashMap<Point, (Direction, Direction)>;

impl<'a> ScaffoldWalker<'a> {
    fn complete(self) -> Option<WalkPath> {
        self.intersections
            .into_iter()
            .map(|(p, i)| {
                if let Intersection::Completed {
                    first_departure,
                    second_departure,
                } = i
                {
                    Ok((p, (first_departure, second_departure)))
                } else {
                    Err(())
                }
            })
            .collect::<Result<WalkPath, ()>>()
            .ok()
    }
    fn new(map: &'a Map, graph: &'a Graph, start: Point) -> ScaffoldWalker<'a> {
        let intersections = map
            .intersections()
            .into_iter()
            .map(|p| (p, Intersection::Unvisited))
            .collect();
        ScaffoldWalker {
            loc: start,
            prev: start,
            intersections,
            map,
            graph,
        }
    }
    fn branch(&self, incoming: Direction, branches: &mut Vec<ScaffoldWalker<'a>>) {
        for d in Direction::rose().iter() {
            let mut sub_walker = self.clone();
            if *d == incoming {
                continue;
            }
            //let intersection = Intersection::Entered(incoming, *d);
            let intersection = Intersection::Entered {
                first_approach: incoming,
                first_departure: *d,
            };
            sub_walker
                .intersections
                .insert(sub_walker.loc, intersection);
            sub_walker.prev = sub_walker.loc;
            sub_walker.loc = intersection.adjust(sub_walker.loc);
            trace!(
                slog_scope::logger(),
                "creating branch: {:?} {} -> {}",
                intersection,
                sub_walker.prev,
                sub_walker.loc
            );
            branches.push(sub_walker);
        }
    }
    fn path_step(&mut self) {
        for (src, dst, _) in self.graph.edges(self.loc) {
            if dst == self.prev {
                continue;
            }
            trace!(
                slog_scope::logger(),
                "regular_step: prev: {} loc: {} -> src: {} dst: {}",
                self.prev,
                self.loc,
                src,
                dst
            );
            self.prev = self.loc;
            self.loc = dst;
            return;
        }
    }
    fn step(&mut self, branches: &mut Vec<ScaffoldWalker<'a>>) {
        trace!(
            slog_scope::logger(),
            "step: prev: {}, loc: {}",
            self.prev,
            self.loc
        );
        if let Some(intersect) = self.intersections.get(&self.loc).cloned() {
            let d = self.prev - self.loc;
            let incoming = Direction::from_delta(d).unwrap();
            trace!(
                slog_scope::logger(),
                "we are at an intersection {:?}, entered from: {:?}",
                intersect,
                incoming
            );
            match intersect {
                Intersection::Unvisited => {
                    // This walker will be replaced by three children
                    self.branch(incoming, branches);
                    return;
                }
                Intersection::Entered {
                    first_approach,
                    first_departure,
                } => {
                    trace!(
                        slog_scope::logger(),
                        "first entrance: {:?}, first exit: {:?}, current entrance: {:?}",
                        first_approach,
                        first_departure,
                        incoming,
                    );
                    if let Some(outgoing) = incoming.last_direction(first_approach, first_departure)
                    {
                        let new_state = Intersection::Completed {
                            first_departure,
                            second_departure: outgoing,
                        };
                        self.intersections.insert(self.loc, new_state);
                        self.prev = self.loc;
                        self.loc = new_state.adjust(self.loc);
                    } else {
                        trace!(slog_scope::logger(), "dead end: {:?}", intersect);
                        return;
                    }
                }
                Intersection::Completed { .. } => return,
            };
        } else {
            self.path_step();
        }
        trace!(slog_scope::logger(), "walk: {}", self.loc);
    }
    fn walk(mut self, branches: &mut Vec<ScaffoldWalker<'a>>) -> Option<WalkPath> {
        let mut save = Point::new(-1, -1);
        while self.loc != save {
            save = self.loc;
            self.step(branches)
        }
        self.complete()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn from_delta(p: Point) -> Option<Direction> {
        match (p.x, p.y) {
            (0, -1) => Some(Direction::North),
            (1, 0) => Some(Direction::East),
            (0, 1) => Some(Direction::South),
            (-1, 0) => Some(Direction::West),
            _ => None,
        }
    }
    fn from_carrot(c: char) -> Direction {
        match c {
            '^' => Direction::North,
            '>' => Direction::East,
            'v' => Direction::South,
            '<' => Direction::West,
            _ => unreachable!("unknown direction char: {:?}", c),
        }
    }
    fn adjust(&self, p: Point) -> Point {
        match self {
            Direction::North => (p.x, p.y - 1),
            Direction::East => (p.x + 1, p.y),
            Direction::South => (p.x, p.y + 1),
            Direction::West => (p.x - 1, p.y),
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
    fn last_direction(self, d1: Direction, d2: Direction) -> Option<Direction> {
        let p = Point::new(0, 0);
        let off = d2.adjust(d1.adjust(self.adjust(p)));
        Direction::from_delta(p - off)
    }
}

fn adjacent_points(p: Point) -> Vec<Point> {
    Direction::rose().iter().map(|d| d.adjust(p)).collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tile {
    Void,
    Scaffold,
}

impl Tile {
    fn from_ascii(c: u8) -> Option<Tile> {
        match c {
            35 => Some(Tile::Scaffold),
            46 => Some(Tile::Void),
            10 => None,
            _ => unreachable!("unknown char: {} = {:?}", c, c as char),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Map {
    data: Vec<Tile>,
    width: usize,
}

#[inline]
fn index_to_point(w: usize, idx: usize) -> Point {
    let y = idx / w;
    let x = idx % w;
    Point::new(x as i32, y as i32)
}

#[inline]
fn point_to_index(w: usize, p: Point) -> usize {
    let x = p.x as usize;
    let y = p.y as usize;
    //trace!(slog_scope::logger(), "p2i w={} p={}", w, p);
    y * w + x
}

impl Map {
    fn to_graph(&self) -> Graph {
        let mut g = Graph::new();
        for s in self.scaffold() {
            g.add_node(s);
            for adj_p in adjacent_points(s) {
                if self.is_scaffold(adj_p) {
                    trace!(
                        slog_scope::logger(),
                        "graph: check adj {} -> {} (true)",
                        s,
                        adj_p
                    );
                    g.add_node(adj_p);
                    g.add_edge(s, adj_p, ());
                } else {
                    trace!(
                        slog_scope::logger(),
                        "graph: check adj {} -> {} (false)",
                        s,
                        adj_p
                    );
                }
            }
        }
        g
    }
    fn intersections(&self) -> Vec<Point> {
        self.scaffold()
            .filter(|p| {
                let paths = adjacent_points(*p)
                    .iter()
                    .filter(|adj| self.is_scaffold(**adj))
                    .count();
                paths >= 3
            })
            .collect()
    }
    #[inline]
    fn width(&self) -> i32 {
        self.width as i32
    }
    #[inline]
    fn height(&self) -> i32 {
        (self.data.len() / self.width) as i32
    }
    #[inline]
    fn point_on_board(&self, p: Point) -> bool {
        p.x >= 0 && p.x < self.width() && p.y >= 0 && p.y < self.height()
    }
    #[inline]
    fn get_tile(&self, p: Point) -> Tile {
        let idx = point_to_index(self.width, p);
        self.data[idx]
    }
    fn is_scaffold(&self, p: Point) -> bool {
        self.point_on_board(p) && self.get_tile(p) == Tile::Scaffold
    }
    fn scaffold(&self) -> impl Iterator<Item = Point> + '_ {
        let w = self.width;
        self.data.iter().enumerate().filter_map(move |(idx, t)| {
            if *t == Tile::Scaffold {
                Some(index_to_point(w, idx))
            } else {
                None
            }
        })
    }
    fn from_render(data: &[i64]) -> Result<(Map, Robot)> {
        let mut inner = vec![];
        let mut width = None;
        let mut orientation = None;
        let mut loc = None;

        let mut x = 0;
        let mut y = 0;
        for (idx, encoded) in data.iter().enumerate() {
            let c = *encoded as u8 as char;
            match c {
                '.' => inner.push(Tile::Void),
                '#' => inner.push(Tile::Scaffold),
                '\n' => {
                    width.get_or_insert(idx);
                    x = -1;
                    y += 1;
                }
                '^' | '>' | '<' | 'v' => {
                    orientation.get_or_insert(Direction::from_carrot(c));
                    loc.get_or_insert(Point::new(x, y));
                    inner.push(Tile::Scaffold);
                }
                c => return Err(ah!("unknown camera char: {} => {:?}", x, c)),
            }
            x += 1;
        }

        Ok((
            Map {
                data: inner,
                width: width.ok_or_else(|| ah!("data did not contain a newline"))?,
            },
            Robot {
                orientation: orientation.ok_or_else(|| ah!("no robot orientation found"))?,
                loc: loc.ok_or_else(|| ah!("no robot location found"))?,
            },
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Robot {
    orientation: Direction,
    loc: Point,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn day17part1() {
        assert_eq!(part1(DAY17_INPUT).unwrap().as_str(), "3336")
    }

    #[test]
    fn day17part2() {
        //assert_eq!(part2(DAY17_INPUT).unwrap().as_str(), "0")
    }
}
