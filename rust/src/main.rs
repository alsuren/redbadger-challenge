use anyhow::{bail, Error, Result};
use enum_display_derive::Display;
use itertools::Itertools;
use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::io::{self, BufRead};

// The grid looks like this:
//     y (North)
//     ^
//     |
//     +-------> x (East)
// If a robot falls off the edge then we set grid[x,y] to true
struct Grid {
    x_max: i32,
    y_max: i32,
    scents: HashSet<(i32, i32)>,
}

impl TryFrom<String> for Grid {
    type Error = anyhow::Error;

    fn try_from(size_line: String) -> Result<Self, Self::Error> {
        let mut split = size_line.split(' ');
        let x = split
            .next()
            .ok_or_else(|| Error::msg("missing x coordinate"))?
            .parse::<i32>()?;
        let y = split
            .next()
            .ok_or_else(|| Error::msg("missing y coordinate"))?
            .parse::<i32>()?;
        if let Some(_) = split.next() {
            bail!("grid line has too many fields");
        }
        Ok(Grid {
            x_max: x,
            y_max: y,
            scents: Default::default(),
        })
    }
}

// We use a robot that is off the edge of the board to denote a dead robot
// rather than storing it as an extra flag here (in the spirit of making invalid
// states unrepresentable). There is a little code in drive_robots() to nudge the
// robot back on the map for reporting and scent marking.
#[derive(Clone, Debug)]
struct Robot {
    x: i32,
    y: i32,
    bearing: Bearing,
}

impl TryFrom<String> for Robot {
    type Error = anyhow::Error;

    fn try_from(position_line: String) -> Result<Self, Self::Error> {
        let mut split = position_line.split(' ');
        let x = split
            .next()
            .ok_or_else(|| Error::msg("missing x coordinate"))?
            .parse::<i32>()?;
        let y = split
            .next()
            .ok_or_else(|| Error::msg("missing y coordinate"))?
            .parse::<i32>()?;
        let bearing = split
            .next()
            .ok_or_else(|| Error::msg("missing bearing"))?
            .try_into()?;
        if let Some(_) = split.next() {
            bail!("grid line has too many fields");
        }
        Ok(Robot { x, y, bearing })
    }
}

impl Robot {
    fn move_unchecked(mut self, steps: i32) -> Self {
        use Bearing::*;
        match self.bearing {
            N => {
                self.y += steps;
            }
            E => {
                self.x += steps;
            }
            S => {
                self.y -= steps;
            }
            W => {
                self.x -= steps;
            }
        }
        self
    }
}

#[derive(Clone, Copy, Debug, Display)]
enum Bearing {
    N,
    E,
    S,
    W,
}

impl TryFrom<&str> for Bearing {
    type Error = anyhow::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        use Bearing::*;
        match input {
            "N" => Ok(N),
            "E" => Ok(E),
            "S" => Ok(S),
            "W" => Ok(W),
            _ => Err(Error::msg("Bearing must be one of N, E, S, or W")),
        }
    }
}

impl Bearing {
    fn rotate(self, rotation: &Rotation) -> Bearing {
        use Bearing::*;
        use Rotation::*;

        match (self, rotation) {
            (N, L) => W,
            (E, L) => N,
            (S, L) => E,
            (W, L) => S,
            (W, R) => N,
            (N, R) => E,
            (E, R) => S,
            (S, R) => W,
        }
    }
}
// The current set of valid instructions. When adding an instruction, add it
// here, and then typescript will complain about the case in get_next_position()
// being non-exhaustive:
//   Function lacks ending return statement and return type does not include 'undefined'.
// so you will be forced to fix it there too.
#[derive(Debug)]
enum Instruction {
    F,
    Turn(Rotation),
}
#[derive(Debug)]
enum Rotation {
    L,
    R,
}

impl TryFrom<char> for Instruction {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Instruction::*;
        use Rotation::*;
        match value {
            'F' => Ok(F),
            'L' => Ok(Turn(L)),
            'R' => Ok(Turn(R)),
            _ => Err(Error::msg("instruction must be F, L, or R")),
        }
    }
}

// If a function takes grid, robot and/or instruction then they should always
// be provided in the order (grid, robot, instruction).

// I might collapse move_unchecked() back down here at some point,
// or move a bunch of the 2-argument functions onto Grid. I'm not
// really sure what to do with the 3-argument functions (putting
// them on Grid feels overly object-oriented).

fn is_out_of_bounds(grid: &Grid, robot: &Robot) -> bool {
    let Robot { x, y, .. } = robot;
    let Grid { x_max, y_max, .. } = grid;
    0 > *x || x > x_max || 0 > *y || y > y_max
}

fn has_scent(grid: &Grid, robot: &Robot) -> bool {
    grid.scents.contains(&(robot.x, robot.y))
}

fn apply_scent(grid: &mut Grid, robot: &Robot) {
    grid.scents.insert((robot.x, robot.y));
}

