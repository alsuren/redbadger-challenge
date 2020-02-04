import {
  driveRobots,
  makeGrid,
  getStartPosition,
  rotateBearingLeft,
  rotateBearingRight,
  Bearing,
  Position,
  isOutOfBounds,
  moveUnchecked,
  goForwards,
} from '.';

const SAMPLE_INPUT = `
5 3
1 1 E
RFRFRFRF

3 2 N
FRRFLLFFRRFLL

0 3 W
LLFFFLFLFL
`.trim();

const SAMPLE_OUTPUT = `
1 1 E
3 3 N LOST
2 3 S
`.trim();

describe('driveRobots', () => {
  it('returns the expected output', () => {
    expect(driveRobots(SAMPLE_INPUT)).toEqual(SAMPLE_OUTPUT);
  });
});

describe('makeGrid', () => {
  it('returns an array of the correct shape', () => {
    expect(makeGrid('10 20').length).toEqual(11);
    expect(makeGrid('10 20')[0].length).toEqual(21);
  });
});

describe('getStartPosition', () => {
  it('returns the correct position', () => {
    expect(getStartPosition('10 20 W')).toEqual({x: 10, y: 20, bearing: 'W'});
  });
});

describe('isOutOfBounds', () => {
  it('returns false if on the map', () => {
    expect(isOutOfBounds(makeGrid('5 5'), {x: 1, y: 1} as Position)).toEqual(
      false,
    );
  });
  it('returns true if x too big', () => {
    expect(isOutOfBounds(makeGrid('5 5'), {x: 10, y: 1} as Position)).toEqual(
      true,
    );
  });
  it('returns true if y too big', () => {
    expect(isOutOfBounds(makeGrid('5 5'), {x: 1, y: 10} as Position)).toEqual(
      true,
    );
  });
  it('returns true if x too small', () => {
    expect(isOutOfBounds(makeGrid('5 5'), {x: -1, y: 1} as Position)).toEqual(
      true,
    );
  });
  it('returns true if y too small', () => {
    expect(isOutOfBounds(makeGrid('5 5'), {x: 1, y: -1} as Position)).toEqual(
      true,
    );
  });
});

describe('rotateBearingLeft', () => {
  it.each(['N', 'E', 'S', 'W'] as Bearing[])(
    'cancels if applied 4 times from %s',
    (bearing) => {
      expect(
        rotateBearingLeft(
          rotateBearingLeft(rotateBearingLeft(rotateBearingLeft(bearing))),
        ),
      ).toEqual(bearing);
    },
  );

  it('can turn left from North', () => {
    expect(rotateBearingLeft('N')).toEqual('W');
  });
});

describe('rotateBearingRight', () => {
  it('can turn right from North', () => {
    expect(rotateBearingRight('N')).toEqual('E');
  });
  it.each(['N', 'E', 'S', 'W'] as Bearing[])(
    'cancels if applied 4 times from %s',
    (bearing) => {
      expect(
        rotateBearingRight(
          rotateBearingRight(rotateBearingRight(rotateBearingRight(bearing))),
        ),
      ).toEqual(bearing);
    },
  );
  it.each(['N', 'E', 'S', 'W'] as Bearing[])(
    'is the reverse of a rotateBearingLeft from %s',
    (bearing) => {
      expect(rotateBearingRight(rotateBearingLeft(bearing))).toEqual(bearing);
    },
  );
});

describe('moveUnchecked', () => {
  it('moves north if facing north', () => {
    expect(moveUnchecked({x: 0, y: 0, bearing: 'N'}, 1)).toEqual({
      x: 0,
      y: 1,
      bearing: 'N',
    });
  });
  it('moves east if facing east', () => {
    expect(moveUnchecked({x: 0, y: 0, bearing: 'E'}, 1)).toEqual({
      x: 1,
      y: 0,
      bearing: 'E',
    });
  });
});

describe('goForwards', () => {
  it('moves north if facing north', () => {
    expect(goForwards(makeGrid('10 10'), {x: 0, y: 0, bearing: 'N'})).toEqual({
      x: 0,
      y: 1,
      bearing: 'N',
    });
  });
  it('moves east if facing east', () => {
    expect(goForwards(makeGrid('10 10'), {x: 0, y: 0, bearing: 'E'})).toEqual({
      x: 1,
      y: 0,
      bearing: 'E',
    });
  });
  it('happily jumps off a clean edge', () => {
    expect(goForwards(makeGrid('10 10'), {x: 0, y: 0, bearing: 'W'})).toEqual({
      x: -1,
      y: 0,
      bearing: 'W',
    });
  });
  it('refuses to move off a smelly edge', () => {
    const grid = makeGrid('10 10');
    grid[0][0] = true;
    expect(goForwards(grid, {x: 0, y: 0, bearing: 'W'})).toEqual({
      x: 0,
      y: 0,
      bearing: 'W',
    });
  });
});
