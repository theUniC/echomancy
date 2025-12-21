import { match, P } from "ts-pattern"
import type { CardDefinition } from "../cards/CardDefinition"
import type { CardInstance } from "../cards/CardInstance"
import type { Target } from "../targets/Target"
import {
  CannotPayActivationCostError,
  CardIsNotLandError,
  CardIsNotSpellError,
  CardNotFoundInHandError,
  CreatureAlreadyAttackedError,
  InvalidCastSpellStepError,
  InvalidEndTurnError,
  InvalidPlayerActionError,
  InvalidPlayerCountError,
  InvalidPlayLandStepError,
  InvalidStartingPlayerError,
  LandLimitExceededError,
  PermanentHasNoActivatedAbilityError,
  PermanentNotFoundError,
  PlayerNotFoundError,
  TappedCreatureCannotAttackError,
} from "./GameErrors"
import type { Player } from "./Player"
import type { PlayerState } from "./PlayerState"
import { advance } from "./StepMachine"
import { type GameSteps, Step } from "./Steps"

type AdvanceStep = { type: "ADVANCE_STEP"; playerId: string }
type EndTurn = { type: "END_TURN"; playerId: string }
type PlayLand = { type: "PLAY_LAND"; playerId: string; cardId: string }
type CastSpell = {
  type: "CAST_SPELL"
  playerId: string
  cardId: string
  targets: Target[]
}
type PassPriority = { type: "PASS_PRIORITY"; playerId: string }
type DeclareAttacker = {
  type: "DECLARE_ATTACKER"
  playerId: string
  creatureId: string
}
type ActivateAbility = {
  type: "ACTIVATE_ABILITY"
  playerId: string
  permanentId: string
}

type Actions =
  | AdvanceStep
  | EndTurn
  | PlayLand
  | CastSpell
  | PassPriority
  | DeclareAttacker
  | ActivateAbility

export type AllowedAction =
  | "ADVANCE_STEP"
  | "END_TURN"
  | "PLAY_LAND"
  | "CAST_SPELL"
  | "PASS_PRIORITY"
  | "DECLARE_ATTACKER"
  | "ACTIVATE_ABILITY"

export type CreatureState = {
  isTapped: boolean
  isAttacking: boolean
  hasAttackedThisTurn: boolean
}

export type SpellOnStack = {
  kind: "SPELL"
  card: CardInstance
  controllerId: string
  targets: Target[]
}

/**
 * Represents an activated ability on the stack.
 *
 * IMPORTANT: This is NOT a spell. Abilities:
 * - Do not move cards between zones
 * - Do not trigger ETB/LTB effects
 * - Are not affected by "counter target spell" effects
 * - Come from permanents on the battlefield
 * - Resolve independently once on stack (Last Known Information)
 *
 * The effect is stored when activated so the ability can resolve
 * even if the source permanent leaves the battlefield.
 *
 * MVP LIMITATIONS:
 * - No targeting support (targets array always empty)
 * - Only supports permanents as sources (not emblems, etc.)
 *
 * TODO: Add support for:
 * - Targeting in abilities
 * - Mana abilities (special rules, don't use stack)
 * - Loyalty abilities (planeswalkers)
 * - Triggered abilities (separate from activated)
 */
export type AbilityOnStack = {
  kind: "ABILITY"
  sourceId: string // permanentId of the card with the ability
  effect: Effect // Stored when activated for Last Known Information
  controllerId: string
  targets: Target[] // TODO: Implement targeting for abilities
}

type StackItem = SpellOnStack | AbilityOnStack