fn go_forwards(grid: &Grid, robot: Robot) -> Robot {
    let new_position = robot.clone().move_unchecked(1);
    if is_out_of_bounds(grid, &new_position) && has_scent(grid, &robot) {
        robot
    } else {
        new_position
    }
}

fn get_next_position(grid: &Grid, robot: Robot, instruction: &Instruction) -> Robot {
    // If we're already off the edge of the board then skip all instructions
    if is_out_of_bounds(grid, &robot) {
        return robot;
    }

    match instruction {
        Instruction::Turn(t) => Robot {
            bearing: robot.bearing.rotate(t),
            ..robot
        },
        Instruction::F => go_forwards(grid, robot),
    }
}

fn get_end_position(grid: &Grid, robot: Robot, instructions: &[Instruction]) -> Robot {
    let mut current = robot;
    for instruction in instructions {
        current = get_next_position(grid, current, instruction);
    }
    current
}

fn is_interesting(l: &Result<String>) -> bool {
    match l {
        Ok(l) => !l.is_empty(),
        Err(_) => true,
    }
}

enum FlattenedIteratorOfResult<T>
where
    T: Iterator<Item = Result<String>> + Sized,
{
    Err(Error),
    Ok(T),
}

impl<T> Iterator for FlattenedIteratorOfResult<T>
where
    T: Iterator<Item = Result<String>> + Sized,
{
    type Item = Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            FlattenedIteratorOfResult::Ok(iter) => iter.next(),
            FlattenedIteratorOfResult::Err(err) => Some(Err(std::mem::replace(
                err,
                Error::msg("Stop Iterating! It's already dead!"),
            ))),
        }
    }
}

trait FlattenableResultOfIteratorOfResult<T>
where
    T: Iterator<Item = Result<String>>,
{
    fn flatten(self) -> FlattenedIteratorOfResult<T>;
}

impl<T> FlattenableResultOfIteratorOfResult<T> for Result<T>
where
    T: Iterator<Item = Result<String>>,
{
    fn flatten(self) -> FlattenedIteratorOfResult<T> {
        match self {
            Ok(iter) => FlattenedIteratorOfResult::Ok(iter),
            Err(err) => FlattenedIteratorOfResult::Err(err),
        }
    }
}

fn drive_robots(
    lines: impl Iterator<Item = Result<String>>,
) -> Result<impl Iterator<Item = Result<String>>> {
    let mut lines = lines.filter(is_interesting);

    let mut grid = lines
        .next()
        .ok_or(Error::msg("input must not be empty"))??
        .try_into()?;

    let output = lines.tuples().map(
        move |(position_line, instruction_line): (Result<String>, Result<String>)| {
            let start = position_line?.try_into()?;
            let instructions = instruction_line?
                .chars()
                .map(|c| c.try_into())
                .collect::<Result<Vec<_>>>()?;
            let end = get_end_position(&grid, start, &instructions);
            if is_out_of_bounds(&grid, &end) {
                // robots stay where they are as soon as they fall off the world,
                // so if we back the robot up then we will have the position where
                // it should leave its scent and be reported
                let last = end.move_unchecked(-1);
                apply_scent(&mut grid, &last);
                Ok(format!("{} {} {} LOST", last.x, last.y, last.bearing))
            } else {
                Ok(format!("{} {} {}", end.x, end.y, end.bearing))
            }
        },
    );
    return Ok(output);
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let locked = stdin.lock();
    // It's a bit annoying that .lines() allocates a new buffer for
    // each line, but I think it will be easier to refactor this
    // once Rust has const generics than it would be to use something
    // other than Iterator to drive the data flow.
    let lines = locked.lines().map(|l| Ok(l?));

    drive_robots(lines)
        .flatten()
        .try_for_each(|result| Ok::<_, Error>(println!("{}", result?)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(input: &str) -> impl Iterator<Item = Result<String>> + '_ {
        input.lines().map(|l| Ok(l.trim().to_owned()))
    }

    fn join(input: impl Iterator<Item = Result<String>>) -> Result<String> {
        Ok(input
            .collect::<Result<Vec<String>>>()?
            .join("\n")
            .to_owned())
    }

    fn format(input: &str) -> Result<String> {
        join(split(input).filter(is_interesting))
    }

    #[test]
    fn example_input_produces_example_output() -> Result<()> {
        let input = r#"
        5 3
        1 1 E
        RFRFRFRF
        3 2 N
        FRRFLLFFRRFLL
        0 3 W
        LLFFFLFLFL
        "#;
        let output = join(drive_robots(split(input)).flatten())?;

        let expected_output = format(
            r#"
            1 1 E
            3 3 N LOST
            2 3 S
            "#,
        )?;
        assert_eq!(output, expected_output);
        Ok(())
    }

    #[test]
    fn empty_input_produces_error() -> Result<()> {
        let input = r#""#;
        let output = drive_robots(split(input))
            .flatten()
            .next()
            .ok_or_else(|| Error::msg("should output something"))?;

        assert_eq!(output.unwrap_err().to_string(), "input must not be empty");
        Ok(())
    }
}
