use anyhow::{Error, Result};
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
        let x = split.next().map(|i| i.parse::<i32>().unwrap()).unwrap();
        let y = split.next().map(|i| i.parse::<i32>().unwrap()).unwrap();
        assert!(split.next().is_none(), "grid line has too many fields");
        Ok(Grid {
            x_max: x,
            y_max: y,
            scents: Default::default(),
        })
    }
}
// We use a position that is off the edge of the board to denote a dead robot
// rather than storing it as an extra flag here (in the spirit of making invalid
// states unrepresentable). There is a little code in drive_robots() to nudge the
// robot back on the map for reporting and scent marking.
#[derive(Clone, Debug)]
struct Position {
    x: i32,
    y: i32,
    bearing: Bearing,
}

impl TryFrom<String> for Position {
    type Error = anyhow::Error;

    fn try_from(position_line: String) -> Result<Self, Self::Error> {
        let mut split = position_line.split(' ');
        let x = split.next().map(|i| i.parse::<i32>().unwrap()).unwrap();
        let y = split.next().map(|i| i.parse::<i32>().unwrap()).unwrap();
        let bearing = split.next().map(|i| i.try_into().unwrap()).unwrap();
        assert!(split.next().is_none(), "start position has too many fields");
        Ok(Position { x, y, bearing })
    }
}

impl Position {
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
    fn rotate(self, rotation: Rotation) -> Bearing {
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

// If a function takes grid, position and/or instruction then they should always
// be provided in the order (grid, position, instruction).

// I might collapse move_unchecked() back down here at some point,
// or move a bunch of the 2-argument functions onto Grid. I'm not
// really sure what to do with the 3-argument functions (putting
// them on Grid feels overly object-oriented).

fn is_out_of_bounds(grid: &Grid, position: &Position) -> bool {
    let Position { x, y, .. } = position;
    let Grid { x_max, y_max, .. } = grid;
    0 > *x || x > x_max || 0 > *y || y > y_max
}

fn has_scent(grid: &Grid, position: &Position) -> bool {
    grid.scents.contains(&(position.x, position.y))
}

fn apply_scent(grid: &mut Grid, position: &Position) {
    grid.scents.insert((position.x, position.y));
}

fn go_forwards(grid: &Grid, position: Position) -> Position {
    let new_position = position.clone().move_unchecked(1);
    if is_out_of_bounds(grid, &new_position) && has_scent(grid, &position) {
        position
    } else {
        new_position
    }
}

fn get_next_position(grid: &Grid, position: Position, instruction: Instruction) -> Position {
    // If we're already off the edge of the board then skip all instructions
    if is_out_of_bounds(grid, &position) {
        return position;
    }

    match instruction {
        Instruction::Turn(t) => Position {
            bearing: position.bearing.rotate(t),
            ..position
        },
        Instruction::F => go_forwards(grid, position),
    }
}

fn get_end_position(grid: &Grid, position: Position, instructions: &str) -> Position {
    let mut current = position;
    for instruction in instructions.chars() {
        let parsed_instruction = instruction.try_into().unwrap();
        current = get_next_position(grid, current, parsed_instruction);
    }
    current
}

fn is_interesting(l: &Result<String>) -> bool {
    match l {
        Ok(l) => !l.is_empty(),
        Err(_) => true,
    }
}

enum DriveRobotsResult<T>
where
    T: Iterator<Item = Result<String>> + Sized,
{
    Err(Error),
    Ok(T),
}

impl<T> Iterator for DriveRobotsResult<T>
where
    T: Iterator<Item = Result<String>> + Sized,
{
    type Item = Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DriveRobotsResult::Ok(iter) => iter.next(),
            DriveRobotsResult::Err(err) => Some(Err(std::mem::replace(
                err,
                Error::msg("Stop Iterating! It's already dead!"),
            ))),
        }
    }
}

fn drive_robots(
    lines: impl Iterator<Item = Result<String>>,
) -> impl Iterator<Item = Result<String>> {
    let mut lines = lines.filter(is_interesting);

    let mut grid = match lines.next() {
        Some(Ok(line)) => match line.try_into() {
            Ok(grid) => grid,
            Err(err) => return DriveRobotsResult::Err(err),
        },
        Some(Err(err)) => return DriveRobotsResult::Err(err),
        None => return DriveRobotsResult::Err(Error::msg("input must not be empty")),
    };

    let output = lines.tuples().map(
        move |(position_line, instruction_line): (Result<String>, Result<String>)| {
            let start = position_line?.try_into().unwrap();
            let end = get_end_position(&grid, start, &instruction_line?);
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
    return DriveRobotsResult::Ok(output);
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let locked = stdin.lock();
    // It's a bit annoying that .lines() allocates a new buffer for
    // each line, but I think it will be easier to refactor this
    // once Rust has const generics than it would be to use something
    // other than Iterator to drive the data flow.
    let lines = locked.lines().map(|l| Ok(l?));

    drive_robots(lines).for_each(|result| println!("{}", result.unwrap()));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(input: &str) -> impl Iterator<Item = Result<String>> + '_ {
        input.lines().map(|l| Ok(l.trim().to_owned()))
    }

    fn join(input: impl Iterator<Item = Result<String>>) -> String {
        input
            .collect::<Result<Vec<String>>>()
            .unwrap()
            .join("\n")
            .to_owned()
    }

    fn format(input: &str) -> String {
        join(split(input).filter(is_interesting))
    }

    #[test]
    fn example_input_produces_example_output() {
        let input = r#"
        5 3
        1 1 E
        RFRFRFRF
        3 2 N
        FRRFLLFFRRFLL
        0 3 W
        LLFFFLFLFL
        "#;
        let output = join(drive_robots(split(input)));

        let expected_output = format(
            r#"
            1 1 E
            3 3 N LOST
            2 3 S
            "#,
        );
        assert_eq!(output, expected_output);
    }
    #[test]
    fn empty_input_produces_error() {
        let input = r#""#;
        let output = drive_robots(split(input)).next();

        assert_eq!(
            output.unwrap().unwrap_err().to_string(),
            "input must not be empty"
        );
    }
}
