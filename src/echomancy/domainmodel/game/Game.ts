import { v4 as uuidv4 } from 'uuid';
import { Player } from './Player';
import { AdvanceStep } from './actions/AdvanceStep';
import { match, P } from 'ts-pattern';
import { InvalidPlayerCountError, InvalidStartingPlayerError, InvalidPlayerActionError, PlayerNotFoundError } from './GameErrors';

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
} as const;

export type GameSteps = typeof Step[keyof typeof Step];

const STEP_ORDER: GameSteps[] = [
    Step.UNTAP,
    Step.UPKEEP,
    Step.DRAW,
    Step.MAIN1,
    Step.BEGINING_OF_COMBAT,
    Step.DECLARE_ATTACKERS,
    Step.DECLARE_BLOCKERS,
    Step.COMBAT_DAMAGE,
    Step.END_OF_COMBAT,
    Step.SECOND_MAIN,
    Step.END_STEP,
    Step.CLEANUP
];

type Actions =
    | { type: "ADVANCE_STEP", playerId: string };

export class Game {
    constructor(
        public readonly id: string,
        public readonly players: Player[],
        public currentPlayerId: string,
        public currentStep: GameSteps,
    ) { }

    static start(
        players: Player[],
        startingPlayerId: string
    ): Game {
        if (players.length < 2) {
            throw new InvalidPlayerCountError(players.length);
        }

        const playerIds = players.map(p => p.id);
        if (!playerIds.includes(startingPlayerId)) {
            throw new InvalidStartingPlayerError(startingPlayerId);
        }

        return new Game(
            uuidv4(),
            players,
            startingPlayerId,
            Step.UNTAP
        );
    }

    apply(action: Actions): void {
        match(action)
            .with({ type: "ADVANCE_STEP", playerId: P.string }, (action) => this.advanceStep(new AdvanceStep(action.playerId)))
            .exhaustive();
    }

    private advanceStep(action: AdvanceStep): void {
        if (action.playerId !== this.currentPlayerId) {
            throw new InvalidPlayerActionError(action.playerId, 'ADVANCE_STEP');
        }

        const currentStepIndex = STEP_ORDER.indexOf(this.currentStep);
        const nextStepIndex = (currentStepIndex + 1) % STEP_ORDER.length;
        this.currentStep = STEP_ORDER[nextStepIndex];

        if (this.currentStep === Step.UNTAP) {
            this.advanceToNextPlayer();
        }
    }

    private advanceToNextPlayer(): void {
        const currentIndex = this.players.findIndex(p => p.id === this.currentPlayerId);
        const nextIndex = (currentIndex + 1) % this.players.length;
        this.currentPlayerId = this.players[nextIndex].id;
    }

    getCurrentPlayer(): Player {
        const player = this.players.find(p => p.id === this.currentPlayerId);
        if (!player) {
            throw new PlayerNotFoundError(this.currentPlayerId);
        }
        return player;
    }
}