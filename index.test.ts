import {
  driveRobots,
  makeGrid,
  getStartPosition,
  rotateLeft,
  rotateRight,
  Bearing,
  Position,
  isOutOfBounds,
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

describe('rotateLeft', () => {
  it.each(['N', 'E', 'S', 'W'])(
    'cancels if applied 4 times from %s',
    (bearing) => {
      expect(
        rotateLeft(rotateLeft(rotateLeft(rotateLeft(bearing as Bearing)))),
      ).toEqual(bearing);
    },
  );

  it('can turn left from North', () => {
    expect(rotateLeft('N')).toEqual('W');
  });
});

describe('rotateRight', () => {
  it('can turn right from North', () => {
    expect(rotateRight('N')).toEqual('E');
  });
  it.each(['N', 'E', 'S', 'W'])(
    'cancels if applied 4 times from %s',
    (bearing) => {
      expect(
        rotateRight(rotateRight(rotateRight(rotateRight(bearing as Bearing)))),
      ).toEqual(bearing);
    },
  );
  it.each(['N', 'E', 'S', 'W'])(
    'is the reverse of a rotateLeft from %s',
    (bearing) => {
      expect(rotateRight(rotateLeft(bearing as Bearing))).toEqual(bearing);
    },
  );
});
