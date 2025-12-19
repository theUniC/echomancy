import { match, P } from "ts-pattern"
import type { CardDefinition } from "../cards/CardDefinition"
import type { CardInstance } from "../cards/CardInstance"
import {
  CardIsNotLandError,
  CardIsNotSpellError,
  CardNotFoundInHandError,
  InvalidCastSpellStepError,
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
type CastSpell = { type: "CAST_SPELL"; playerId: string; cardId: string }
type PassPriority = { type: "PASS_PRIORITY"; playerId: string }

type Actions = AdvanceStep | EndTurn | PlayLand | CastSpell | PassPriority

export type AllowedAction =
  | "ADVANCE_STEP"
  | "END_TURN"
  | "PLAY_LAND"
  | "CAST_SPELL"
  | "PASS_PRIORITY"

export type SpellOnStack = {
  card: CardInstance
  controllerId: string
}

type Stack = {
  spells: SpellOnStack[]
}

type GameParams = {
  id: string
  players: Player[]
  startingPlayerId: string
}

export class Game {
  private playedLands: number
  private playerStates: Map<string, PlayerState>
  private stack: Stack
  private priorityPlayerId: string | null
  private hasPassedPriority: Set<string>

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
    this.stack = { spells: [] }
    this.priorityPlayerId = null
    this.hasPassedPriority = new Set()
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
            graveyard: { cards: [] },
          },
        ]
      }),
    )

    const game = new Game(
      id,
      playersById,
      turnOrder,
      startingPlayerId,
      Step.UNTAP,
      playerStates,
    )
    game.priorityPlayerId = startingPlayerId
    return game
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
      .with(
        { type: "CAST_SPELL", playerId: P.string, cardId: P.string },
        (action) => this.castSpell(action),
      )
      .with({ type: "PASS_PRIORITY", playerId: P.string }, (action) =>
        this.passPriority(action),
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

  getStack(): readonly SpellOnStack[] {
    return [...this.stack.spells]
  }

  getGraveyard(playerId: string): readonly CardInstance[] {
    const playerState = this.getPlayerState(playerId)
    return [...playerState.graveyard.cards]
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

    if (this.playedLands === 0 && this.isMainPhase()) {
      actions.push("PLAY_LAND")
    }

    if (this.isMainPhase() && this.playerHasSpellInHand(playerId)) {
      actions.push("CAST_SPELL")
    }

    if (this.priorityPlayerId === playerId && this.stack.spells.length > 0) {
      actions.push("PASS_PRIORITY")
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
    this.assertIsMainPhase()

    if (this.playedLands > 0) {
      throw new LandLimitExceededError()
    }

    const playerState = this.getPlayerState(action.playerId)
    const { card, cardIndex } = this.findCardInHandByInstanceId(
      playerState,
      action.cardId,
      action.playerId,
    )

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

  private findCardInHandByInstanceId(
    playerState: PlayerState,
    cardId: string,
    playerId: string,
  ): { card: CardInstance; cardIndex: number } {
    const cardIndex = playerState.hand.cards.findIndex(
      (card) => card.instanceId === cardId,
    )

    if (cardIndex === -1) {
      throw new CardNotFoundInHandError(cardId, playerId)
    }

    const card = playerState.hand.cards[cardIndex]

    return { card, cardIndex }
  }

  private castSpell(action: CastSpell): void {
    this.assertIsCurrentPlayer(action.playerId, "CAST_SPELL")

    if (!this.isMainPhase()) {
      throw new InvalidCastSpellStepError()
    }

    const playerState = this.getPlayerState(action.playerId)
    const { card, cardIndex } = this.findCardInHandByInstanceId(
      playerState,
      action.cardId,
      action.playerId,
    )

    if (card.definition.type !== "SPELL") {
      throw new CardIsNotSpellError(action.cardId)
    }

    // Move card from hand to stack
    playerState.hand.cards.splice(cardIndex, 1)
    this.stack.spells.push({
      card,
      controllerId: action.playerId,
    })
  }

  private playerHasSpellInHand(playerId: string): boolean {
    const playerState = this.getPlayerState(playerId)
    if (!playerState) {
      return false
    }

    return playerState.hand.cards.some(
      (card) => card.definition.type === "SPELL",
    )
  }

  private isMainPhase(): boolean {
    return (
      this.currentStep === Step.FIRST_MAIN ||
      this.currentStep === Step.SECOND_MAIN
    )
  }

  private assertIsMainPhase(): void {
    if (!this.isMainPhase()) {
      throw new InvalidPlayLandStepError()
    }
  }

  private passPriority(action: PassPriority): void {
    // Only the player with priority can pass
    if (action.playerId !== this.priorityPlayerId) {
      throw new InvalidPlayerActionError(action.playerId, "PASS_PRIORITY")
    }

    // Register that the player has passed
    this.hasPassedPriority.add(action.playerId)

    // Change priority to the other player
    const otherPlayerId = this.turnOrder.find((id) => id !== action.playerId)
    if (!otherPlayerId) {
      throw new PlayerNotFoundError(action.playerId)
    }
    this.priorityPlayerId = otherPlayerId

    // Detect double pass (both players have passed)
    if (this.hasPassedPriority.size === this.turnOrder.length) {
      this.resolveTopOfStack()
    }
  }

  private resolveTopOfStack(): void {
    // If stack is empty, do nothing
    if (this.stack.spells.length === 0) {
      return
    }

    // Extract the top spell (LIFO)
    const spell = this.stack.spells.pop()
    if (!spell) {
      return
    }

    // Apply dummy effect: move card to controller's graveyard
    const controllerState = this.getPlayerState(spell.controllerId)
    controllerState.graveyard.cards.push(spell.card)

    // Clear priority state
    this.hasPassedPriority.clear()
    this.priorityPlayerId = this.currentPlayerId
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
