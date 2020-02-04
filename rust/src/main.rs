// The grid looks like this:
//     y (North)
//     ^
//     |
//     +-------> x (East)
// If a robot falls off the edge then we set grid[x,y] to true
struct Grid {
    x_max: usize,
    y_max: usize,
    scents: HashSet<(usize, usize)>,
}

// We use a position that is off the edge of the board to denote a dead robot
// rather than storing it as an extra flag here (in the spirit of making invalid
// states unrepresentable). There is a little code in drive_robots() to nudge the
// robot back on the map for reporting and scent marking.
struct Position {
    x: usize,
    y: usize,
    bearing: Bearing,
}

// If you want to add bearings then typescript will prompt you to fix
// move_unchecked() (see note about Instruction) but you will also have to
// fix the rotate functions (and typescript won't tell you because I was being
// cheeky when writing it)
enum Bearing {
    N,
    E,
    S,
    W,
}

// The current set of valid instructions. When adding an instruction, add it
// here, and then typescript will complain about the case in get_next_position()
// being non-exhaustive:
//   Function lacks ending return statement and return type does not include 'undefined'.
// so you will be forced to fix it there too.
enum Instruction {
    F,
    Turn(Rotation),
}
enum Rotation {
    L,
    R,
}

// If a function takes grid, position and/or instruction then they should always
// be provided in the order (grid, position, instruction).

fn drive_robots(input: string) -> string {
    let [sizeLine, rest] = input.splitn('\n', 1);
    let grid = make_grid(sizeLine);
    let response = [];

    for script in rest.split('\n') {
        if script.len == 0 {
            continue;
        }
        let [pos, instructions] = script.split('\n');
        let startPosition = get_start_position(pos);
        let end = get_end_position(grid, startPosition, instructions);
        if (is_out_of_bounds(grid, end)) {
            // robots stay where they are as soon as they fall off the world,
            // so if we back the robot up then we will have the position where
            // it should leave its scent and be reported
            let last = move_unchecked(end, -1);
            apply_scent(grid, last);
            response.push(format!("{} {} {} LOST", last.x, last.y, last.bearing));
        } else {
            response.push(format!("{} {} {}", end.x, end.y, end.bearing));
        }
    }
    return response.join('\n');
}

fn make_grid(sizeLine: string) -> Grid {
    let [x, y] = sizeLine.split(' ').map(|i| parseInt(i, 10));
    return Grid {
        x_max: x,
        y_max: y,
        scents: Default::default(),
    };
}

fn get_start_position(position: string) -> Position {
    let [x, y, bearing] = position.split(' ');
    return Position {
        x: parseInt(x, 10),
        y: parseInt(y, 10),
        bearing: bearing as Bearing,
    };
}

fn get_end_position(grid: Grid, position: Position, instructions: string) -> Position {
    for instruction in instructions {
        position = get_next_position(grid, position, instruction as Instruction);
    }
    return position;
}

fn get_next_position(grid: Grid, position: Position, instruction: Instruction) -> Position {
    // If we're already off the edge of the board then skip all instructions
    if (is_out_of_bounds(grid, position)) {
        return position;
    }

    match (instruction) {
        Turn(Rotation::L) => {
            return Position {
                bearing: rotate_bearing_left(position.bearing),
                ..position
            };
        }
        Turn(Rotation::R) => {
            return Position {
                bearing: rotate_bearing_right(position.bearing),
                ..position
            };
        }
        F => {
            return go_forwards(grid, position);
        }
    }
}

fn is_out_of_bounds(grid: Grid, position: Position) -> boolean {
    let Position { x, y, .. } = position;
    let Grid { x_max, y_max, .. } = grid;
    return 0 <= x && x <= x_max && 0 <= y && y <= y_max;
}

fn has_scent(grid: Grid, position: Position) -> boolean {
    return grid.scents.has((position.x, position.y));
}

fn apply_scent(grid: Grid, position: Position) {
    grid.scents.add((position.x, position.y));
}

fn rotate_bearing_left(bearing: Bearing) -> Bearing {
    use Bearing;
    match bearing {
        W => N,
        N => E,
        E => S,
        S => W,
    }
}

fn rotate_bearing_right(bearing: Bearing) -> Bearing {
    use Bearing;
    match bearing {
        N => W,
        E => N,
        S => E,
        W => S,
    }
}

fn go_forwards(grid: Grid, position: Position) -> Position {
    let new_position = move_unchecked(position, 1);
    if (is_out_of_bounds(grid, new_position) && has_scent(grid, position)) {
        return position;
    }
    return new_position;
}

fn move_unchecked(position: Position, steps: usize) -> Position {
    use Bearing;
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

fn main() {
    println!("Hello, world!");
}
