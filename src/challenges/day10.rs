use self::space_map::{parse_map, Point, RadialMap};
use anyhow::Result;
use std::collections::HashSet;

pub fn part1(input: &str) -> Result<String> {
    let map = parse_map(input);
    let rmap = find_max_asteroid(&map).expect("map does not have two points");
    Ok(format!("{}", rmap.visable()))
}

pub fn part2(input: &str) -> Result<String> {
    let map = parse_map(input);
    let rmap = find_max_asteroid(&map).expect("map does not have two points");
    let p = rmap.sweep().nth(200 - 1).unwrap();
    let answer = p.x * 100 + p.y;
    Ok(format!("{}", answer))
}

fn find_max_asteroid(map: &HashSet<Point>) -> Option<RadialMap> {
    let mut max_point: Option<(RadialMap, usize)> = None;
    for p in map {
        let rmap = RadialMap::new(*p, map);
        let len = rmap.visable();
        let prev = if let Some((_, old_len)) = max_point {
            old_len
        } else {
            0
        };
        if len > prev {
            max_point = Some((rmap, len))
        }
    }
    max_point.map(|(r, _)| r)
}

mod space_map {
    pub use self::point::{Point, SortedPointArray};
    use std::cmp::Ordering;
    use std::collections::{BTreeMap, HashSet};
    use std::f64::consts::PI;

    pub fn parse_map(input: &str) -> HashSet<Point> {
        let mut points = HashSet::new();
        for (line, row) in input.lines().enumerate() {
            for (cidx, c) in row.chars().enumerate() {
                if c == '#' {
                    points.insert(Point::new(cidx as i64, line as i64));
                }
            }
        }
        points
    }

    mod point {
        use num::Integer;
        use std::cmp::{Ordering, Reverse};
        use std::collections::BinaryHeap;
        use std::fmt;
        use std::ops::{Add, Mul, Sub};
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct Point {
            pub x: i64,
            pub y: i64,
        }

        impl Point {
            pub fn new(x: i64, y: i64) -> Self {
                Self { x, y }
            }

            pub fn ray(self, other: Point) -> Point {
                if self == other {
                    return Point::new(0, 0);
                }
                let diff = other - self;
                let gcd = diff.x.gcd(&diff.y);
                Point {
                    x: diff.x / gcd,
                    y: diff.y / gcd,
                }
            }
        }

