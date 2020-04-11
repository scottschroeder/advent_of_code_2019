use crate::{display::Point, intcode::run_intcode, util::parse_intcode};
use anyhow::{anyhow as ah, Result};
use std::{collections::HashMap, fmt};

mod sequence_extractor;

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
    fn outgoing(&self) -> Direction {
        self.path[0]
    }
    fn incoming(&self) -> Direction {
        self.path[self.path.len() - 1]
    }
}

struct PrintablePath<'a>(&'a [Direction]);

impl<'a> fmt::Display for PrintablePath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for d in self.0 {
            write!(f, "{}", d)?
        }
        Ok(())
    }
}

impl fmt::Display for PathInstructions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p = PrintablePath(self.path.as_slice());
        write!(f, "{}", p)
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

fn program_walk(map_data: &[i64]) -> Result<()> {
    let (m, r) = Map::from_render(map_data)?;

    let chunk_map = m.to_chunk_graph()?;
    trace!(slog_scope::logger(), "chunks: {:#?}", chunk_map);

    let paths = ScaffoldSearcher::new(&m, &chunk_map, r.loc)
        .map(|p| {
            debug!(slog_scope::logger(), "{}", PrintablePath(p.as_slice()));
            let instr = Instruction::sequence(r.orientation, p.as_slice());
            debug!(slog_scope::logger(), "{}", PrintableInstructions(instr.as_slice()));
            let instr = compact_instructions(instr.as_slice());
            debug!(slog_scope::logger(), "{}", PrintableInstructions(instr.as_slice()));
            instr
        })
        .count();
    debug!(slog_scope::logger(), "found {} paths", paths);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Intersection {
    Unvisited,
    Arrived {
        first_approach: Direction,
    },
    Entered {
        first_approach: Direction,
        first_departure: Direction,
    },
    Returned {
        first_approach: Direction,
        first_departure: Direction,
        second_approach: Direction,
    },
    Completed,
}

impl Intersection {
    fn arrived(&self, d: Direction) -> Result<Option<Intersection>> {
        match self {
            Intersection::Unvisited => Ok(Some(Intersection::Arrived { first_approach: d })),
            Intersection::Arrived { .. } => Err(ah!("already arrived")),
            Intersection::Entered {
                first_approach,
                first_departure,
            } => Ok(if d != *first_approach && d != *first_departure {
                Some(Intersection::Returned {
                    first_approach: *first_approach,
                    first_departure: *first_departure,
                    second_approach: d,
                })
            } else {
                None
            }),
            Intersection::Returned { .. } => Err(ah!("already returned")),
            Intersection::Completed => {
                Err(ah!("can not arrive from intersection we have completed"))
            }
        }
    }
    fn departed(&self, d: Direction) -> Result<Option<Intersection>> {
        match self {
            Intersection::Unvisited => Err(ah!(
                "can not depart from an intersection we never arrived at"
            )),
            Intersection::Arrived { first_approach } => Ok(if d != *first_approach {
                Some(Intersection::Entered {
                    first_approach: *first_approach,
                    first_departure: d,
                })
            } else {
                None
            }),
            Intersection::Entered { .. } => {
                Err(ah!("can not depart from intersection we just left"))
            }
            Intersection::Returned {
                first_approach,
                first_departure,
                second_approach,
            } => Ok(
                if d != *first_approach && d != *first_departure && d != *second_approach {
                    Some(Intersection::Completed)
                } else {
                    None
                },
            ),
            Intersection::Completed => {
                Err(ah!("can not depart from intersection we have completed"))
            }
        }
    }
}

struct ScaffoldSearcher<'a> {
    stack: Vec<ScaffoldWalker<'a>>,
}

impl<'a> ScaffoldSearcher<'a> {
    fn new(map: &'a Map, graph: &'a ChunkMap, start: Point) -> ScaffoldSearcher<'a> {
        let walker = ScaffoldWalker::new(map, graph, start);
        ScaffoldSearcher {
            stack: vec![walker],
        }
    }
}

impl<'a> Iterator for ScaffoldSearcher<'a> {
    type Item = Vec<Direction>;
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
#[derive(Debug, Clone)]
struct ScaffoldWalker<'a> {
    loc: Point,
    intersections: HashMap<Point, Intersection>,
    path: Vec<Direction>,
    graph: &'a ChunkMap,
}

