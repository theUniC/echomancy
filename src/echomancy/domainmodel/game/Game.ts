import { v4 as uuidv4 } from 'uuid';

const Step = {
    UNTAP: "UNTAP",
    UPKEEP: "UPKEEP",
    DRAW: "DRAW",
    MAIN1: "MAIN1",
    BEGINING_OF_COMBAT: "BEGINING_OF_COMBAT",
    DECLARE_ATTACKERS: "DECLARE_ATTACKERS",
    DECLARE_BLOCKERS: "DECLARE_BLOCKERS",
    COMBAT_DAMAGE: "COMBAT_DAMAGE",
    END_OF_COMBAT: "END_OF_COMBAT",
    SECOND_MAIN: "SECOND_MAIN",
    END_STEP: "END_STEP",
    CLEANUP: "CLEANUP"
};

export type GameSteps = typeof Step[keyof typeof Step];

export class Game {
    constructor(
        public id: string,
        public playerIds: string[],
    ) { }

    static start(playerIds: string[]): Game {
        return new Game(uuidv4(), playerIds);
    }
}