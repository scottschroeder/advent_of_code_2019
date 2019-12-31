use self::space_map::{Map, Point};
use anyhow::Result;

pub fn part1(input: &str) -> Result<String> {
    let map = Map::from(input);
    let max_point = find_max_asteroid(&map).expect("map does not have two points");
    Ok(format!("{}", max_point.1))
}

pub fn part2(input: &str) -> Result<String> {
    Ok(format!("{}", 0))
}

fn find_max_asteroid(map: &Map) -> Option<(Point, usize)> {
    let mut max_point: Option<(Point, usize)> = None;
    for p in &map.data {
        let vis = map.visible_from(*p);
        let len = vis.len();
        if len > max_point.unwrap_or((Point::new(-1, -1), 0)).1 {
            max_point = Some((*p, len))
        }
    }
    max_point
}

mod space_map {
    pub use self::point::{LinePoints, Point};
    use std::collections::HashSet;

    fn parse_map(input: &str) -> Map {
        let mut max_w = 0usize;
        let mut line = 0usize;
        let mut points = HashSet::new();
        for row in input.lines() {
            for (cidx, c) in row.chars().enumerate() {
                if c == '#' {
                    points.insert(Point::new(cidx as i64, line as i64));
                }
                max_w = std::cmp::max(max_w, cidx);
            }
            line += 1;
        }
        Map {
            data: points,
            width: max_w as i64 + 1,
            height: line as i64,
        }
    }

    mod point {
        use num::Integer;
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

        pub struct LinePoints {
            base: Point,
            ray: Point,
            n: i64,
        }

        impl LinePoints {
            pub fn new(src: Point, dst: Point) -> Self {
                assert!(src != dst);
                let diff = dst - src;
                let gcd = diff.x.gcd(&diff.y);
                let ray = Point::new(diff.x / gcd, diff.y / gcd);

                Self {
                    base: dst,
                    ray,
                    n: 0,
                }
            }
        }

        impl Iterator for LinePoints {
            type Item = Point;
            fn next(&mut self) -> Option<Self::Item> {
                self.n += 1;
                Some(self.base + self.ray * self.n)
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;

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

            #[test]
            fn point_iter() {
                let origin = Point::new(3, 4);
                let target = Point::new(4, 6);
                let points = LinePoints::new(origin, target).take(5).collect::<Vec<_>>();
                assert_eq!(
                    points,
                    vec![
                        Point::new(5, 8),
                        Point::new(6, 10),
                        Point::new(7, 12),
                        Point::new(8, 14),
                        Point::new(9, 16),
                    ]
                )
            }

            #[test]
            fn point_iter_reduce() {
                let origin = Point::new(3, 4);
                let target = Point::new(5, 6); // 2x2 should be reduced to 1x1
                let points = LinePoints::new(origin, target).take(5).collect::<Vec<_>>();
                assert_eq!(
                    points,
                    vec![
                        Point::new(6, 7),
                        Point::new(7, 8),
                        Point::new(8, 9),
                        Point::new(9, 10),
                        Point::new(10, 11),
                    ]
                )
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Map {
        pub data: HashSet<Point>,
        width: i64,
        height: i64,
    }

    impl Map {
        #[inline]
        fn in_map_bounds(&self, p: Point) -> bool {
            trace!(slog_scope::logger(), "{:?}", p);
            (p.x >= 0 && p.x < self.width) && (p.y >= 0 && p.y < self.height)
        }

        #[inline]
        fn is_object(&self, p: Point) -> bool {
            self.data.contains(&p)
        }

        pub fn visible_from(&self, o: Point) -> HashSet<Point> {
            let mut hidden = HashSet::new();
            hidden.insert(o);

            for p in &self.data {
                if o == *p || hidden.contains(p) {
                    continue;
                }
                hidden.extend(
                    LinePoints::new(o, *p)
                        .take_while(|t| self.in_map_bounds(*t))
                        .filter(|t| self.is_object(*t)),
                )
            }
            self.data.difference(&hidden).cloned().collect()
        }
    }

    impl<T: AsRef<str>> From<T> for Map {
        fn from(input: T) -> Self {
            parse_map(input.as_ref())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    fn check_map_input(input: &str, p: Point, len: usize) {
        let map = Map::from(input);
        let max_point = find_max_asteroid(&map).expect("map does not have two points");
        assert_eq!(max_point.0, p);
        assert_eq!(max_point.1, len);
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
        let input = "\
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
        check_map_input(input, Point::new(11, 13), 210);
    }

    #[test]
    fn check_part1() {
        assert_eq!(part1(DAY10_INPUT).unwrap().as_str(), "267")
    }

    #[test]
    fn check_part2() {
        assert_eq!(part2(DAY10_INPUT).unwrap().as_str(), "0")
    }
}
