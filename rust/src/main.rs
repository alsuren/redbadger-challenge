#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate enum_derive;

use itertools::Itertools;
use std::collections::HashSet;
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

// We use a position that is off the edge of the board to denote a dead robot
// rather than storing it as an extra flag here (in the spirit of making invalid
// states unrepresentable). There is a little code in drive_robots() to nudge the
// robot back on the map for reporting and scent marking.
#[derive(Clone, Copy, Debug)]
struct Position {
    x: i32,
    y: i32,
    bearing: Bearing,
}

// If you want to add bearings then typescript will prompt you to fix
// move_unchecked() (see note about Instruction) but you will also have to
// fix the rotate functions (and typescript won't tell you because I was being
// cheeky when writing it)
custom_derive! {
    #[derive(Clone, Copy, Debug, EnumFromStr, EnumDisplay)]
    enum Bearing {
        N,
        E,
        S,
        W,
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

// If a function takes grid, position and/or instruction then they should always
// be provided in the order (grid, position, instruction).

fn drive_robots(mut lines: impl Iterator<Item = String>) -> impl Iterator<Item = String> {
    let mut grid = make_grid(&lines.next().unwrap());

    lines
        .filter(|l| l.len() > 0)
        .tuples()
        .map(move |(position_line, instruction_line)| {
            let start = get_start_position(&position_line);
            let end = get_end_position(&grid, start, &instruction_line);
            if is_out_of_bounds(&grid, end) {
                // robots stay where they are as soon as they fall off the world,
                // so if we back the robot up then we will have the position where
                // it should leave its scent and be reported
                let last = move_unchecked(end, -1);
                apply_scent(&mut grid, last);
                format!("{} {} {} LOST", last.x, last.y, last.bearing)
            } else {
                format!("{} {} {}", end.x, end.y, end.bearing)
            }
        })
}

fn make_grid(size_line: &str) -> Grid {
    let mut split = size_line.split(' ');
    let x = split.next().map(|i| i.parse::<i32>().unwrap()).unwrap();
    let y = split.next().map(|i| i.parse::<i32>().unwrap()).unwrap();
    assert!(split.next().is_none(), "grid line has too many fields");
    return Grid {
        x_max: x,
        y_max: y,
        scents: Default::default(),
    };
}

fn get_start_position(position: &str) -> Position {
    let mut split = position.split(' ');
    let x = split.next().map(|i| i.parse::<i32>().unwrap()).unwrap();
    let y = split.next().map(|i| i.parse::<i32>().unwrap()).unwrap();
    let bearing = split.next().map(|i| i.parse::<Bearing>().unwrap()).unwrap();
    assert!(split.next().is_none(), "start position has too many fields");
    return Position {
        x: x,
        y: y,
        bearing: bearing as Bearing,
    };
}

fn get_end_position(grid: &Grid, position: Position, instructions: &str) -> Position {
    let mut current = position;
    for instruction in instructions.chars() {
        let parsed = match instruction {
            'L' => Instruction::Turn(Rotation::L),
            'R' => Instruction::Turn(Rotation::R),
            'F' => Instruction::F,
            _ => todo!("think harder about parsing"),
        };
        current = get_next_position(grid, current, parsed);
    }
    return current;
}

fn get_next_position(grid: &Grid, position: Position, instruction: Instruction) -> Position {
    // If we're already off the edge of the board then skip all instructions
    if is_out_of_bounds(grid, position) {
        return position;
    }

    match instruction {
        Instruction::Turn(Rotation::L) => {
            return Position {
                bearing: rotate_bearing_left(position.bearing),
                ..position
            };
        }
        Instruction::Turn(Rotation::R) => {
            return Position {
                bearing: rotate_bearing_right(position.bearing),
                ..position
            };
        }
        Instruction::F => {
            return go_forwards(grid, position);
        }
    }
}

fn is_out_of_bounds(grid: &Grid, position: Position) -> bool {
    let Position { x, y, .. } = position;
    let Grid { x_max, y_max, .. } = grid;
    return 0 > x || x > *x_max || 0 > y || y > *y_max;
}

fn has_scent(grid: &Grid, position: Position) -> bool {
    return grid.scents.contains(&(position.x, position.y));
}

fn apply_scent(grid: &mut Grid, position: Position) {
    grid.scents.insert((position.x, position.y));
}

fn rotate_bearing_left(bearing: Bearing) -> Bearing {
    use Bearing::*;
    match bearing {
        N => W,
        E => N,
        S => E,
        W => S,
    }
}

fn rotate_bearing_right(bearing: Bearing) -> Bearing {
    use Bearing::*;
    match bearing {
        W => N,
        N => E,
        E => S,
        S => W,
    }
}

fn go_forwards(grid: &Grid, position: Position) -> Position {
    let new_position = move_unchecked(position, 1);
    if is_out_of_bounds(grid, new_position) && has_scent(grid, position) {
        return position;
    }
    return new_position;
}

fn move_unchecked(position: Position, steps: i32) -> Position {
    use Bearing::*;
    let Position { x, y, bearing } = position;
    match bearing {
        N => {
            return Position {
                y: y + steps,
                ..position
            };
        }
        E => {
            return Position {
                x: x + steps,
                ..position
            };
        }
        S => {
            return Position {
                y: y - steps,
                ..position
            };
        }
        W => {
            return Position {
                x: x - steps,
                ..position
            };
        }
    }
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let locked = stdin.lock();
    let lines = locked.lines().map(|l| l.unwrap());

    drive_robots(lines).for_each(|result| println!("{}", result));

    Ok(())
}
