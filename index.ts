// The grid looks like this:
//     y (North)
//     ^
//     |
// ----+-------> x (East)
//     |
//     |
export type Position = {
  x: number;
  y: number;
  bearing: Bearing;
};

export type Bearing = 'N' | 'E' | 'S' | 'W';

type Instruction = 'L' | 'R' | 'F';

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

export function makeGrid(sizeLine: string): boolean[][] {
  const [x, y] = sizeLine.split(' ').map((i) => parseInt(i, 10));
  return new Array(x + 1)
    .fill(null)
    .map(() => new Array<boolean>(y + 1).fill(false));
}

export function getStartPosition(position: string): Position {
  const [x, y, bearing] = position.split(' ');
  return {x: parseInt(x, 10), y: parseInt(y, 10), bearing: bearing as Bearing};
}

export function getEndPosition(
  grid: boolean[][],
  position: Position,
  instructions: string,
): Position {
  for (const instruction of instructions) {
    position = getNextPosition(grid, position, instruction as Instruction);
  }
  return position;
}

export function getNextPosition(
  grid: boolean[][],
  position: Position,
  instruction: Instruction,
): Position {
  // If we're already off the edge of the board then skip
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

export function isOutOfBounds(grid: boolean[][], {x, y}: Position): boolean {
  return typeof grid[x] == 'undefined' || typeof grid[x][y] == 'undefined';
}

export function hasScent(grid: boolean[][], {x, y}: Position): boolean {
  return typeof grid[x] != 'undefined' && Boolean(grid[x][y]);
}

export function applyScent(grid: boolean[][], {x, y}: Position) {
  grid[x][y] = true;
}

export function rotateBearingLeft(bearing: Bearing): Bearing {
  return 'WNES'['NESW'.indexOf(bearing)] as Bearing;
}

export function rotateBearingRight(bearing: Bearing): Bearing {
  return 'ESWN'['NESW'.indexOf(bearing)] as Bearing;
}

export function goForwards(grid: boolean[][], position: Position): Position {
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
