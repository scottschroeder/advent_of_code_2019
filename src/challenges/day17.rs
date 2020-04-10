use crate::{display::Point, intcode::run_intcode, util::parse_intcode};
use anyhow::{anyhow as ah, Result};
use std::{collections::HashMap, fmt};

type Graph = petgraph::graphmap::DiGraphMap<Point, Vec<Direction>>;
type CGraph = petgraph::graphmap::DiGraphMap<CNode, PathInstructions>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum CNode {
    Edge(Point),
    Intersection(Point),
}

#[derive(Clone)]
struct PathInstructions {
    src: Point,
    dst: Point,
    path: Vec<Direction>,
}

impl PathInstructions {
    fn reverse(&self) -> PathInstructions {
        PathInstructions {
            src: self.dst,
            dst: self.src,
            path: self.path.iter().rev().map(|d| d.reverse()).collect(),
        }
    }
}

impl fmt::Display for PathInstructions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for d in &self.path {
            write!(f, "{}", d)?
        }
        Ok(())
    }
}

impl fmt::Debug for PathInstructions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self, self.dst)
    }
}

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let (_, out) = run_intcode(intcode, vec![])?;
    let (m, _) = Map::from_render(out.as_slice())?;
    //trace!(slog_scope::logger(), "m: {:?}, r: {:#?}", m, r);
    let intersections = m.intersections().collect::<Vec<_>>();
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

fn print_graph(g: &Graph) {
    let dot = petgraph::dot::Dot::new(g);
    println!("{:?}", dot);
}

fn program_walk(map_data: &[i64]) -> Result<()> {
    let (m, r) = Map::from_render(map_data)?;

    let chunk_map = m.to_chunk_graph()?;
    trace!(slog_scope::logger(), "chunks: {:#?}", chunk_map);
    return Err(ah!("e"));

    let intersections = m.intersections().collect::<Vec<_>>();
    trace!(slog_scope::logger(), "intersections: {:#?}", intersections);
    let g = m.to_graph();
    print_graph(&g);
    return Err(ah!("e"));
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

const COMPASS_SIZE: usize = 4;
const COMPASS_ROSE: [Direction; COMPASS_SIZE] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl From<usize> for Direction {
    fn from(x: usize) -> Self {
        match x % COMPASS_SIZE {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            _ => unreachable!(),
        }
    }
}
impl From<Direction> for usize {
    fn from(x: Direction) -> Self {
        match x {
            Direction::North => 0,
            Direction::East => 1,
            Direction::South => 2,
            Direction::West => 3,
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::North => write!(f, "N"),
            Direction::East => write!(f, "E"),
            Direction::South => write!(f, "S"),
            Direction::West => write!(f, "W"),
        }
    }
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
    fn rose() -> [Direction; COMPASS_SIZE] {
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
    fn reverse(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }
}

fn adjacent_points(p: Point) -> Vec<(Direction, Point)> {
    Direction::rose()
        .iter()
        .map(|d| (*d, d.adjust(p)))
        .collect()
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

struct ChunkMapBuilder {
    intersections: HashMap<Point, [Option<PathInstructions>; COMPASS_SIZE]>,
    tails: HashMap<Point, PathInstructions>,
}
#[derive(Debug)]
struct ChunkMap {
    intersections: HashMap<Point, [PathInstructions; COMPASS_SIZE]>,
    tails: HashMap<Point, PathInstructions>,
}

fn finalize_intersection_paths(
    mut input: [Option<PathInstructions>; COMPASS_SIZE],
) -> Result<[PathInstructions; COMPASS_SIZE]> {
    let err = || ah!("incomplete chunkmap");
    Ok([
        input[0].take().ok_or_else(err)?,
        input[1].take().ok_or_else(err)?,
        input[2].take().ok_or_else(err)?,
        input[3].take().ok_or_else(err)?,
    ])
}

impl ChunkMapBuilder {
    fn add_path(&mut self, src: Point, heading: Direction, dst: Point, instr: PathInstructions) {
        debug_assert!(instr.path.len() > 0);
        let reverse = instr.reverse();
        self.intersections.get_mut(&src).unwrap()[usize::from(heading)] = Some(instr);

        let reverse_heading = reverse.path[0];
        if let Some(intersection) = self.intersections.get_mut(&dst) {
            intersection[usize::from(reverse_heading)] = Some(reverse);
        } else {
            self.tails.insert(dst, reverse);
        }
    }
    fn get_path(&self, src: Point, heading: Direction) -> Option<&PathInstructions> {
        self.intersections
            .get(&src)
            .and_then(|rose| rose[usize::from(heading)].as_ref())
    }
    fn finalize(self) -> Result<ChunkMap> {
        Ok(ChunkMap {
            intersections: self
                .intersections
                .into_iter()
                .map(|(p, paths)| {
                    finalize_intersection_paths(paths).map(|good_paths| (p, good_paths))
                })
                .collect::<Result<HashMap<Point, [PathInstructions; COMPASS_SIZE]>>>()?,
            tails: self.tails,
        })
    }
}

impl Map {
    fn to_graph(&self) -> Graph {
        let mut g = Graph::new();
        for s in self.scaffold() {
            g.add_node(s);
            for (d, adj_p) in adjacent_points(s) {
                if self.is_scaffold(adj_p) {
                    g.add_node(adj_p);
                    g.add_edge(s, adj_p, vec![d]);
                }
            }
        }
        g
    }
    fn to_chunk_graph(&self) -> Result<ChunkMap> {
        let mut chunks = ChunkMapBuilder {
            intersections: self
                .intersections()
                .map(|p| {
                    let builder = [None, None, None, None];
                    (p, builder)
                })
                .collect(),
            tails: HashMap::new(),
        };

        for (p, d) in self
            .intersections()
            .flat_map(|p| COMPASS_ROSE.iter().map(move |d| (p, *d)))
        {
            if chunks.get_path(p, d).is_some() {
                continue;
            }
            let mut path = vec![d];
            let mut current = d.adjust(p);
            let mut last_step = d;
            'walk: loop {
                let neighbors = adjacent_points(current)
                    .into_iter()
                    .filter(|(step_d, p)| *step_d != last_step.reverse() && self.is_scaffold(*p))
                    .collect::<Vec<(Direction, Point)>>();
                if neighbors.len() != 1 {
                    break 'walk;
                }
                let (step_d, new_loc) = neighbors[0];
                path.push(step_d);
                last_step = step_d;
                current = new_loc;
            }
            let mut instr = PathInstructions {
                src: p,
                dst: current,
                path: Vec::new(),
            };
            std::mem::swap(&mut path, &mut instr.path);
            chunks.add_path(p, d, current, instr);
        }
        chunks.finalize()
    }
    fn intersections(&self) -> impl Iterator<Item = Point> + '_ {
        self.scaffold().filter(move |p| {
            let paths = adjacent_points(*p)
                .iter()
                .filter(|(_, adj)| self.is_scaffold(*adj))
                .count();
            paths >= 3
        })
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
