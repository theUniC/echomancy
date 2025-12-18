import { match, P } from "ts-pattern"
import type { CardDefinition } from "../cards/CardDefinition"
import type { CardInstance } from "../cards/CardInstance"
import {
  CardIsNotLandError,
  CardNotFoundInHandError,
  InvalidEndTurnError,
  InvalidPlayerActionError,
  InvalidPlayerCountError,
  InvalidPlayLandStepError,
  InvalidStartingPlayerError,
  LandLimitExceededError,
  PlayerNotFoundError,
} from "./GameErrors"
import type { Player } from "./Player"
import type { PlayerState } from "./PlayerState"
import { advance } from "./StepMachine"
import { type GameSteps, Step } from "./Steps"

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
  private playerStates: Map<string, PlayerState>

  constructor(
    public readonly id: string,
    private readonly playersById: Map<string, Player>,
    private readonly turnOrder: string[],
    public currentPlayerId: string,
    public currentStep: GameSteps,
    playerStates: Map<string, PlayerState>,
  ) {
    this.playedLands = 0
    this.playerStates = playerStates
  }

  static start({ id, players, startingPlayerId }: GameParams): Game {
    Game.assertMoreThanOnePlayer(players)
    Game.assertStartingPlayerExists(players, startingPlayerId)

    const playersById = new Map(players.map((p) => [p.id, p]))
    const turnOrder = players.map((p) => p.id)

    // Create dummy land card for MVP
    const dummyLandDefinition: CardDefinition = {
      id: "dummy-land",
      name: "Dummy Land",
      type: "LAND",
    }

    // Initialize player states with one land in hand
    const playerStates = new Map(
      players.map((player) => {
        const dummyLandInstance: CardInstance = {
          instanceId: `${player.id}-dummy-land-instance`,
          definition: dummyLandDefinition,
          ownerId: player.id,
        }

        return [
          player.id,
          {
            hand: { cards: [dummyLandInstance] },
            battlefield: { cards: [] },
          },
        ]
      }),
    )

    return new Game(
      id,
      playersById,
      turnOrder,
      startingPlayerId,
      Step.UNTAP,
      playerStates,
    )
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

  getPlayerState(playerId: string): PlayerState {
    const playerState = this.playerStates.get(playerId)
    if (!playerState) {
      throw new PlayerNotFoundError(playerId)
    }
    return playerState
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

    const playerState = this.playerStates.get(action.playerId)
    if (!playerState) {
      throw new PlayerNotFoundError(action.playerId)
    }

    // Find card in hand
    const cardIndex = playerState.hand.cards.findIndex(
      (card) => card.instanceId === action.cardId,
    )

    if (cardIndex === -1) {
      throw new CardNotFoundInHandError(action.cardId, action.playerId)
    }

    const card = playerState.hand.cards[cardIndex]

    // Verify it's a land
    if (card.definition.type !== "LAND") {
      throw new CardIsNotLandError(action.cardId)
    }

    // Move card from hand to battlefield
    playerState.hand.cards.splice(cardIndex, 1)
    playerState.battlefield.cards.push(card)

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
