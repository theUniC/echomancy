import type { Player } from "./Player"
import { match, P } from "ts-pattern"
import {
  InvalidPlayerCountError,
  InvalidStartingPlayerError,
  InvalidPlayerActionError,
  PlayerNotFoundError,
  InvalidEndTurnError,
  InvalidPlayLandStepError,
  LandLimitExceededError,
} from "./GameErrors"
import { advance } from "./StepMachine"
import { Step, type GameSteps } from "./Steps"

type AdvanceStep = { type: "ADVANCE_STEP"; playerId: string }
type EndTurn = { type: "END_TURN"; playerId: string }
type PlayLand = { type: "PLAY_LAND"; playerId: string; cardId: string }

type Actions = AdvanceStep | EndTurn | PlayLand

export type AllowedAction = "ADVANCE_STEP" | "END_TURN" | "PLAY_LAND"

type GameParams = {
  id: string
  players: Player[]
  startingPlayerId: string
}

export class Game {
  private playedLands: number

  constructor(
    public readonly id: string,
    private readonly playersById: Map<string, Player>,
    private readonly turnOrder: string[],
    public currentPlayerId: string,
    public currentStep: GameSteps,
  ) {
    this.playedLands = 0
  }

  static start({ id, players, startingPlayerId }: GameParams): Game {
    Game.assertMoreThanOnePlayer(players)
    Game.assertStartingPlayerExists(players, startingPlayerId)

    const playersById = new Map(players.map((p) => [p.id, p]))
    const turnOrder = players.map((p) => p.id)

    return new Game(id, playersById, turnOrder, startingPlayerId, Step.UNTAP)
  }

  apply(action: Actions): void {
    match(action)
      .with({ type: "ADVANCE_STEP", playerId: P.string }, (action) =>
        this.advanceStep(action),
      )
      .with({ type: "END_TURN", playerId: P.string }, (action) =>
        this.endTurn(action),
      )
      .with(
        { type: "PLAY_LAND", playerId: P.string, cardId: P.string },
        (action) => this.playLand(action),
      )
      .exhaustive()
  }

  getCurrentPlayer(): Player {
    const player = this.playersById.get(this.currentPlayerId)
    if (!player) {
      throw new PlayerNotFoundError(this.currentPlayerId)
    }
    return player
  }

  hasPlayer(playerId: string): boolean {
    return this.playersById.has(playerId)
  }

  getPlayersInTurnOrder(): readonly string[] {
    return [...this.turnOrder]
  }

  getAllowedActionsFor(playerId: string): AllowedAction[] {
    if (playerId !== this.currentPlayerId) {
      return []
    }

    if (this.currentStep === Step.CLEANUP) {
      return []
    }

    const actions: AllowedAction[] = ["ADVANCE_STEP", "END_TURN"]

    if (
      this.playedLands === 0 &&
      (this.currentStep === Step.FIRST_MAIN ||
        this.currentStep === Step.SECOND_MAIN)
    ) {
      actions.push("PLAY_LAND")
    }

    return actions
  }

  private advanceStep(action: AdvanceStep): void {
    this.assertIsCurrentPlayer(action.playerId, "ADVANCE_STEP")
    this.performStepAdvance()
  }

  private endTurn(action: EndTurn): void {
    this.assertIsCurrentPlayer(action.playerId, "END_TURN")

    if (this.currentStep === Step.CLEANUP) {
      throw new InvalidEndTurnError()
    }

    while ((this.currentStep as GameSteps) !== Step.CLEANUP) {
      this.performStepAdvance()
    }

    // Advance once more from CLEANUP to move to the next player
    this.performStepAdvance()
  }

  private playLand(action: PlayLand): void {
    this.assertIsCurrentPlayer(action.playerId, "PLAY_LAND")

    if (
      this.currentStep !== Step.FIRST_MAIN &&
      this.currentStep !== Step.SECOND_MAIN
    ) {
      throw new InvalidPlayLandStepError()
    }

    if (this.playedLands > 0) {
      throw new LandLimitExceededError()
    }

    this.playedLands += 1
  }

  private performStepAdvance(): void {
    const { nextStep, shouldAdvancePlayer } = advance(this.currentStep)
    this.currentStep = nextStep

    if (shouldAdvancePlayer) {
      this.advanceToNextPlayer()
    }
  }

  private assertIsCurrentPlayer(playerId: string, action: string): void {
    if (playerId !== this.currentPlayerId) {
      throw new InvalidPlayerActionError(playerId, action)
    }
  }

  private advanceToNextPlayer(): void {
    const currentIndex = this.turnOrder.indexOf(this.currentPlayerId)
    if (currentIndex < 0) {
      throw new PlayerNotFoundError(this.currentPlayerId)
    }

    const nextIndex = (currentIndex + 1) % this.turnOrder.length
    this.currentPlayerId = this.turnOrder[nextIndex]
    this.playedLands = 0
  }

  private static assertStartingPlayerExists(
    players: Player[],
    startingPlayerId: string,
  ) {
    const exists = players.some((p) => p.id === startingPlayerId)
    if (!exists) {
      throw new InvalidStartingPlayerError(startingPlayerId)
    }
  }

  private static assertMoreThanOnePlayer(players: Player[]) {
    if (players.length < 2) {
      throw new InvalidPlayerCountError(players.length)
    }
  }
}