impl<'a> ScaffoldWalker<'a> {
    fn complete(self) -> Option<Vec<Direction>> {
        let is_complete = self.intersections.into_iter().all(|(_, i)| {
            if let Intersection::Completed = i {
                true
            } else {
                false
            }
        });
        if is_complete {
            Some(self.path)
        } else {
            None
        }
    }
    fn new(map: &'a Map, graph: &'a ChunkMap, start: Point) -> ScaffoldWalker<'a> {
        let intersections = map
            .intersections()
            .into_iter()
            .map(|p| (p, Intersection::Unvisited))
            .collect();
        ScaffoldWalker {
            loc: start,
            intersections,
            path: vec![],
            graph,
        }
    }
    fn walk(self, branches: &mut Vec<ScaffoldWalker<'a>>) -> Option<Vec<Direction>> {
        let mut branched = false;
        for path in self.graph.paths(self.loc) {
            if let Some(sub_walker) = self.child_from_path(path) {
                branched = true;
                trace!(slog_scope::logger(), "push");
                branches.push(sub_walker)
            }
        }
        if !branched {
            trace!(
                slog_scope::logger(),
                "check complete: {}",
                PrintablePath(self.path.as_slice())
            );
            self.complete()
        } else {
            None
        }
    }
    fn child_from_path(&self, path: &PathInstructions) -> Option<ScaffoldWalker<'a>> {
        trace!(
            slog_scope::logger(),
            "{} -> {:?} ({})",
            self.loc,
            path,
            PrintablePath(self.path.as_slice())
        );
        let mut sub_walker = self.clone();
        if let Some(src) = sub_walker.intersections.get_mut(&path.src) {
            match src.departed(path.outgoing()) {
                Ok(Some(intersect)) => *src = intersect,
                x => {
                    trace!(slog_scope::logger(), "Could not depart {:?}: {:?}", src, x);
                    return None;
                }
            }
        }
        if let Some(dst) = sub_walker.intersections.get_mut(&path.dst) {
            match dst.arrived(path.incoming().reverse()) {
                Ok(Some(intersect)) => *dst = intersect,
                x => {
                    trace!(slog_scope::logger(), "Could not arrive {:?}: {:?}", dst, x);
                    return None;
                }
            }
        }

        sub_walker.loc = path.dst;
        sub_walker.path.extend(path.path.iter());
        Some(sub_walker)
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
    fn from_carrot(c: char) -> Direction {
        match c {
            '^' => Direction::North,
            '>' => Direction::East,
            'v' => Direction::South,
            '<' => Direction::West,
            _ => unreachable!("unknown direction char: {:?}", c),
        }
    }
    fn turn(&self, desired: Direction) -> Instruction {
        let zero = Point::new(0,0);
        let src = self.adjust(zero);
        let dst = desired.adjust(zero);
        let cross = src.x * dst.y - src.y * dst.x;
        if cross > 0 {
            return Instruction::Right
        } else if cross < 0 {
            return Instruction::Left
        } else {
            panic!("this is not a turn")
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

impl ChunkMap {
    fn paths(&self, src: Point) -> impl Iterator<Item = &PathInstructions> + '_ {
        let paths = if let Some(instr) = self.intersections.get(&src) {
            PathOptions::List(instr)
        } else if let Some(instr) = self.tails.get(&src) {
            PathOptions::Single(instr)
        } else {
            PathOptions::None
        };

        PathIterator { paths, idx: 0 }
    }
}

enum PathOptions<'a> {
    List(&'a [PathInstructions; COMPASS_SIZE]),
    Single(&'a PathInstructions),
    None,
}

struct PathIterator<'a> {
    paths: PathOptions<'a>,
    idx: usize,
}
impl<'a> Iterator for PathIterator<'a> {
    type Item = &'a PathInstructions;
    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.paths {
            PathOptions::Single(p) => {
                if self.idx == 0 {
                    Some(p)
                } else {
                    None
                }
            }
            PathOptions::List(l) => {
                if self.idx > 3 {
                    None
                } else {
                    Some(&l[self.idx])
                }
            }
            PathOptions::None => None,
        };
        self.idx += 1;
        result
    }
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

enum Instruction {
    Right,
    Left,
    Forward(u32),
}
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Right => write!(f, "R"),
            Instruction::Left => write!(f, "L"),
            Instruction::Forward(n) => write!(f, "{}", n),
        }
    }
}

struct PrintableInstructions<'a>(&'a [Instruction]);

impl<'a> fmt::Display for PrintableInstructions<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in self.0 {
            write!(f, "{}", i)?
        }
        Ok(())
    }
}

impl<'a> fmt::Debug for PrintableInstructions<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)

    }
}

impl Instruction {
    fn sequence(orientation: Direction, directions: &[Direction]) -> Vec<Instruction> {
        let mut o = orientation;
        let mut cmd = Vec::with_capacity(directions.len());
        for d in directions {
            if o != *d {
                cmd.push(o.turn(*d));
                o = *d;
            }
            cmd.push(Instruction::Forward(1))
        }
        cmd
    }
}
fn compact_instructions(instr: &[Instruction]) -> Vec<Instruction> {
    let mut cmd = Vec::new();
    let mut run = 0;
    for i in instr {
        match i {
            Instruction::Right => {
                if run > 0 {
                    cmd.push(Instruction::Forward(run));
                    run = 0;
                }
                cmd.push(Instruction::Right)
            }
            Instruction::Left => {
                if run > 0 {
                    cmd.push(Instruction::Forward(run));
                    run = 0;
                }
                cmd.push(Instruction::Left);
            }
            Instruction::Forward(n) => {
                run += n;
            }
        }
    }
    if run > 0 {
        cmd.push(Instruction::Forward(run));
    }
    cmd
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
