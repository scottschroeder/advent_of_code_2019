use crate::intcode::{run_intcode, IntCode};
use crate::util::parse_intcode;
use anyhow::{Result, Error};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt;
use crate::intcode::intcode_io::{Input, Output};
use std::fmt::Formatter;
use crate::display::{ImageNormal};

pub fn part1(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let hull = Hull::black();
    let robot = run_robot(intcode, hull)?;
    let robot = robot.robot.lock().unwrap();
    Ok(format!("{}", robot.hull.len()))
}

pub fn part2(input: &str) -> Result<String> {
    let intcode = parse_intcode(input)?;
    let hull = Hull::white();
    let robot = run_robot(intcode, hull)?;
    let robot = robot.robot.lock().unwrap();

    let img = ImageNormal::create(&robot.hull.inner);

    Ok(format!("{}", img))
}

fn run_robot(intcode: Vec<i64>, hull: Hull) -> Result<RobotController> {
    let (r_in, r_out) = robot_io(hull);
    let mut ic = IntCode::new(intcode, r_in, r_out);
    ic.run_till_end()?;
    let (_, out) = ic.emit();
    Ok(out)
}

#[derive(Debug, Clone, Copy)]
enum Heading {
    North,
    East,
    South,
    West,
}

impl Heading {
    fn rotate(self, rotation: Rotation) -> Heading {
        match (self, rotation) {
            (Heading::North, Rotation::Clockwise) => Heading::East,
            (Heading::East, Rotation::Clockwise) => Heading::South,
            (Heading::South, Rotation::Clockwise) => Heading::West,
            (Heading::West, Rotation::Clockwise) => Heading::North,
            (Heading::North, Rotation::CounterClockwise) => Heading::West,
            (Heading::East, Rotation::CounterClockwise) => Heading::North,
            (Heading::South, Rotation::CounterClockwise) => Heading::East,
            (Heading::West, Rotation::CounterClockwise) => Heading::South,
        }
    }
    fn direction(self) -> (i32, i32) {
        match self {
            Heading::North => (0, 1),
            Heading::East => (1, 0),
            Heading::South => (0, -1),
            Heading::West => (-1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Rotation {
    Clockwise,
    CounterClockwise,
}

impl From<i64> for Rotation {
    fn from(r: i64) -> Self {
        match r {
            0 => Rotation::CounterClockwise,
            1 => Rotation::Clockwise,
            _ => unreachable!("rotation unsupported: {}", r),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Color {
    White,
    Black,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "X"),
            Color::Black => write!(f, " "),
        }
    }
}

impl From<Color> for i64 {
    fn from(c: Color) -> Self {
        match c {
            Color::White => 1,
            Color::Black => 0,
        }
    }
}

impl From<i64> for Color {
    fn from(c: i64) -> Self {
        match c {
            0 => Color::Black,
            1 => Color::White,
            _ => unreachable!("color unsupported: {}", c),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RobotState {
    x: i32,
    y: i32,
    heading: Heading,
}

impl RobotState {
    fn rotate_advance(&mut self, rotation: Rotation) {
        self.heading = self.heading.rotate(rotation);
        let (dx, dy) = self.heading.direction();
        self.x += dx;
        self.y += dy;
    }
}

#[derive(Debug)]
pub(crate) struct Hull {
    inner: HashMap<(i32, i32), Color>
}

impl Hull {
    fn black() -> Hull {
        Hull {
            inner: HashMap::default(),
        }
    }
    fn white() -> Hull {
        let mut h = Hull::black();
        h.write(0, 0, Color::White);
        h
    }
    fn read(&self, x: i32, y: i32) -> Color {
        let coord = (x, y);
        *self.inner.get(&coord).unwrap_or(&Color::Black)
    }
    fn write(&mut self, x: i32, y: i32, c: Color) {
        let coord = (x, y);
        self.inner.insert(coord, c);
    }
    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Debug)]
struct Robot {
    robot: RobotState,
    hull: Hull,
}

impl Robot {
    fn new(hull: Hull) -> Self {
        Robot {
            robot: RobotState {
                x: 0,
                y: 0,
                heading: Heading::North,

            },
            hull,
        }
    }
}

fn robot_io(hull: Hull) -> (RobotCamera, RobotController) {
    let r = Arc::new(Mutex::new(Robot::new(hull)));
    let input = RobotCamera {
        robot: r.clone(),
    };
    let output = RobotController {
        robot: r,
        is_rotate: false,
    };
    (input, output)
}


#[derive(Debug)]
struct RobotCamera {
    robot: Arc<Mutex<Robot>>
}

impl Input for RobotCamera {
    fn input(&mut self) -> Result<i64, Error> {
        let r = self.robot.lock().unwrap();
        let c = r.hull.read(
            r.robot.x,
            r.robot.y,
        );
        Ok(c.into())
    }
}

#[derive(Debug)]
struct RobotController {
    robot: Arc<Mutex<Robot>>,
    is_rotate: bool,
}

impl Output for RobotController {
    fn output(&mut self, out: i64) -> Result<(), Error> {
        let mut robot = self.robot.lock().unwrap();
        if self.is_rotate {
            let r = Rotation::from(out);
            robot.robot.rotate_advance(r);
        } else {
            let c = Color::from(out);
            let x = robot.robot.x;
            let y = robot.robot.y;
            robot.hull.write(x, y, c);
        }
        self.is_rotate = !self.is_rotate;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn check_part1() {
        assert_eq!(part1(DAY11_INPUT).unwrap().as_str(), "2093")
    }

    #[test]
    fn check_part2() {
        assert_eq!(part2(DAY11_INPUT).unwrap().trim(), DAY11_PART2_OUTPUT.trim())
    }
}
