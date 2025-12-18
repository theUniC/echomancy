import type { Player } from "./Player"
import { match, P } from "ts-pattern"
import {
  InvalidPlayerCountError,
  InvalidStartingPlayerError,
  InvalidPlayerActionError,
  PlayerNotFoundError,
} from "./GameErrors"
import { advance } from "./StepMachine"
import { Step, type GameSteps } from "./Steps"

type AdvanceStep = { type: "ADVANCE_STEP"; playerId: string }

type Actions = AdvanceStep

type GameParams = {
  id: string
  players: Player[]
  startingPlayerId: string
}

export class Game {
  constructor(
    public readonly id: string,
    public readonly players: Player[],
    public currentPlayerId: string,
    public currentStep: GameSteps,
  ) {}

  static start({ id, players, startingPlayerId }: GameParams): Game {
    Game.assertMoreThanOnePlayer(players)
    Game.assertStartingPlayerExists(players, startingPlayerId)

    return new Game(id, players, startingPlayerId, Step.UNTAP)
  }

  apply(action: Actions): void {
    match(action)
      .with({ type: "ADVANCE_STEP", playerId: P.string }, (action) =>
        this.advanceStep(action),
      )
      .exhaustive()
  }

  getCurrentPlayer(): Player {
    const player = this.players.find((p) => p.id === this.currentPlayerId)
    if (!player) {
      throw new PlayerNotFoundError(this.currentPlayerId)
    }
    return player
  }

  private advanceStep(action: AdvanceStep): void {
    if (action.playerId !== this.currentPlayerId) {
      throw new InvalidPlayerActionError(action.playerId, "ADVANCE_STEP")
    }

    const { nextStep, shouldAdvancePlayer } = advance(this.currentStep)
    this.currentStep = nextStep

    if (shouldAdvancePlayer) {
      this.advanceToNextPlayer()
    }
  }

  private advanceToNextPlayer(): void {
    const currentIndex = this.players.findIndex(
      (p) => p.id === this.currentPlayerId,
    )
    const nextIndex = (currentIndex + 1) % this.players.length
    this.currentPlayerId = this.players[nextIndex].id
  }

  private static assertStartingPlayerExists(
    players: Player[],
    startingPlayerId: string,
  ) {
    const playerIds = players.map((p) => p.id)
    if (!playerIds.includes(startingPlayerId)) {
      throw new InvalidStartingPlayerError(startingPlayerId)
    }
  }

  private static assertMoreThanOnePlayer(players: Player[]) {
    if (players.length < 2) {
      throw new InvalidPlayerCountError(players.length)
    }
  }
}
