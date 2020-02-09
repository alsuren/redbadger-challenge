mod flatten;

use crate::flatten::ResultOfIteratorOfResult;
use anyhow::{bail, Error, Result};
use enum_display_derive::Display;
use itertools::Itertools;
use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::io::{self, BufRead};

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

#[derive(Debug)]
enum Rotation {
    L,
    R,
}

#[derive(Debug)]
enum Instruction {
    F,
    Turn(Rotation),
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
struct Coords {
    x: i32,
    y: i32,
}

impl<'a> Coords {
    fn try_from_iterator(split: &mut impl Iterator<Item = &'a str>) -> Result<Self, anyhow::Error> {
        let x = split
            .next()
            .ok_or_else(|| Error::msg("missing x coordinate"))?
            .parse::<i32>()?;
        let y = split
            .next()
            .ok_or_else(|| Error::msg("missing y coordinate"))?
            .parse::<i32>()?;
        Ok(Coords { x, y })
    }
}

// The grid looks like this:
//     y (North)
//     ^
//     |
//     +-------> x (East)
// If a robot falls off the edge then we add {x, y} to scents.
#[derive(Debug)]
struct Grid {
    max: Coords,
    scents: HashSet<Coords>,
}

impl TryFrom<String> for Grid {
    type Error = anyhow::Error;

    fn try_from(size_line: String) -> Result<Self, Self::Error> {
        let mut split = size_line.split(' ');
        let max = Coords::try_from_iterator(&mut split)?;
        if let Some(_) = split.next() {
            bail!("grid line has too many fields");
        }
        Ok(Grid {
            max,
            scents: Default::default(),
        })
    }
}

impl Grid {
    fn has_scent(&self, robot: &Robot) -> bool {
        self.scents.contains(&robot.coords)
    }

    fn apply_scent(&mut self, robot: &Robot) {
        self.scents.insert(robot.coords.clone());
    }
}

#[derive(Clone, Debug)]
struct Robot {
    coords: Coords,
    bearing: Bearing,
}

impl TryFrom<String> for Robot {
    type Error = anyhow::Error;

    fn try_from(position_line: String) -> Result<Self, Self::Error> {
        let mut split = position_line.split(' ');
        let coords = Coords::try_from_iterator(&mut split)?;
        let bearing = split
            .next()
            .ok_or_else(|| Error::msg("missing bearing"))?
            .try_into()?;
        if let Some(_) = split.next() {
            bail!("grid line has too many fields");
        }
        Ok(Robot { coords, bearing })
    }
}

impl Robot {
    fn is_out_of_bounds(&self, grid: &Grid) -> bool {
        let Robot { coords, .. } = self;
        let Grid { max, .. } = grid;
        0 > coords.x || coords.x > max.x || 0 > coords.y || coords.y > max.y
    }

    fn go_forwards_unchecked(mut self) -> Self {
        use Bearing::*;
        match self.bearing {
            N => {
                self.coords.y += 1;
            }
            E => {
                self.coords.x += 1;
            }
            S => {
                self.coords.y -= 1;
            }
            W => {
                self.coords.x -= 1;
            }
        }
        self
    }

    fn try_going_forwards(self, grid: &Grid) -> std::result::Result<Robot, Robot> {
        let next = self.clone().go_forwards_unchecked();

        if next.is_out_of_bounds(grid) {
            if grid.has_scent(&self) {
                Ok(self)
            } else {
                Err(self)
            }
        } else {
            Ok(next)
        }
    }

    /// Returns either the position that the robot ended up at or the
    /// position where it was before it fell off the board.
    fn try_next_instruction(
        self,
        grid: &Grid,
        instruction: &Instruction,
    ) -> std::result::Result<Robot, Robot> {
        match instruction {
            Instruction::Turn(t) => Ok(Robot {
                bearing: self.bearing.rotate(t),
                ..self
            }),
            Instruction::F => self.try_going_forwards(grid),
        }
    }

    /// Returns either the position that the robot ended up at or the
    /// position where it was before it fell off the board.
    fn try_all_instructions(
        self,
        grid: &Grid,
        instructions: &[Instruction],
    ) -> std::result::Result<Robot, Robot> {
        let mut current = self;
        for instruction in instructions {
            current = current.try_next_instruction(grid, instruction)?;
        }
        Ok(current)
    }
}

/// Expects Strings to be trimmed lines, with empty lines omited.
/// Yields strings representing Robot end positions, ready to be
/// passed to stdout, or Error if an input line is invalid.
fn drive_robots(
    mut lines: impl Iterator<Item = Result<String>>,
) -> Result<impl Iterator<Item = Result<String>>> {
    let mut grid = lines
        .next()
        .ok_or(Error::msg("input must not be empty"))??
        .try_into()?;

    let output = lines.tuples().map(
        move |(position_line, instruction_line): (Result<String>, Result<String>)| {
            let start: Robot = position_line?.try_into()?;
            let instructions = instruction_line?
                .chars()
                .map(|c| c.try_into())
                .collect::<Result<Vec<Instruction>>>()?;

            let result = start.try_all_instructions(&grid, &instructions);
            match result {
                Ok(alive) => Ok(format!(
                    "{} {} {}",
                    alive.coords.x, alive.coords.y, alive.bearing
                )),
                Err(dead) => {
                    grid.apply_scent(&dead);
                    Ok(format!(
                        "{} {} {} LOST",
                        dead.coords.x, dead.coords.y, dead.bearing
                    ))
                }
            }
        },
    );
    return Ok(output);
}

fn no_empty_lines(l: &Result<String>) -> bool {
    match l {
        Ok(l) => !l.is_empty(),
        Err(_) => true,
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let locked = stdin.lock();
    // It's a bit annoying that .lines() allocates a new buffer for
    // each line, but I think it will be easier to refactor this
    // (once Rust is able to express the lifetime of a re-used buffer)
    // than it would be to use something other than Iterator to drive
    // the data flow.
    let lines = locked.lines();

    // Convert errors to anyhow::Error, and remove empty lines.
    let lines = lines
        .map(|l| Ok(l?.trim().to_owned()))
        .filter(no_empty_lines);

    drive_robots(lines)
        .flatten_to_iterator()
        .try_for_each(|result| Ok::<_, Error>(println!("{}", result?)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(input: &str) -> impl Iterator<Item = Result<String>> + '_ {
        input
            .lines()
            .map(|l| Ok(l.trim().to_owned()))
            .filter(no_empty_lines)
    }

    fn join(input: impl Iterator<Item = Result<String>>) -> Result<String> {
        Ok(input
            .collect::<Result<Vec<String>>>()?
            .join("\n")
            .to_owned())
    }

    fn format(input: &str) -> Result<String> {
        join(split(input).filter(no_empty_lines))
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
        let output = join(drive_robots(split(input)).flatten_to_iterator())?;

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
            .flatten_to_iterator()
            .next()
            .ok_or_else(|| Error::msg("should output something"))?;

        assert_eq!(output.unwrap_err().to_string(), "input must not be empty");
        Ok(())
    }
}
