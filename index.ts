type Position = {
  x: number;
  y: number;
  bearing: Bearing;
};

export type Bearing = 'N' | 'E' | 'S' | 'W';

type Instruction = 'L' | 'R' | 'F';

export function driveRobots(input: string): string {
  const [sizeLine, rest] = input.split('\n', 1);
  let grid = makeGrid(sizeLine);

  for (const script of rest.split('\n\n')) {
    const [pos, instructions] = script.split('\n');
    const startPosition = getStartPosition(pos);
    const end = getEndPosition(grid, startPosition, instructions);
    // TODO: record scent at end poistion if dead, and describe
    // end position in output.
  }
  return input;
}

export function makeGrid(sizeLine: string): boolean[][] {
  const [x, y] = sizeLine.split(' ').map((i) => parseInt(i, 10));
  return new Array(x).fill(null).map(() => new Array<boolean>(y).fill(false));
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
  switch (instruction) {
    case 'L': {
      return {...position, bearing: rotateLeft(position.bearing)};
    }
    case 'R': {
      return {...position, bearing: rotateRight(position.bearing)};
    }
    case 'F': {
      // TODO
      return position;
    }
  }
}

export function rotateLeft(bearing: Bearing): Bearing {
  return 'WNES'['NESW'.indexOf(bearing)] as Bearing;
}

export function rotateRight(bearing: Bearing): Bearing {
  return 'ESWN'['NESW'.indexOf(bearing)] as Bearing;
}
