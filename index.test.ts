import { driveRobots } from ".";

const SAMPLE_INPUT = `
5 3
1 1 E
RFRFRFRF
3 2 N
FRRFLLFFRRFLL
0 3 W
LLFFFLFLFL
`.trim()

const SAMPLE_OUTPUT = `
1 1 E
3 3 N LOST
2 3 S
`.trim()

describe.skip("driveRobots", () => {
    it("returns what it is given", () => {
        expect(driveRobots(SAMPLE_INPUT)).toEqual(SAMPLE_OUTPUT)
    })
})
