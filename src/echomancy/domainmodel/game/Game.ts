import { match, P } from "ts-pattern"
import type { ActivationCost } from "../abilities/ActivatedAbility"
import type { CardDefinition } from "../cards/CardDefinition"
import type { CardInstance } from "../cards/CardInstance"
import type { EffectContext } from "../effects/EffectContext"
import { type ZoneName, ZoneNames } from "../zones/Zone"
import type { Actions, AllowedAction } from "./GameActions"
import {
  CannotPayActivationCostError,
  CardIsNotLandError,
  CardIsNotSpellError,
  CardNotFoundInHandError,
  CreatureAlreadyAttackedError,
  InsufficientManaError,
  InvalidCastSpellStepError,
  InvalidEndTurnError,
  InvalidManaAmountError,
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
import { type GameEvent, GameEventTypes } from "./GameEvents"
import type { Player } from "./Player"
import type { PlayerState } from "./PlayerState"
import type { AbilityOnStack, SpellOnStack, StackItem } from "./StackTypes"
import { advance } from "./StepMachine"
import { type GameSteps, Step } from "./Steps"

// Re-export stack types for backward compatibility
export type { AbilityOnStack, SpellOnStack, StackItem }

export type CreatureState = {
  isTapped: boolean
  isAttacking: boolean
  hasAttackedThisTurn: boolean
}

/**
 * Planeswalker state (MVP - placeholder only)
 *
 * In the MVP, planeswalkers exist as permanents but do not have functional state.
 * This is a placeholder type for future expansion.
 *
 * TODO: Implement loyalty counters
 * TODO: Implement loyalty ability activation (once per turn limit)
 * TODO: Implement damage redirection to planeswalkers
 * TODO: Implement planeswalker uniqueness rule
 */
export type PlaneswalkerState = Record<string, never>

// ============================================================================
// MANA POOL TYPES
// ============================================================================

export type ManaColor = "W" | "U" | "B" | "R" | "G" | "C"

export type ManaPool = {
  W: number
  U: number
  B: number
  R: number
  G: number
  C: number
}

export type ManaPoolSnapshot = Readonly<ManaPool>

type Stack = {
  items: StackItem[]
}

type PermanentOnBattlefield = {
  permanent: CardInstance
  controllerId: string
}

type TriggeredAbility = {
  effect: (game: Game, context: EffectContext) => void
  controllerId: string
  source: CardInstance
}

type GameParams = {
  id: string
  players: Player[]
  startingPlayerId: string
}

export class Game {
  private playedLands: number
  private playerStates: Map<string, PlayerState>
  private manaPools: Map<string, ManaPool>
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
    manaPools: Map<string, ManaPool>,
  ) {
    this.playedLands = 0
    this.playerStates = playerStates
    this.manaPools = manaPools
    this.stack = { items: [] }
    this.priorityPlayerId = null
    this.playersWhoPassedPriority = new Set()
    this.scheduledSteps = []
    this.resumeStepAfterScheduled = undefined
    this.creatureStates = new Map()
  }

  // ============================================================================
  // STATIC FACTORY & VALIDATORS
  // ============================================================================

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

    // Initialize mana pools (all colors start at 0)
    const manaPools = new Map(
      players.map((player) => [player.id, Game.createEmptyManaPool()]),
    )

    const game = new Game(
      id,
      playersById,
      turnOrder,
      startingPlayerId,
      Step.UNTAP,
      playerStates,
      manaPools,
    )
    game.priorityPlayerId = startingPlayerId
    return game
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

  private static createEmptyManaPool(): ManaPool {
    return { W: 0, U: 0, B: 0, R: 0, G: 0, C: 0 }
  }

  // ============================================================================
  // PUBLIC API - HIGH LEVEL (Commands & Primary Queries)
  // ============================================================================

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

  // ============================================================================
  // PUBLIC API - QUERIES (Game State Access)
  // ============================================================================

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

  getCreatureState(creatureId: string): CreatureState {
    return this.getCreatureStateOrThrow(creatureId)
  }

  getManaPool(playerId: string): ManaPoolSnapshot {
    const pool = this.manaPools.get(playerId)
    if (!pool) {
      throw new PlayerNotFoundError(playerId)
    }
    // Return a copy to prevent mutation
    return { ...pool }
  }

  // ============================================================================
  // PUBLIC API - COMMANDS (State Mutations)
  // ============================================================================

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

  drawCards(_playerId: string, _amount: number): void {
    // MVP: no-op implementation
    // TODO: implement deck and actual card drawing
  }

  tapPermanent(permanentId: string): void {
    const state = this.getCreatureStateOrThrow(permanentId)
    state.isTapped = true
  }

  untapPermanent(permanentId: string): void {
    const state = this.getCreatureStateOrThrow(permanentId)
    state.isTapped = false
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

  /**
   * Add mana of a specific color to a player's mana pool.
   *
   * @param playerId - The player receiving the mana
   * @param color - The color of mana to add
   * @param amount - The amount of mana to add (must be > 0)
   * @throws PlayerNotFoundError if player doesn't exist
   * @throws InvalidManaAmountError if amount <= 0
   */
  addMana(playerId: string, color: ManaColor, amount: number): void {
    if (amount <= 0) {
      throw new InvalidManaAmountError(amount)
    }

    const pool = this.manaPools.get(playerId)
    if (!pool) {
      throw new PlayerNotFoundError(playerId)
    }

    pool[color] += amount
  }

  /**
   * Spend mana of a specific color from a player's mana pool.
   *
   * @param playerId - The player spending the mana
   * @param color - The color of mana to spend
   * @param amount - The amount of mana to spend (must be > 0)
   * @throws PlayerNotFoundError if player doesn't exist
   * @throws InvalidManaAmountError if amount <= 0
   * @throws InsufficientManaError if player doesn't have enough mana
   */
  spendMana(playerId: string, color: ManaColor, amount: number): void {
    if (amount <= 0) {
      throw new InvalidManaAmountError(amount)
    }

    const pool = this.manaPools.get(playerId)
    if (!pool) {
      throw new PlayerNotFoundError(playerId)
    }

    if (pool[color] < amount) {
      throw new InsufficientManaError(playerId, color, amount, pool[color])
    }

    pool[color] -= amount
  }

  /**
   * Clear all mana from a specific player's mana pool.
   *
   * @param playerId - The player whose mana pool to clear
   * @throws PlayerNotFoundError if player doesn't exist
   */
  clearManaPool(playerId: string): void {
    const pool = this.manaPools.get(playerId)
    if (!pool) {
      throw new PlayerNotFoundError(playerId)
    }

    // Reset all colors to 0 using Object.assign for cleaner code
    Object.assign(pool, Game.createEmptyManaPool())
  }

  /**
   * Clear all mana from all players' mana pools.
   *
   * This is called during step transitions (currently at CLEANUP step only in MVP).
   *
   * TODO: In real Magic, mana empties at the end of each step and phase.
   * TODO: For MVP we clear at CLEANUP only to keep behavior deterministic.
   */
  clearAllManaPools(): void {
    for (const playerId of this.turnOrder) {
      this.clearManaPool(playerId)
    }
  }

  /**
   * enterBattlefield - Central entry point for all permanents entering the battlefield
   *
   * ⚠️ LOW-LEVEL API - INTERNAL USE ONLY ⚠️
   *
   * This is a low-level game engine method that bypasses normal game rule validation.
   * It does NOT check:
   * - Whether you can afford to play this permanent
   * - Whether it's the right timing/phase to play it
   * - Whether you have permission to play it
   * - Any other game rule restrictions
   *
   * @internal
   *
   * Valid use cases:
   * - Game mechanics (resolveTopOfStack, playLand) - these already validated rules
   * - Test helpers (addCreatureToBattlefield) - for setting up test scenarios
   * - Future effects (blink, reanimate, tokens) - special game effects
   *
   * For normal gameplay, use game actions instead:
   * - game.apply({ type: "CAST_SPELL", ... }) - validates mana, timing, etc.
   * - game.apply({ type: "PLAY_LAND", ... }) - validates land-per-turn limit, etc.
   *
   * This method represents the single source of truth for when a permanent enters
   * the battlefield. ALL paths to the battlefield MUST go through this method.
   *
   * Responsibilities:
   * 1. Move the permanent to the controller's battlefield
   * 2. Initialize creature state if the permanent is a creature
   * 3. Execute ETB (enter-the-battlefield) effects if present
   *
   * ETB Implementation Notes (MVP):
   * - ETB effects execute immediately (not queued as separate triggers)
   * - ETB effects receive empty targets (targeting not yet implemented)
   * - ETB effects do NOT inherit targets from spells
   * - Full triggered ability system will come later
   *
   * TODO: ETB with targets is not implemented yet
   * TODO: ETB does not use the stack as a separate trigger
   * TODO: Full triggered abilities will be implemented later
   * TODO: Replacement effects (e.g., "enters tapped") not yet implemented
   * TODO: Complete Last Known Information handling not yet implemented
   *
   * @param permanent - The CardInstance entering the battlefield
   * @param controllerId - The ID of the player who controls this permanent
   */
  enterBattlefield(
    permanent: CardInstance,
    controllerId: string,
    fromZone?: ZoneName,
  ): void {
    // 1. Move permanent to battlefield
    const controllerState = this.getPlayerState(controllerId)
    controllerState.battlefield.cards.push(permanent)

    // 2. Initialize creature state if needed
    this.initializeCreatureStateIfNeeded(permanent)

    // 3. Emit zone change event and evaluate triggers
    // NOTE: fromZone defaults to STACK for backward compatibility
    // Most permanents enter from the stack when spells resolve
    this.evaluateTriggers({
      type: GameEventTypes.ZONE_CHANGED,
      card: permanent,
      fromZone: fromZone ?? ZoneNames.STACK,
      toZone: ZoneNames.BATTLEFIELD,
      controllerId: controllerId,
    })
  }

  /**
   * Moves a permanent from the battlefield to the graveyard
   *
   * This method handles the complete zone transition including:
   * - Removing the permanent from battlefield
   * - Adding it to the appropriate graveyard
   * - Cleaning up associated state (creature state, etc.)
   * - Emitting ZONE_CHANGED event
   * - Evaluating triggered abilities (e.g., "dies" triggers)
   *
   * @param permanentId - The instance ID of the permanent to move
   * @param _reason - The reason for the zone change (reserved for future event metadata)
   * @throws PermanentNotFoundError if permanent is not on any battlefield
   */
  movePermanentToGraveyard(
    permanentId: string,
    _reason: "sacrifice" | "destroy" | "state-based",
  ): void {
    // 1. Find the permanent on any battlefield
    const playerIds = this.getPlayersInTurnOrder()
    let permanent: CardInstance | null = null
    let controllerState: PlayerState | null = null
    let controllerId: string | null = null
    let permanentIndex: number | null = null

    for (const playerId of playerIds) {
      const playerState = this.getPlayerState(playerId)
      const index = playerState.battlefield.cards.findIndex(
        (card) => card.instanceId === permanentId,
      )
      if (index !== -1) {
        permanent = playerState.battlefield.cards[index]
        controllerState = playerState
        controllerId = playerId
        permanentIndex = index
        break
      }
    }

    if (
      !permanent ||
      !controllerState ||
      !controllerId ||
      permanentIndex === null
    ) {
      throw new PermanentNotFoundError(permanentId)
    }

    // 2. Remove from controller's battlefield
    controllerState.battlefield.cards.splice(permanentIndex, 1)

    // 3. Add to owner's graveyard (cards always go to owner's graveyard in Magic)
    const ownerState = this.getPlayerState(permanent.ownerId)
    ownerState.graveyard.cards.push(permanent)

    // 4. Clean up creature state if needed
    if (this.creatureStates.has(permanentId)) {
      this.creatureStates.delete(permanentId)
    }

    // 5. Emit zone change event and evaluate triggers
    // This enables "dies" triggers and other zone-change abilities
    this.evaluateTriggers({
      type: GameEventTypes.ZONE_CHANGED,
      card: permanent,
      fromZone: ZoneNames.BATTLEFIELD,
      toZone: ZoneNames.GRAVEYARD,
      controllerId: controllerId,
    })
  }

  // ============================================================================
  // PRIVATE - ACTION HANDLERS (High-Level Commands)
  // ============================================================================

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

    // Remove land from hand
    playerState.hand.cards.splice(cardIndex, 1)

    // Use enterBattlefield to ensure consistent ETB handling
    this.enterBattlefield(card, action.playerId)

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

    // Emit creature declared attacker event and evaluate triggers
    this.evaluateTriggers({
      type: GameEventTypes.CREATURE_DECLARED_ATTACKER,
      creature: creature,
      controllerId: action.playerId,
    })
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

  // ============================================================================
  // PRIVATE - DOMAIN LOGIC (Mid-Level Game Mechanics)
  // ============================================================================

  /**
   * Pays the cost to activate an ability.
   *
   * MVP LIMITATION - Only {T} (tap) cost is supported.
   * MVP LIMITATION - Only creatures can have tap/untap state tracked.
   * Artifacts, enchantments, and lands with activated abilities are assumed
   * to be untapped and can always pay tap costs.
   *
   * TODO: Support other costs (mana, sacrifice, discard, etc.)
   * TODO: Track tapped state for all permanents, not just creatures
   */
  private payActivationCost(permanentId: string, cost: ActivationCost): void {
    if (cost.type === "TAP") {
      // Check if permanent can be tapped
      const creatureState = this.creatureStates.get(permanentId)

      if (creatureState) {
        // It's a creature - check if already tapped
        if (creatureState.isTapped) {
          throw new CannotPayActivationCostError(
            permanentId,
            "permanent is already tapped",
          )
        }
        // Tap the creature
        creatureState.isTapped = true
      }
      // If not a creature (artifact, enchantment, land), assume it can be tapped
      // TODO: Track tapped state for all permanents, not just creatures
    }
  }

  private performStepAdvance(): void {
    // Emit combat ended event and clear isAttacking when leaving END_OF_COMBAT
    if (this.currentStep === Step.END_OF_COMBAT) {
      this.evaluateTriggers({
        type: GameEventTypes.COMBAT_ENDED,
        activePlayerId: this.currentPlayerId,
      })
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
    // Execute step-specific actions first
    if (step === Step.UNTAP) {
      this.autoUntapForCurrentPlayer()
    }

    // Clear mana pools when entering CLEANUP step (MVP behavior)
    // TODO: In real Magic, mana empties at the end of each step and phase.
    // TODO: For MVP we clear at CLEANUP only to keep behavior deterministic.
    if (step === Step.CLEANUP) {
      this.clearAllManaPools()
    }

    // Emit step started event and evaluate triggers
    this.evaluateTriggers({
      type: GameEventTypes.STEP_STARTED,
      step: step,
      activePlayerId: this.currentPlayerId,
    })
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
      .with({ kind: "SPELL" }, (spell) => this.resolveSpell(spell))
      .with({ kind: "ABILITY" }, (ability) => this.resolveAbility(ability))
      .exhaustive()

    this.playersWhoPassedPriority.clear()
    this.priorityPlayerId = this.currentPlayerId
  }

  private resolveSpell(spell: SpellOnStack): void {
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
    if (this.isPermanent(spell.card)) {
      // Use enterBattlefield to ensure consistent ETB handling
      this.enterBattlefield(spell.card, spell.controllerId)
    } else {
      controllerState.graveyard.cards.push(spell.card)
    }

    // Emit spell resolved event
    // NOTE: This fires AFTER the spell's effect has been applied
    // and the card has been moved to its final zone
    this.evaluateTriggers({
      type: GameEventTypes.SPELL_RESOLVED,
      card: spell.card,
      controllerId: spell.controllerId,
    })
  }

  private resolveAbility(ability: AbilityOnStack): void {
    // Resolve the ability using the stored effect (Last Known Information)
    // The ability resolves even if its source permanent has left the battlefield
    const controllerState = this.getPlayerState(ability.controllerId)
    const permanent = controllerState.battlefield.cards.find(
      (card) => card.instanceId === ability.sourceId,
    )

    // Use the effect stored when the ability was activated
    ability.effect.resolve(this, {
      source: permanent, // May be undefined if permanent left battlefield
      controllerId: ability.controllerId,
      targets: ability.targets,
    })

    // IMPORTANT: Abilities do NOT move cards or trigger ETB/LTB
    // The source permanent remains on battlefield (if it still exists)
  }

  private givePriorityToOpponentOf(playerId: string): void {
    const opponentId = this.getOpponentOf(playerId)
    this.priorityPlayerId = opponentId
    this.playersWhoPassedPriority.clear()
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

  /**
   * Evaluates triggers for a game event.
   *
   * TRIGGER SYSTEM: Declarative, not reactive. Cards declare triggers;
   * Game evaluates them at specific points (NOT continuously).
   *
   * Called ONLY at:
   * 1. After enterBattlefield() → ZONE_CHANGED (ETB)
   * 2. After declareAttacker() → CREATURE_DECLARED_ATTACKER
   * 3. After resolveSpell() → SPELL_RESOLVED
   * 4. On step transition → STEP_STARTED, COMBAT_ENDED
   *
   * Flow: Collect permanents → Check trigger conditions → Execute matching triggers
   *
   * CRITICAL MVP LIMITATION: Triggers execute immediately (see executeTriggeredAbilities).
   *
   * See ABILITY_CONTRACT_MVP.md for complete evaluation rules.
   */
  private evaluateTriggers(event: GameEvent): void {
    const permanents = this.collectPermanentsFromBattlefield()
    const triggersToExecute = this.collectMatchingTriggers(event, permanents)
    this.executeTriggeredAbilities(triggersToExecute)
  }

  /**
   * Collects all permanents from all players' battlefields.
   *
   * @returns Array of permanents with their controller IDs
   */
  private collectPermanentsFromBattlefield(): PermanentOnBattlefield[] {
    const permanents: PermanentOnBattlefield[] = []

    for (const [playerId, playerState] of this.playerStates.entries()) {
      for (const permanent of playerState.battlefield.cards) {
        permanents.push({ permanent, controllerId: playerId })
      }
    }

    return permanents
  }

  /**
   * Collects triggers that match the given event.
   *
   * Filters permanents to find triggers that:
   * 1. Watch for this event type
   * 2. Have their condition met
   *
   * @param event - The game event
   * @param permanents - Permanents to check for triggers
   * @returns Array of triggered abilities ready to execute
   */
  private collectMatchingTriggers(
    event: GameEvent,
    permanents: PermanentOnBattlefield[],
  ): TriggeredAbility[] {
    const triggersToExecute: TriggeredAbility[] = []

    for (const { permanent, controllerId } of permanents) {
      const triggers = permanent.definition.triggers
      if (!triggers) continue

      for (const trigger of triggers) {
        // Skip if event type doesn't match (avoid evaluating condition unnecessarily)
        if (trigger.eventType !== event.type) continue

        // Evaluate condition only for matching event types
        const isConditionMet = trigger.condition(this, event, permanent)

        if (isConditionMet) {
          triggersToExecute.push({
            effect: trigger.effect,
            controllerId,
            source: permanent,
          })
        }
      }
    }

    return triggersToExecute
  }

  /**
   * Executes triggered abilities in order.
   *
   * CRITICAL MVP LIMITATION: Executes triggers IMMEDIATELY (not on stack).
   *
   * TODO(stack): Create TriggeredAbilityOnStack instead of executing immediately
   * TODO(stack): Add triggered abilities to stack, allow responses
   * TODO(apnap): Implement APNAP ordering (active player first, then non-active)
   * TODO(targeting): Support targeting in trigger effects
   *
   * See ABILITY_CONTRACT_MVP.md for complete contract.
   */
  private executeTriggeredAbilities(
    triggeredAbilities: TriggeredAbility[],
  ): void {
    for (const ability of triggeredAbilities) {
      ability.effect(this, {
        source: ability.source,
        controllerId: ability.controllerId,
        targets: [],
      })
    }
  }

  // ============================================================================
  // PRIVATE - QUERIES & PREDICATES (Low-Level Checks)
  // ============================================================================

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

  private isPermanent(card: CardInstance): boolean {
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

  // ============================================================================
  // PRIVATE - ASSERTIONS & HELPERS (Lowest-Level Utilities)
  // ============================================================================

  private getCreatureStateOrThrow(creatureId: string): CreatureState {
    const state = this.creatureStates.get(creatureId)
    if (!state) {
      throw new PermanentNotFoundError(creatureId)
    }
    return state
  }

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
}
