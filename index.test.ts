import {driveRobots, makeGrid, getStartPosition} from '.';

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

describe.skip('driveRobots', () => {
  it('returns the expected output', () => {
    expect(driveRobots(SAMPLE_INPUT)).toEqual(SAMPLE_OUTPUT);
  });
});

describe('makeGrid', () => {
  it('returns an array of the correct shape', () => {
    expect(makeGrid('10 20').length).toEqual(10);
    expect(makeGrid('10 20')[0].length).toEqual(20);
  });
});

describe('getStartPosition', () => {
  it('returns the correct position', () => {
    expect(getStartPosition('10 20 W')).toEqual({x: 10, y: 20, bearing: 'W'});
  });
});