        impl fmt::Debug for Point {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "({}, {})", self.x, self.y)
            }
        }

        impl Add<Point> for Point {
            type Output = Point;
            fn add(self, rhs: Point) -> Self::Output {
                Point {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                }
            }
        }
        impl Sub<Point> for Point {
            type Output = Point;
            fn sub(self, rhs: Point) -> Self::Output {
                Point {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                }
            }
        }

        impl Mul<i64> for Point {
            type Output = Point;
            fn mul(self, rhs: i64) -> Self::Output {
                Point {
                    x: self.x * rhs,
                    y: self.y * rhs,
                }
            }
        }

        pub struct SortedPointArray {
            origin: Point,
            heap: BinaryHeap<Reverse<PointVec>>,
        }

        impl SortedPointArray {
            pub fn new(origin: Point) -> SortedPointArray {
                SortedPointArray {
                    origin,
                    heap: BinaryHeap::new(),
                }
            }
            pub fn insert(&mut self, point: Point) {
                let v = PointVec {
                    src: self.origin,
                    dst: point,
                };
                self.heap.push(Reverse(v));
            }

            pub fn into_vec(mut self) -> Vec<Point> {
                let mut out = Vec::with_capacity(self.heap.len());
                while let Some(v) = self.heap.pop() {
                    out.push(v.0.dst)
                }
                out
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct PointVec {
            src: Point,
            dst: Point,
        }

        impl Ord for PointVec {
            fn cmp(&self, other: &Self) -> Ordering {
                self.partial_cmp(other).unwrap()
            }
        }

        impl PartialOrd for PointVec {
            fn partial_cmp(&self, other: &PointVec) -> Option<Ordering> {
                fn magnitude_squared(p: Point) -> i64 {
                    p.x * p.x + p.y * p.y
                }
                magnitude_squared(self.dst - self.src)
                    .partial_cmp(&magnitude_squared(other.dst - other.src))
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;

            fn check_sort_points<F: Fn(Point, &[Point]) -> Vec<Point>>(f: F) {
                let src = Point::new(11, 13);
                let points = vec![
                    Point::new(11, 12),
                    Point::new(11, 2),
                    Point::new(11, 3),
                    Point::new(11, 5),
                    Point::new(11, 1),
                ];

                let s_pts = f(src, &points);
                let yvals = s_pts.into_iter().map(|v| v.y).collect::<Vec<_>>();
                assert_eq!(yvals, vec![12, 5, 3, 2, 1]);
            }

            #[test]
            fn pointvec_order() {
                check_sort_points(|src: Point, points: &[Point]| {
                    let mut pv = points
                        .iter()
                        .map(|dst| PointVec { src, dst: *dst })
                        .collect::<Vec<_>>();
                    pv.sort();
                    pv.into_iter().map(|v| v.dst).collect::<Vec<_>>()
                });
            }

            #[test]
            fn sorted_point_array_order() {
                check_sort_points(|src: Point, points: &[Point]| {
                    let mut spa = SortedPointArray::new(src);
                    for p in points {
                        spa.insert(*p);
                    }
                    spa.into_vec()
                });
            }

            #[test]
            fn point_add() {
                let p1 = Point::new(3, 4);
                let p2 = Point::new(10, 100);
                let p3 = p1 + p2;
                assert_eq!(p3, Point::new(13, 104));
            }

            #[test]
            fn point_sub() {
                let p1 = Point::new(3, 4);
                let p2 = Point::new(10, 100);
                let p3 = p1 - p2;
                assert_eq!(p3, Point::new(-7, -96));
            }

            #[test]
            fn point_mul_scalar() {
                let p1 = Point::new(3, 4);
                let p2 = p1 * 3;
                assert_eq!(p2, Point::new(9, 12));
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct RadiallySortedPoint(Point);

    impl RadiallySortedPoint {
        /// This considers zero at the 12 o'clock position
        /// and clockwise rotation is the positive direction.
        pub fn unit_circle(self) -> f64 {
            let x = self.0.x as f64;
            let y = self.0.y as f64;
            let t = y.atan2(x);
            ((t + PI / 2.0) + 2.0 * PI).rem_euclid(2.0 * PI)
        }
    }

    impl Ord for RadiallySortedPoint {
        fn cmp(&self, other: &Self) -> Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    impl PartialOrd for RadiallySortedPoint {
        fn partial_cmp(&self, other: &RadiallySortedPoint) -> Option<Ordering> {
            self.unit_circle().partial_cmp(&other.unit_circle())
        }
    }

    #[derive(Debug)]
    pub struct RadialMap {
        pub origin: Point,
        points: Vec<Vec<Point>>,
    }

    impl RadialMap {
        pub fn new(origin: Point, points: &HashSet<Point>) -> RadialMap {
            let mut builder = BTreeMap::new();
            let mut flat_map = Vec::new();

            for p in points {
                if *p == origin {
                    continue;
                }
                let ray = RadiallySortedPoint(origin.ray(*p));
                let heap = builder
                    .entry(ray)
                    .or_insert_with(|| SortedPointArray::new(origin));
                heap.insert(*p);
            }
            for (_, angle_points) in builder {
                flat_map.push(angle_points.into_vec())
            }
            RadialMap {
                origin,
                points: flat_map,
            }
        }
        pub fn visable(&self) -> usize {
            self.points.len()
        }

        pub fn sweep(&self) -> RadialSweep<'_> {
            RadialSweep::new(self)
        }
    }

    pub struct RadialSweep<'a> {
        map: &'a RadialMap,
        theta: usize,
        radius: usize,
        depth: usize,
    }

    impl<'a> RadialSweep<'a> {
        fn new(map: &'a RadialMap) -> RadialSweep<'a> {
            let depth = map.points.iter().map(|c| c.len()).max().unwrap();
            RadialSweep {
                map,
                theta: 0,
                radius: 0,
                depth,
            }
        }
        fn get(&self) -> Option<Point> {
            let row = &self.map.points[self.theta];
            if self.radius < row.len() {
                Some(row[self.radius])
            } else {
                None
            }
        }
        fn increment(&mut self) {
            self.theta += 1;
            if self.theta == self.map.points.len() {
                self.theta = 0;
                self.radius += 1;
            }
        }
    }

    impl<'a> Iterator for RadialSweep<'a> {
        type Item = Point;
        fn next(&mut self) -> Option<Self::Item> {
            while self.radius < self.depth {
                let p = self.get();
                self.increment();
                if let Some(p) = p {
                    return Some(p);
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;
    use std::collections::HashMap;

    const ADVENT_LARGE_EXAMPLE: &str = "\
        .#..##.###...#######\n\
        ##.############..##.\n\
        .#.######.########.#\n\
        .###.#######.####.#.\n\
        #####.##.#.##.###.##\n\
        ..#####..#.#########\n\
        ####################\n\
        #.####....###.#.#.##\n\
        ##.#################\n\
        #####.##.###..####..\n\
        ..######..##.#######\n\
        ####.##.####...##..#\n\
        .#####..#.######.###\n\
        ##...#.##########...\n\
        #.##########.#######\n\
        .####.#.###.###.#.##\n\
        ....##.##.###..#####\n\
        .#.#.###########.###\n\
        #.#.#.#####.####.###\n\
        ###.##.####.##.#..##
    ";

    fn check_map_input(input: &str, p: Point, len: usize) {
        let map = parse_map(input);
        let rmap = find_max_asteroid(&map).expect("map does not have two points");
        assert_eq!(rmap.origin, p);
        assert_eq!(rmap.visable(), len);
    }

    #[test]
    fn advent_example_1() {
        let input = "\
            .#..#\n\
            .....\n\
            #####\n\
            ....#\n\
            ...##
        ";
        check_map_input(input, Point::new(3, 4), 8);
    }

    #[test]
    fn advent_example_2() {
        let input = "\
            ......#.#.\n\
            #..#.#....\n\
            ..#######.\n\
            .#.#.###..\n\
            .#..#.....\n\
            ..#....#.#\n\
            #..#....#.\n\
            .##.#..###\n\
            ##...#..#.\n\
            .#....####
        ";
        check_map_input(input, Point::new(5, 8), 33);
    }

    #[test]
    fn advent_example_3() {
        let input = "\
            #.#...#.#.\n\
            .###....#.\n\
            .#....#...\n\
            ##.#.#.#.#\n\
            ....#.#.#.\n\
            .##..###.#\n\
            ..#...##..\n\
            ..##....##\n\
            ......#...\n\
            .####.###.
        ";
        check_map_input(input, Point::new(1, 2), 35);
    }
    #[test]
    fn advent_example_4() {
        let input = "\
            .#..#..###\n\
            ####.###.#\n\
            ....###.#.\n\
            ..###.##.#\n\
            ##.##.#.#.\n\
            ....###..#\n\
            ..#.#..#.#\n\
            #..#.#.###\n\
            .##...##.#\n\
            .....#.#..
        ";
        check_map_input(input, Point::new(6, 3), 41);
    }

    #[test]
    fn advent_example_5() {
        let input = ADVENT_LARGE_EXAMPLE;
        check_map_input(input, Point::new(11, 13), 210);
    }

    #[test]
    fn advent_p2_asteroid_sweep() {
        let input = ADVENT_LARGE_EXAMPLE;
        let mut checkpoints = HashMap::new();
        checkpoints.insert(1, Point::new(11, 12));
        checkpoints.insert(2, Point::new(12, 1));
        checkpoints.insert(3, Point::new(12, 2));
        checkpoints.insert(10, Point::new(12, 8));
        checkpoints.insert(20, Point::new(16, 0));
        checkpoints.insert(50, Point::new(16, 9));
        checkpoints.insert(100, Point::new(10, 16));
        checkpoints.insert(199, Point::new(9, 6));
        checkpoints.insert(200, Point::new(8, 2));
        checkpoints.insert(201, Point::new(10, 9));
        checkpoints.insert(299, Point::new(11, 1));

        let map = parse_map(input);
        let rmap = find_max_asteroid(&map).expect("map does not have two points");
        assert_eq!(rmap.origin, Point::new(11, 13));
        for (idx, a) in rmap.sweep().enumerate() {
            let count = idx + 1;
            if let Some(e) = checkpoints.get(&count) {
                assert_eq!(a, *e, "idx: {}", idx)
            }
        }
    }

    #[test]
    fn check_part1() {
        assert_eq!(part1(DAY10_INPUT).unwrap().as_str(), "267")
    }

    #[test]
    fn check_part2() {
        assert_eq!(part2(DAY10_INPUT).unwrap().as_str(), "1309")
    }
}