type Stack = {
  items: StackItem[]
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
  private playersWhoPassedPriority: Set<string>
  private scheduledSteps: GameSteps[]
  private resumeStepAfterScheduled?: GameSteps
  private creatureStates: Map<string, CreatureState>

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
    this.stack = { items: [] }
    this.priorityPlayerId = null
    this.playersWhoPassedPriority = new Set()
    this.scheduledSteps = []
    this.resumeStepAfterScheduled = undefined
    this.creatureStates = new Map()
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
      types: ["LAND"],
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
      .with(
        { type: "DECLARE_ATTACKER", playerId: P.string, creatureId: P.string },
        (action) => this.declareAttacker(action),
      )
      .with(
        { type: "ACTIVATE_ABILITY", playerId: P.string, permanentId: P.string },
        (action) => this.activateAbility(action),
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

  getStack(): readonly StackItem[] {
    return [...this.stack.items]
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

  addScheduledSteps(steps: GameSteps[]): void {
    if (this.scheduledSteps.length === 0) {
      // Calculate resume point: the next step in normal flow
      // that is NOT in the inserted extra phases
      const insertedSteps = new Set(steps)
      let tempStep = this.currentStep

      // Advance until finding a step that is not in the inserted phases
      do {
        const { nextStep } = advance(tempStep)
        tempStep = nextStep
      } while (insertedSteps.has(tempStep))

      this.resumeStepAfterScheduled = tempStep
    }

    this.scheduledSteps.push(...steps)
  }

  getAllowedActionsFor(playerId: string): AllowedAction[] {
    if (this.currentStep === Step.CLEANUP) {
      return []
    }

    if (!this.hasPriority(playerId)) {
      return []
    }

    const actions: AllowedAction[] = []

    if (this.canAdvanceOrEndTurn(playerId)) {
      actions.push("ADVANCE_STEP", "END_TURN")
    }

    if (this.canPlayLand(playerId)) {
      actions.push("PLAY_LAND")
    }

    if (this.canCastSpell(playerId)) {
      actions.push("CAST_SPELL")
    }

    if (this.canDeclareAttacker(playerId)) {
      actions.push("DECLARE_ATTACKER")
    }

    if (this.canActivateAbility(playerId)) {
      actions.push("ACTIVATE_ABILITY")
    }

    if (this.hasSpellsOnStack()) {
      actions.push("PASS_PRIORITY")
    }

    return actions
  }

  // Action handlers (high-level)

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

    this.performStepAdvance()
  }

  private playLand(action: PlayLand): void {
    this.assertIsCurrentPlayer(action.playerId, "PLAY_LAND")
    this.assertIsMainPhase()
    this.assertHasNotPlayedLandThisTurn()

    const playerState = this.getPlayerState(action.playerId)
    const { card, cardIndex } = this.findCardInHandByInstanceId(
      playerState,
      action.cardId,
      action.playerId,
    )

    if (!card.definition.types.includes("LAND")) {
      throw new CardIsNotLandError(action.cardId)
    }

    playerState.hand.cards.splice(cardIndex, 1)
    playerState.battlefield.cards.push(card)

    this.playedLands += 1
  }

  private castSpell(action: CastSpell): void {
    this.assertHasPriority(action.playerId, "CAST_SPELL")

    if (!this.isMainPhase()) {
      throw new InvalidCastSpellStepError()
    }

    // Validate targets
    for (const target of action.targets) {
      if (target.kind === "PLAYER") {
        if (!this.hasPlayer(target.playerId)) {
          throw new InvalidPlayerActionError(action.playerId, "CAST_SPELL")
        }
      }
    }

    const playerState = this.getPlayerState(action.playerId)
    const { card, cardIndex } = this.findCardInHandByInstanceId(
      playerState,
      action.cardId,
      action.playerId,
    )

    if (!this.isCastable(card)) {
      throw new CardIsNotSpellError(action.cardId)
    }

    playerState.hand.cards.splice(cardIndex, 1)
    this.stack.items.push({
      kind: "SPELL",
      card,
      controllerId: action.playerId,
      targets: action.targets,
    })

    this.givePriorityToOpponentOf(action.playerId)
  }

  private passPriority(action: PassPriority): void {
    this.assertHasPriority(action.playerId, "PASS_PRIORITY")

    this.playersWhoPassedPriority.add(action.playerId)

    if (this.bothPlayersHavePassed()) {
      this.resolveTopOfStack()
    } else {
      const opponentId = this.getOpponentOf(action.playerId)
      this.priorityPlayerId = opponentId
    }
  }

  private declareAttacker(action: DeclareAttacker): void {
    this.assertIsCurrentPlayer(action.playerId, "DECLARE_ATTACKER")

    // Verify it's DECLARE_ATTACKERS step
    if (this.currentStep !== Step.DECLARE_ATTACKERS) {
      throw new InvalidPlayerActionError(action.playerId, "DECLARE_ATTACKER")
    }

    // Verify creature exists on battlefield and is controlled by player
    const playerState = this.getPlayerState(action.playerId)
    const creature = playerState.battlefield.cards.find(
      (card) => card.instanceId === action.creatureId,
    )

    if (!creature) {
      throw new PermanentNotFoundError(action.creatureId)
    }

    if (!this.isCreature(creature)) {
      throw new PermanentNotFoundError(action.creatureId)
    }

    const creatureState = this.getCreatureState(action.creatureId)

    // Verify creature is not tapped
    if (creatureState.isTapped) {
      throw new TappedCreatureCannotAttackError(action.creatureId)
    }

    // Verify creature has not attacked this turn
    if (creatureState.hasAttackedThisTurn) {
      throw new CreatureAlreadyAttackedError(action.creatureId)
    }

    // Mark creature as attacking and tapped
    creatureState.isAttacking = true
    creatureState.isTapped = true
    creatureState.hasAttackedThisTurn = true
  }

  private activateAbility(action: ActivateAbility): void {
    this.assertHasPriority(action.playerId, "ACTIVATE_ABILITY")

    // Find the permanent on battlefield controlled by the player
    const playerState = this.getPlayerState(action.playerId)
    const permanent = playerState.battlefield.cards.find(
      (card) => card.instanceId === action.permanentId,
    )

    if (!permanent) {
      throw new PermanentNotFoundError(action.permanentId)
    }

    // Check if permanent has an activated ability
    const ability = permanent.definition.activatedAbility
    if (!ability) {
      throw new PermanentHasNoActivatedAbilityError(action.permanentId)
    }

    // Pay the activation cost
    this.payActivationCost(action.permanentId, ability.cost)

    // Put ability on stack (store effect for Last Known Information)
    this.stack.items.push({
      kind: "ABILITY",
      sourceId: permanent.instanceId,
      effect: ability.effect,
      controllerId: action.playerId,
      targets: [], // TODO: Support targeting in abilities
    })

    this.givePriorityToOpponentOf(action.playerId)
  }

  /**
   * Pays the cost to activate an ability.
   *
   * MVP LIMITATION - Only {T} (tap) cost is supported.
   * TODO: Support other costs (mana, sacrifice, discard, etc.)
   */
  private payActivationCost(permanentId: string, cost: { type: "TAP" }): void {
    if (cost.type === "TAP") {
      // Check if permanent can be tapped
      const creatureState = this.creatureStates.get(permanentId)
      if (!creatureState) {
        throw new PermanentNotFoundError(permanentId)
      }

      if (creatureState.isTapped) {
        throw new CannotPayActivationCostError(
          permanentId,
          "permanent is already tapped",
        )
      }

      // Tap the permanent
      creatureState.isTapped = true
    }
  }

  // Domain logic (mid-level)

  private performStepAdvance(): void {
    // Clear isAttacking when leaving END_OF_COMBAT
    if (this.currentStep === Step.END_OF_COMBAT) {
      this.clearAttackingState()
    }

    // 1. Consume scheduled phases first
    if (this.scheduledSteps.length > 0) {
      const nextScheduledStep = this.scheduledSteps.shift()
      if (nextScheduledStep) {
        this.setCurrentStep(nextScheduledStep)
      }
      return
    }

    // 2. If no extra phases pending but there's a resume point,
    //    jump directly there without using advance()
    if (this.resumeStepAfterScheduled) {
      this.setCurrentStep(this.resumeStepAfterScheduled)
      this.resumeStepAfterScheduled = undefined
      return
    }

    // 3. Normal flow
    const { nextStep, shouldAdvancePlayer } = advance(this.currentStep)

    if (shouldAdvancePlayer) {
      this.advanceToNextPlayer()
    }

    this.setCurrentStep(nextStep)

    if (this.isMainPhase()) {
      this.priorityPlayerId = this.currentPlayerId
      this.playersWhoPassedPriority.clear()
    }
  }

  private setCurrentStep(nextStep: GameSteps): void {
    this.currentStep = nextStep
    this.onEnterStep(nextStep)
  }

  private onEnterStep(step: GameSteps): void {
    if (step === Step.UNTAP) {
      this.autoUntapForCurrentPlayer()
    }
  }

  private autoUntapForCurrentPlayer(): void {
    const playerState = this.getPlayerState(this.currentPlayerId)

    // Untap only creatures controlled by the current player
    for (const card of playerState.battlefield.cards) {
      if (this.isCreature(card)) {
        const creatureState = this.creatureStates.get(card.instanceId)
        if (creatureState) {
          creatureState.isTapped = false
        }
      }
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

    // Reset creature states when turn changes
    this.resetCreatureStatesForNewTurn()
  }

  private resolveTopOfStack(): void {
    if (!this.hasSpellsOnStack()) {
      return
    }

    const stackItem = this.stack.items.pop()
    if (!stackItem) {
      return
    }

    match(stackItem)
      .with({ kind: "SPELL" }, (spell) => {
        // Execute effect if present
        const effect = spell.card.definition.effect
        if (effect) {
          effect.resolve(this, {
            source: spell.card,
            controllerId: spell.controllerId,
            targets: spell.targets,
          })
        }

        const controllerState = this.getPlayerState(spell.controllerId)

        // Move card to appropriate zone: permanents → battlefield, one-shots → graveyard
        if (this.entersBattlefieldOnResolve(spell.card)) {
          controllerState.battlefield.cards.push(spell.card)
          this.initializeCreatureStateIfNeeded(spell.card)

          // Execute ETB trigger if present
          // NOTE: ETB triggers are conceptually separate from spell resolution.
          // In Magic, ETB triggers have their own targeting when applicable.
          // For this MVP, we pass empty targets to make it explicit that
          // ETB targeting is not yet implemented. This prevents accidental
          // dependencies on spell targets and reduces technical debt.
          const etbEffect = spell.card.definition.onEnterBattlefield
          if (etbEffect) {
            etbEffect.resolve(this, {
              source: spell.card,
              controllerId: spell.controllerId,
              targets: [], // ETB targeting not yet implemented
            })
          }
        } else {
          controllerState.graveyard.cards.push(spell.card)
        }
      })
      .with({ kind: "ABILITY" }, (ability) => {
        // Resolve the ability using the stored effect (Last Known Information)
        // The ability resolves even if its source permanent has left the battlefield
        const controllerState = this.getPlayerState(ability.controllerId)
        const permanent = controllerState.battlefield.cards.find(
          (card) => card.instanceId === ability.sourceId,
        )

        if (!permanent) {
          // In the current type system, EffectContext.source is non-nullable.
          // If the permanent cannot be found, we treat this as an error.
          throw new PermanentNotFoundError(ability.sourceId)
        }

        // Use the effect stored when the ability was activated
        ability.effect.resolve(this, {
          source: permanent,
          controllerId: ability.controllerId,
          targets: ability.targets,
        })

        // IMPORTANT: Abilities do NOT move cards or trigger ETB/LTB
        // The source permanent remains on battlefield (if it still exists)
      })
      .exhaustive()

    this.playersWhoPassedPriority.clear()
    this.priorityPlayerId = this.currentPlayerId
  }

  private givePriorityToOpponentOf(playerId: string): void {
    const opponentId = this.getOpponentOf(playerId)
    this.priorityPlayerId = opponentId
    this.playersWhoPassedPriority.clear()
  }

  drawCards(_playerId: string, _amount: number): void {
    // MVP: no-op implementation
    // TODO: implement deck and actual card drawing
  }

  getCreatureState(creatureId: string): CreatureState {
    return this.getCreatureStateOrThrow(creatureId)
  }

  tapPermanent(permanentId: string): void {
    const state = this.getCreatureStateOrThrow(permanentId)
    state.isTapped = true
  }

  untapPermanent(permanentId: string): void {
    const state = this.getCreatureStateOrThrow(permanentId)
    state.isTapped = false
  }

  private getCreatureStateOrThrow(creatureId: string): CreatureState {
    const state = this.creatureStates.get(creatureId)
    if (!state) {
      throw new PermanentNotFoundError(creatureId)
    }
    return state
  }

  // Queries and predicates (low-level)

  private hasPriority(playerId: string): boolean {
    return playerId === this.priorityPlayerId
  }

  private canPlayLand(playerId: string): boolean {
    return (
      playerId === this.currentPlayerId &&
      !this.hasPlayedLandThisTurn() &&
      this.isMainPhase()
    )
  }

  private hasPlayedLandThisTurn(): boolean {
    return this.playedLands > 0
  }

  private isMainPhase(): boolean {
    return (
      this.currentStep === Step.FIRST_MAIN ||
      this.currentStep === Step.SECOND_MAIN
    )
  }

  private hasSpellsOnStack(): boolean {
    return this.stack.items.length > 0
  }

  private bothPlayersHavePassed(): boolean {
    return this.playersWhoPassedPriority.size === this.turnOrder.length
  }

  private canAdvanceOrEndTurn(playerId: string): boolean {
    return playerId === this.currentPlayerId
  }

  private canCastSpell(playerId: string): boolean {
    return this.isMainPhase() && this.playerHasSpellInHand(playerId)
  }

  private canDeclareAttacker(playerId: string): boolean {
    return (
      this.currentStep === Step.DECLARE_ATTACKERS &&
      playerId === this.currentPlayerId &&
      this.playerHasAttackableCreature(playerId)
    )
  }

  private canActivateAbility(playerId: string): boolean {
    return this.playerHasActivatableAbility(playerId)
  }

  private playerHasSpellInHand(playerId: string): boolean {
    const playerState = this.getPlayerState(playerId)
    return playerState.hand.cards.some((card) => this.isCastable(card))
  }

  private playerHasAttackableCreature(playerId: string): boolean {
    const playerState = this.getPlayerState(playerId)
    return playerState.battlefield.cards.some((card) => {
      if (!this.isCreature(card)) {
        return false
      }

      const state = this.creatureStates.get(card.instanceId)
      if (!state) {
        return false
      }

      return !state.isTapped && !state.hasAttackedThisTurn
    })
  }

  private playerHasActivatableAbility(playerId: string): boolean {
    const playerState = this.getPlayerState(playerId)
    return playerState.battlefield.cards.some((card) => {
      // Check if card has an activated ability
      if (!card.definition.activatedAbility) {
        return false
      }

      // Check if the cost can be paid
      const cost = card.definition.activatedAbility.cost
      if (cost.type === "TAP") {
        const state = this.creatureStates.get(card.instanceId)
        if (!state) {
          return false
        }
        // Can activate if not tapped
        return !state.isTapped
      }

      return false
    })
  }

  private isCastable(card: CardInstance): boolean {
    return !card.definition.types.includes("LAND")
  }

  private entersBattlefieldOnResolve(card: CardInstance): boolean {
    const permanentTypes = [
      "CREATURE",
      "ARTIFACT",
      "ENCHANTMENT",
      "PLANESWALKER",
    ]
    return card.definition.types.some((type) => permanentTypes.includes(type))
  }

  private isCreature(card: CardInstance): boolean {
    return card.definition.types.includes("CREATURE")
  }

  initializeCreatureStateIfNeeded(card: CardInstance): void {
    if (this.isCreature(card)) {
      this.creatureStates.set(card.instanceId, {
        isTapped: false,
        isAttacking: false,
        hasAttackedThisTurn: false,
      })
    }
  }

  private clearAttackingState(): void {
    this.updateAllCreatureStates((state) => {
      state.isAttacking = false
    })
  }

  private resetCreatureStatesForNewTurn(): void {
    this.updateAllCreatureStates((state) => {
      state.isAttacking = false
      state.hasAttackedThisTurn = false
    })
  }

  private updateAllCreatureStates(
    updateFn: (state: CreatureState) => void,
  ): void {
    for (const state of this.creatureStates.values()) {
      updateFn(state)
    }
  }

  // Assertions (low-level)

  private assertIsCurrentPlayer(playerId: string, action: string): void {
    if (playerId !== this.currentPlayerId) {
      throw new InvalidPlayerActionError(playerId, action)
    }
  }

  private assertHasPriority(playerId: string, action: string): void {
    if (!this.hasPriority(playerId)) {
      throw new InvalidPlayerActionError(playerId, action)
    }
  }

  private assertIsMainPhase(): void {
    if (!this.isMainPhase()) {
      throw new InvalidPlayLandStepError()
    }
  }

  private assertHasNotPlayedLandThisTurn(): void {
    if (this.hasPlayedLandThisTurn()) {
      throw new LandLimitExceededError()
    }
  }

  // Helpers (lowest-level)

  private getOpponentOf(playerId: string): string {
    const opponentId = this.turnOrder.find((id) => id !== playerId)
    if (!opponentId) {
      throw new PlayerNotFoundError(playerId)
    }
    return opponentId
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

  // Static validators

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
