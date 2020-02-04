// The grid looks like this:
//     y (North)
//     ^
//     |
//     +-------> x (East)
// If a robot falls off the edge then we set grid[x][y] to true
type Grid = boolean[][];

// We use a position that is off the edge of the board to denote a dead robot
// rather than storing it as an extra flag here (in the spirit of making invalid
// states unrepresentable). There is a little code in driveRobots() to nudge the
// robot back on the map for reporting and scent marking.
export type Position = {
  x: number;
  y: number;
  bearing: Bearing;
};

// If you want to add bearings then typescript will prompt you to fix
// moveUnchecked() (see note about Instruction) but you will also have to
// fix the rotate functions (and typescript won't tell you because I was being
// cheeky when writing it)
export type Bearing = 'N' | 'E' | 'S' | 'W';

// The current set of valid instructions. When adding an instruction, add it
// here, and then typescript will complain about the case in getNextPosition()
// being non-exhaustive:
//   Function lacks ending return statement and return type does not include 'undefined'.
// so you will be forced to fix it there too.
type Instruction = 'L' | 'R' | 'F';

// If a function takes grid, position and/or instruction then they should always
// be provided in the order (grid, position, instruction).

export function driveRobots(input: string): string {
  const [sizeLine, rest] = splitOnce(input, '\n');
  let grid = makeGrid(sizeLine);
  const response = [];

  for (const script of rest.split('\n\n')) {
    const [pos, instructions] = script.split('\n');
    const startPosition = getStartPosition(pos);
    const end = getEndPosition(grid, startPosition, instructions);
    if (isOutOfBounds(grid, end)) {
      // robots stay where they are as soon as they fall off the world,
      // so if we back the robot up then we will have the position where
      // it should leave its scent and be reported
      const last = moveUnchecked(end, -1);
      applyScent(grid, last);
      response.push(`${last.x} ${last.y} ${last.bearing} LOST`);
    } else {
      response.push(`${end.x} ${end.y} ${end.bearing}`);
    }
  }
  return response.join('\n');
}

function splitOnce(s: string, on: string): [string, string] {
  const [first, ...rest] = s.split(on);
  return [first, rest.join(on)];
}

export function makeGrid(sizeLine: string): Grid {
  const [x, y] = sizeLine.split(' ').map((i) => parseInt(i, 10));
  return new Array(x + 1).fill(null).map(() => new Array(y + 1).fill(false));
}

export function getStartPosition(position: string): Position {
  const [x, y, bearing] = position.split(' ');
  return {x: parseInt(x, 10), y: parseInt(y, 10), bearing: bearing as Bearing};
}

export function getEndPosition(
  grid: Grid,
  position: Position,
  instructions: string,
): Position {
  for (const instruction of instructions) {
    position = getNextPosition(grid, position, instruction as Instruction);
  }
  return position;
}

export function getNextPosition(
  grid: Grid,
  position: Position,
  instruction: Instruction,
): Position {
  // If we're already off the edge of the board then skip all instructions
  if (isOutOfBounds(grid, position)) {
    return position;
  }

  switch (instruction) {
    case 'L': {
      return {...position, bearing: rotateBearingLeft(position.bearing)};
    }
    case 'R': {
      return {...position, bearing: rotateBearingRight(position.bearing)};
    }
    case 'F': {
      return goForwards(grid, position);
    }
  }
}

export function isOutOfBounds(grid: Grid, {x, y}: Position): boolean {
  return typeof grid[x] == 'undefined' || typeof grid[x][y] == 'undefined';
}

export function hasScent(grid: Grid, {x, y}: Position): boolean {
  return typeof grid[x] != 'undefined' && Boolean(grid[x][y]);
}

export function applyScent(grid: Grid, {x, y}: Position) {
  grid[x][y] = true;
}

export function rotateBearingLeft(bearing: Bearing): Bearing {
  return 'WNES'['NESW'.indexOf(bearing)] as Bearing;
}

export function rotateBearingRight(bearing: Bearing): Bearing {
  return 'ESWN'['NESW'.indexOf(bearing)] as Bearing;
}

export function goForwards(grid: Grid, position: Position): Position {
  const newPosition = moveUnchecked(position, 1);
  if (isOutOfBounds(grid, newPosition) && hasScent(grid, position)) {
    return position;
  }
  return newPosition;
}

export function moveUnchecked(position: Position, steps: number): Position {
  let {x, y, bearing} = position;
  switch (bearing) {
    case 'N': {
      return {...position, y: y + steps};
    }
    case 'E': {
      return {...position, x: x + steps};
    }
    case 'S': {
      return {...position, y: y - steps};
    }
    case 'W': {
      return {...position, x: x - steps};
    }
  }
}
