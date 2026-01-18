import { match, P } from "ts-pattern"
import type { ActivationCost } from "../abilities/ActivatedAbility"
import { StaticAbilities, type StaticAbility } from "../cards/CardDefinition"
import type { CardInstance } from "../cards/CardInstance"
import { type ZoneName, ZoneNames } from "../zones/Zone"
import { Battlefield } from "./entities/Battlefield"
import { Graveyard } from "./entities/Graveyard"
import { Hand } from "./entities/Hand"
import { TheStack } from "./entities/TheStack"
import type {
  Actions,
  ActivateAbility,
  AdvanceStep,
  AllowedAction,
  CastSpell,
  DeclareAttacker,
  DeclareBlocker,
  EndTurn,
  PassPriority,
  PlayLand,
} from "./GameActions"
import {
  CannotAddPlayerAfterStartError,
  CannotPayActivationCostError,
  CardIsNotLandError,
  CardIsNotSpellError,
  CardNotFoundInHandError,
  CreatureHasSummoningSicknessError,
  DuplicatePlayerError,
  GameAlreadyStartedError,
  GameNotStartedError,
  InsufficientManaError,
  InvalidCastSpellStepError,
  InvalidCounterAmountError,
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
} from "./GameErrors"
import { type GameEvent, GameEventTypes } from "./GameEvents"
import type { GameStateExport } from "./GameStateExport"
import type { Player } from "./Player"
import type { PlayerState } from "./PlayerState"
import type { AbilityOnStack, SpellOnStack, StackItem } from "./StackTypes"
import { advance } from "./StepMachine"
import { type GameSteps, Step } from "./Steps"
import {
  type CombatValidationContext,
  validateDeclareAttacker,
  validateDeclareBlocker,
} from "./services/CombatDeclarations"
import { CombatResolution } from "./services/CombatResolution"
import {
  type ExportableGameContext,
  exportGameState,
} from "./services/GameStateExporter"
import { StateBasedActions } from "./services/StateBasedActions"
import {
  TriggerEvaluation,
  type TriggeredAbilityInfo,
} from "./services/TriggerEvaluation"

// Re-export stack types for backward compatibility
export type { AbilityOnStack, SpellOnStack, StackItem }

/**
 * Game configuration constants
 */
const MIN_PLAYERS = 2
const _DEFAULT_CREATURE_POWER = 0
const _DEFAULT_CREATURE_TOUGHNESS = 1

/**
 * Game lifecycle states
 *
 * CREATED: Game instance exists, but rules engine not active yet
 * STARTED: Rules engine active, game in progress
 * FINISHED: Game concluded (future use)
 */
export enum GameLifecycleState {
  CREATED = "CREATED",
  STARTED = "STARTED",
  FINISHED = "FINISHED",
}

/**
 * Reason for a permanent being moved to the graveyard
 */
export enum GraveyardReason {
  SACRIFICE = "sacrifice",
  DESTROY = "destroy",
  STATE_BASED = "state-based",
}

// ============================================================================
// CREATURE STATE TYPES
// ============================================================================

// Re-export CounterType from CreatureState VO for backward compatibility
export type { CounterType } from "./valueobjects/CreatureState"

// Import CreatureState VO (renamed to avoid conflict with backward compat export)
import {
  type CounterType,
  CreatureState as CreatureStateVO,
} from "./valueobjects/CreatureState"

/**
 * Creature state for external use (backward compatibility).
 *
 * This is the export type returned by Game.getCreatureState().
 * Internally, Game.ts uses CreatureStateVO instances.
 *
 * MVP supports:
 * - Base power and toughness
 * - +1/+1 counters
 * - Combat damage tracking
 * - Blocking relationships
 *
 * Explicitly excluded from MVP:
 * - First strike / Double strike (TODO: implement damage assignment order)
 * - Trample (TODO: implement excess damage to player/planeswalker)
 * - Deathtouch (TODO: implement any-amount-is-lethal rule)
 * - Damage prevention (TODO: implement prevention effects)
 * - Indestructible (TODO: implement in state-based actions)
 * - Static ability modifiers (TODO: implement continuous effects)
 * - Temporary "until end of turn" modifiers (TODO: implement duration tracking)
 * - Layer system (TODO: implement 7-layer system)
 *
 * @see Power/Toughness + Counters MVP Contract
 * @see Combat Resolution MVP Contract
 */
export type CreatureState = {
  isTapped: boolean
  isAttacking: boolean
  hasAttackedThisTurn: boolean
  hasSummoningSickness: boolean
  basePower: number
  baseToughness: number
  currentPower: number
  currentToughness: number
  counters: Record<string, number>
  damageMarkedThisTurn: number
  blockingCreatureId: string | null
  blockedBy: string | null
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

// Re-export ManaPool types for backward compatibility
export type {
  ManaColor,
  ManaPoolSnapshot,
} from "./valueobjects/ManaPool"

import {
  type ManaColor,
  ManaPool,
  InsufficientManaError as ManaPoolInsufficientManaError,
  type ManaPoolSnapshot,
} from "./valueobjects/ManaPool"
import { TurnState } from "./valueobjects/TurnState"

// PermanentOnBattlefield and TriggeredAbilityInfo types are now in TriggerEvaluation service

export class Game {
  public readonly id: string
  private readonly playersById: Map<string, Player> = new Map()
  private turnOrder: string[] = []

  // Turn state is managed by the TurnState value object
  private turnState: TurnState = TurnState.initial("")

  // Public getters for turn state (delegating to TurnState VO)
  get currentPlayerId(): string {
    return this.turnState.currentPlayerId
  }
  get currentStep(): GameSteps {
    return this.turnState.currentStep
  }
  get playedLandsThisTurn(): number {
    return this.turnState.playedLands
  }

  private lifecycleState: GameLifecycleState = GameLifecycleState.CREATED
  private _priorityPlayerId: string | null = null
  get priorityPlayerId(): string | null {
    return this._priorityPlayerId
  }
  private playerStates: Map<string, PlayerState> = new Map()
  private manaPools: Map<string, ManaPool> = new Map()
  private stack: TheStack = TheStack.empty()
  private playersWhoPassedPriority: Set<string> = new Set()
  private autoPassPlayers: Set<string> = new Set()
  private scheduledSteps: GameSteps[] = []
  private resumeStepAfterScheduled?: GameSteps = undefined
  private creatureStates: Map<string, CreatureStateVO> = new Map()

  constructor(id: string) {
    this.id = id
  }

  // ============================================================================
  // STATIC FACTORY & VALIDATORS
  // ============================================================================

  /**
   * Create a new Game instance in CREATED state.
   *
   * This is the entry point for the game lifecycle. After creation:
   * 1. Add players using game.addPlayer()
   * 2. Start the game using game.start()
   *
   * At this point:
   * - No players have joined
   * - No Magic rules are active
   * - No actions can be taken
   *
   * @param id - Unique identifier for the game
   * @returns A new Game instance in CREATED state
   */
  static create(id: string): Game {
    return new Game(id)
  }

  /**
   * Add a player to the game.
   *
   * Can only be called while the game is in CREATED state.
   * Initializes player state with empty zones and mana pool.
   *
   * @param player - The player to add
   * @throws CannotAddPlayerAfterStartError if game has already started
   * @throws DuplicatePlayerError if player is already in the game
   */
  addPlayer(player: Player): void {
    // Validate lifecycle state
    if (this.lifecycleState !== GameLifecycleState.CREATED) {
      throw new CannotAddPlayerAfterStartError()
    }

    // Check for duplicate player
    if (this.playersById.has(player.id)) {
      throw new DuplicatePlayerError(player.id)
    }

    // Add player to players map
    this.playersById.set(player.id, player)

    // Add player to turn order
    this.turnOrder.push(player.id)

    // Initialize player state with empty zone entities
    this.playerStates.set(player.id, {
      hand: Hand.empty(),
      battlefield: Battlefield.empty(),
      graveyard: Graveyard.empty(),
    })

    // Initialize mana pool
    this.manaPools.set(player.id, ManaPool.empty())
  }

  /**
   * Start the game and transition from CREATED → STARTED state.
   *
   * LIFECYCLE TRANSITION: This is the critical transition that activates
   * all Magic rules. After this method completes:
   * - All game actions become available via apply()
   * - Magic rules are enforced
   * - Turn order is established
   * - Priority is assigned
   * - Phases and steps begin
   *
   * Can only be called while the game is in CREATED state (after create()
   * and addPlayer() have been called).
   *
   * @param startingPlayerId - The ID of the player who goes first
   * @throws GameAlreadyStartedError if game has already started
   * @throws InvalidPlayerCountError if insufficient players (minimum 2)
   * @throws InvalidStartingPlayerError if starting player not in game
   */
  start(startingPlayerId: string): void {
    // Enforce lifecycle invariant: must be in CREATED state
    if (this.lifecycleState !== GameLifecycleState.CREATED) {
      throw new GameAlreadyStartedError()
    }

    // Validate minimum player count (domain rule)
    const playerCount = this.playersById.size
    if (playerCount < MIN_PLAYERS) {
      throw new InvalidPlayerCountError(playerCount)
    }

    // Validate starting player exists in the game
    if (!this.playersById.has(startingPlayerId)) {
      throw new InvalidStartingPlayerError(startingPlayerId)
    }

    // Initialize Magic rules state
    this.turnState = TurnState.initial(startingPlayerId)
    this._priorityPlayerId = startingPlayerId

    // LIFECYCLE TRANSITION: CREATED → STARTED
    // From this point forward, all Magic rules are active
    this.lifecycleState = GameLifecycleState.STARTED
  }

  // ============================================================================
  // PUBLIC API - HIGH LEVEL (Commands & Primary Queries)
  // ============================================================================

  /**
   * Apply a game action (play land, cast spell, advance step, etc.).
   *
   * LIFECYCLE INVARIANT: This method can only be called when the game is in
   * STARTED state. All Magic rules and game actions require an active game.
   *
   * Before calling apply():
   * 1. Game must be created with Game.create()
   * 2. Players must be added with game.addPlayer()
   * 3. Game must be started with game.start()
   *
   * @param action - The game action to execute
   * @throws GameNotStartedError if game is not in STARTED state
   */
  apply(action: Actions): void {
    // Enforce lifecycle invariant: game must be STARTED
    if (this.lifecycleState !== GameLifecycleState.STARTED) {
      throw new GameNotStartedError()
    }

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
        {
          type: "DECLARE_BLOCKER",
          playerId: P.string,
          blockerId: P.string,
          attackerId: P.string,
        },
        (action) => this.declareBlocker(action),
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

  /**
   * Gets all creature IDs with their states.
   * Used by domain services (e.g., StateBasedActions) to iterate over creatures.
   */
  getCreatureEntries(): ReadonlyArray<[string, CreatureStateVO]> {
    return Array.from(this.creatureStates.entries())
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

  /**
   * Get the current lifecycle state of the game.
   */
  getLifecycleState(): GameLifecycleState {
    return this.lifecycleState
  }

  /**
   * Get the current turn number.
   */
  getCurrentTurnNumber(): number {
    return this.turnState.turnNumber
  }

  /**
   * Get all players in the game with their basic info.
   * Returns players in registration order (before start) or turn order (after start).
   */
  getPlayers(): readonly { id: string; name: string }[] {
    const playerIds =
      this.turnOrder.length > 0
        ? this.turnOrder
        : Array.from(this.playersById.keys())
    return playerIds.map((id) => {
      const player = this.playersById.get(id)
      if (!player) {
        throw new PlayerNotFoundError(id)
      }
      return { id: player.id, name: player.name }
    })
  }

  /**
   * Check if a player is in auto-pass mode.
   *
   * A player in auto-pass mode will automatically pass priority
   * whenever they receive it. This is set by the END_TURN action.
   *
   * @param playerId - The ID of the player to check
   * @returns true if the player is in auto-pass mode
   */
  isPlayerInAutoPass(playerId: string): boolean {
    return this.autoPassPlayers.has(playerId)
  }

  getCreatureState(creatureId: string): CreatureState {
    const state = this.getCreatureStateOrThrow(creatureId)
    return state.toExport()
  }

  /**
   * Export the complete game state as a plain data structure.
   * Delegates to GameStateExporter service.
   */
  exportState(): GameStateExport {
    return exportGameState(this.asExportContext())
  }

  /**
   * Creates the export context interface for GameStateExporter.
   */
  private asExportContext(): ExportableGameContext {
    return {
      id: this.id,
      currentPlayerId: this.currentPlayerId,
      currentStep: this.currentStep,
      priorityPlayerId: this._priorityPlayerId,
      turnNumber: this.turnState.turnNumber,
      playedLands: this.turnState.playedLands,
      turnOrder: this.turnOrder,
      scheduledSteps: this.scheduledSteps,
      resumeStepAfterScheduled: this.resumeStepAfterScheduled,
      getPlayer: (playerId: string) => this.playersById.get(playerId),
      getPlayerState: (playerId: string) => this.getPlayerState(playerId),
      getManaPool: (playerId: string) =>
        this.manaPools.get(playerId) ?? ManaPool.empty(),
      getCreatureState: (instanceId: string) =>
        this.creatureStates.get(instanceId),
      getStackItems: () => this.stack.items,
      isPlaneswalker: (card: CardInstance) => this.isPlaneswalker(card),
      findCardOnBattlefields: (instanceId: string) =>
        this.findCardOnBattlefields(instanceId),
    }
  }

  /**
   * Find a card on any player's battlefield by instance ID.
   */
  private findCardOnBattlefields(instanceId: string): CardInstance | undefined {
    for (const playerState of this.playerStates.values()) {
      const card = playerState.battlefield.cards.find(
        (c) => c.instanceId === instanceId,
      )
      if (card) return card
    }
    return undefined
  }

  /**
   * Creates the combat validation context interface for CombatManager.
   */
  private asCombatValidationContext(): CombatValidationContext {
    return {
      currentStep: this.currentStep,
      currentPlayerId: this.currentPlayerId,
      getOpponentOf: (playerId: string) => this.getOpponentOf(playerId),
      getBattlefieldCards: (playerId: string) =>
        this.getPlayerState(playerId).battlefield.cards,
      isCreature: (card: CardInstance) => this.isCreature(card),
      hasStaticAbility: (card: CardInstance, ability) =>
        this.hasStaticAbility(card, ability),
      getCreatureState: (instanceId: string) =>
        this.creatureStates.get(instanceId),
    }
  }

  // ============================================================================
  // CREATURE STATS API - Power/Toughness + Counters (MVP)
  // ============================================================================

  /**
   * Get the base power of a creature.
   *
   * @param creatureId - The creature's instance ID
   * @returns The base power value (before counters or modifiers)
   */
  getBasePower(creatureId: string): number {
    const state = this.getCreatureStateOrThrow(creatureId)
    return state.basePower
  }

  /**
   * Get the base toughness of a creature.
   *
   * @param creatureId - The creature's instance ID
   * @returns The base toughness value (before counters or modifiers)
   */
  getBaseToughness(creatureId: string): number {
    const state = this.getCreatureStateOrThrow(creatureId)
    return state.baseToughness
  }

  /**
   * Get the number of counters of a specific type on a creature.
   *
   * @param creatureId - The creature's instance ID
   * @param counterType - The type of counter to query
   * @returns The number of counters (0 if none exist)
   */
  getCounters(creatureId: string, counterType: CounterType): number {
    const state = this.getCreatureStateOrThrow(creatureId)
    return state.getCounters(counterType)
  }

  /**
   * Add counters to a creature.
   *
   * @param creatureId - The creature's instance ID
   * @param counterType - The type of counter to add
   * @param amount - The number of counters to add (must be > 0)
   * @throws InvalidCounterAmountError if amount is not positive
   */
  addCounters(
    creatureId: string,
    counterType: CounterType,
    amount: number,
  ): void {
    this.assertValidCounterAmount(amount)

    const state = this.getCreatureStateOrThrow(creatureId)
    this.creatureStates.set(creatureId, state.addCounters(counterType, amount))
  }

  /**
   * Remove counters from a creature.
   *
   * @param creatureId - The creature's instance ID
   * @param counterType - The type of counter to remove
   * @param amount - The number of counters to remove (must be > 0)
   * @throws InvalidCounterAmountError if amount is not positive
   *
   * Note: Counter count will not go below 0 (clamped).
   */
  removeCounters(
    creatureId: string,
    counterType: CounterType,
    amount: number,
  ): void {
    this.assertValidCounterAmount(amount)

    const state = this.getCreatureStateOrThrow(creatureId)
    this.creatureStates.set(
      creatureId,
      state.removeCounters(counterType, amount),
    )
  }

  /**
   * Calculate the current power of a creature.
   *
   * Current power = base power + +1/+1 counters
   *
   * MVP calculation only includes:
   * - Base power
   * - +1/+1 counters
   *
   * Explicitly excluded from MVP:
   * - Static abilities (TODO: implement continuous effects)
   * - Temporary modifiers (TODO: implement "until end of turn" effects)
   * - Layer system (TODO: implement 7-layer system)
   *
   * @param creatureId - The creature's instance ID
   * @returns The current power value
   */
  getCurrentPower(creatureId: string): number {
    const state = this.getCreatureStateOrThrow(creatureId)
    return state.getCurrentPower()
  }

  /**
   * Calculate the current toughness of a creature.
   *
   * Current toughness = base toughness + +1/+1 counters
   *
   * MVP calculation only includes:
   * - Base toughness
   * - +1/+1 counters
   *
   * Explicitly excluded from MVP:
   * - Damage tracking (TODO: implement damage model)
   * - Static abilities (TODO: implement continuous effects)
   * - Temporary modifiers (TODO: implement "until end of turn" effects)
   * - Layer system (TODO: implement 7-layer system)
   *
   * @param creatureId - The creature's instance ID
   * @returns The current toughness value
   */
  getCurrentToughness(creatureId: string): number {
    const state = this.getCreatureStateOrThrow(creatureId)
    return state.getCurrentToughness()
  }

  // ============================================================================
  // END CREATURE STATS API
  // ============================================================================

  getManaPool(playerId: string): ManaPoolSnapshot {
    const pool = this.manaPools.get(playerId)
    if (!pool) {
      throw new PlayerNotFoundError(playerId)
    }
    return pool.toSnapshot()
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
    this.creatureStates.set(permanentId, state.withTapped(true))
  }

  untapPermanent(permanentId: string): void {
    const state = this.getCreatureStateOrThrow(permanentId)
    this.creatureStates.set(permanentId, state.withTapped(false))
  }

  initializeCreatureStateIfNeeded(card: CardInstance): void {
    if (this.isCreature(card)) {
      this.creatureStates.set(
        card.instanceId,
        CreatureStateVO.forCreature(card),
      )
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

    if (!this.hasPlayer(playerId)) {
      throw new PlayerNotFoundError(playerId)
    }

    const pool = this.manaPools.get(playerId) ?? ManaPool.empty()
    this.manaPools.set(playerId, pool.add(color, amount))
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

    if (!this.hasPlayer(playerId)) {
      throw new PlayerNotFoundError(playerId)
    }

    const pool = this.manaPools.get(playerId) ?? ManaPool.empty()

    try {
      this.manaPools.set(playerId, pool.spend(color, amount))
    } catch (error) {
      if (error instanceof ManaPoolInsufficientManaError) {
        // Re-throw as Game's InsufficientManaError for backward compatibility
        throw new InsufficientManaError(
          playerId,
          color,
          amount,
          pool.get(color),
        )
      }
      throw error
    }
  }

  /**
   * Clear all mana from a specific player's mana pool.
   *
   * @param playerId - The player whose mana pool to clear
   * @throws PlayerNotFoundError if player doesn't exist
   */
  clearManaPool(playerId: string): void {
    if (!this.hasPlayer(playerId)) {
      throw new PlayerNotFoundError(playerId)
    }

    this.manaPools.set(playerId, ManaPool.empty())
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
    for (const playerId of this.manaPools.keys()) {
      this.manaPools.set(playerId, ManaPool.empty())
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
    _reason: GraveyardReason,
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

  /**
   * Handle END_TURN action - records intent to auto-pass priority.
   *
   * END_TURN is a player shortcut, NOT a rules action. It expresses:
   * "I intend to keep passing priority until the turn naturally ends."
   *
   * This method:
   * - Records the player's intent to auto-pass
   * - Triggers auto-pass processing
   *
   * This method does NOT:
   * - Directly advance steps
   * - Execute rules
   * - Bypass priority windows
   *
   * The engine progresses through priority resolution only.
   */
  private endTurn(action: EndTurn): void {
    this.assertIsCurrentPlayer(action.playerId, "END_TURN")

    if (this.currentStep === Step.CLEANUP) {
      throw new InvalidEndTurnError()
    }

    // Record intent only - do NOT execute rules
    this.autoPassPlayers.add(action.playerId)

    // Trigger auto-pass processing
    this.processAutoPass()
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

    this.turnState = this.turnState.withLandPlayed()
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
      this.assignPriorityTo(opponentId)
    }
  }

  private declareAttacker(action: DeclareAttacker): void {
    this.assertIsCurrentPlayer(action.playerId, "DECLARE_ATTACKER")

    // Validate and get state changes from CombatManager
    const result = validateDeclareAttacker(
      this.asCombatValidationContext(),
      action.playerId,
      action.creatureId,
    )

    // Apply state changes
    this.creatureStates.set(action.creatureId, result.newCreatureState)

    // Emit creature declared attacker event and evaluate triggers
    this.evaluateTriggers({
      type: GameEventTypes.CREATURE_DECLARED_ATTACKER,
      creature: result.creature,
      controllerId: action.playerId,
    })
  }

  private declareBlocker(action: DeclareBlocker): void {
    // Validate and get state changes from CombatManager
    const result = validateDeclareBlocker(
      this.asCombatValidationContext(),
      action.playerId,
      action.blockerId,
      action.attackerId,
    )

    // Apply state changes
    this.creatureStates.set(action.blockerId, result.newBlockerState)
    this.creatureStates.set(action.attackerId, result.newAttackerState)

    // TODO: Emit CREATURE_DECLARED_BLOCKER event when needed
    // Currently no triggers depend on blocking, so this is deferred
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
      let creatureState = this.creatureStates.get(permanentId)

      if (creatureState) {
        // It's a creature - check if already tapped
        if (creatureState.isTapped) {
          throw new CannotPayActivationCostError(
            permanentId,
            "permanent is already tapped",
          )
        }

        // Find the creature card to check for Haste
        // Search all battlefields for the permanent
        let permanent: CardInstance | undefined
        for (const playerState of this.playerStates.values()) {
          permanent = playerState.battlefield.cards.find(
            (card) => card.instanceId === permanentId,
          )
          if (permanent) break
        }

        // Check for summoning sickness (unless has Haste)
        if (
          creatureState.hasSummoningSickness &&
          permanent &&
          !this.hasStaticAbility(permanent, StaticAbilities.HASTE)
        ) {
          throw new CreatureHasSummoningSicknessError(permanentId)
        }

        // Tap the creature
        creatureState = creatureState.withTapped(true)
        this.creatureStates.set(permanentId, creatureState)
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
      this.playersWhoPassedPriority.clear()
      this.assignPriorityTo(this.currentPlayerId)
    }
  }

  private setCurrentStep(nextStep: GameSteps): void {
    this.turnState = this.turnState.withStep(nextStep)
    this.onEnterStep(nextStep)
  }

  private onEnterStep(step: GameSteps): void {
    // Clear auto-pass intent at the start of a new turn
    // This ensures END_TURN only applies to the intended turn
    if (step === Step.UNTAP) {
      this.autoPassPlayers.clear()
      this.autoUntapForCurrentPlayer()
    }

    // Combat damage resolution at COMBAT_DAMAGE step
    if (step === Step.COMBAT_DAMAGE) {
      this.resolveCombatDamage()
      this.performStateBasedActions()
    }

    // Clear mana pools when entering CLEANUP step (MVP behavior)
    // TODO: In real Magic, mana empties at the end of each step and phase.
    // TODO: For MVP we clear at CLEANUP only to keep behavior deterministic.
    if (step === Step.CLEANUP) {
      this.clearAllManaPools()
      this.clearDamageOnAllCreatures()
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
    // Also clear summoning sickness for creatures controlled by the current player
    for (const card of playerState.battlefield.cards) {
      if (this.isCreature(card)) {
        const creatureState = this.creatureStates.get(card.instanceId)
        if (creatureState) {
          this.creatureStates.set(
            card.instanceId,
            creatureState.withTapped(false).withSummoningSickness(false),
          )
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
    const nextPlayerId = this.turnOrder[nextIndex]

    // Use TurnState.forNewTurn() to handle player change and reset lands
    this.turnState = this.turnState.forNewTurn(nextPlayerId)

    // Increment turn number when we wrap around to the first player
    if (nextIndex === 0) {
      this.turnState = this.turnState.withIncrementedTurnNumber()
    }

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
    this.assignPriorityTo(this.currentPlayerId)
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
    this.playersWhoPassedPriority.clear()
    this.assignPriorityTo(opponentId)
  }

  // ============================================================================
  // AUTO-PASS SYSTEM
  // ============================================================================

  /**
   * Assigns priority to a player and checks for auto-pass.
   *
   * This is the central method for priority assignment. All priority changes
   * (except initial game start) should go through this method to ensure
   * auto-pass is properly handled.
   *
   * If the player receiving priority is in auto-pass mode:
   * - With non-empty stack: they automatically pass priority
   * - With empty stack (and they're active player): they auto-advance through steps
   */
  private assignPriorityTo(playerId: string): void {
    this._priorityPlayerId = playerId

    // Reactive auto-pass: if player is in auto-pass mode
    if (this.autoPassPlayers.has(playerId)) {
      if (this.hasSpellsOnStack()) {
        // Auto-pass priority when stack is non-empty
        this.performInternalPass(playerId)
      } else if (playerId === this.currentPlayerId) {
        // Auto-advance steps when stack is empty and they're the active player
        this.processAutoPass()
      }
    }
  }

  /**
   * Performs an internal priority pass for auto-pass processing.
   *
   * This is the same logic as passPriority but without the player action
   * validation (since this is an internal engine action, not a player action).
   */
  private performInternalPass(playerId: string): void {
    this.playersWhoPassedPriority.add(playerId)

    if (this.bothPlayersHavePassed()) {
      this.resolveTopOfStack()
    } else {
      const opponentId = this.getOpponentOf(playerId)
      this.assignPriorityTo(opponentId)
    }
  }

  /**
   * Processes auto-pass for step advancement.
   *
   * When the active player is in auto-pass mode and the stack is empty,
   * this method advances through steps until:
   * - The turn ends (and new turn starts)
   * - Something is added to the stack
   * - The active player is no longer in auto-pass mode
   *
   * Safety: Limited to MAX_ITERATIONS to prevent infinite loops.
   */
  private processAutoPass(): void {
    let iterations = 0
    const MAX_ITERATIONS = 100 // Safety limit

    while (iterations < MAX_ITERATIONS) {
      iterations++

      // Stop if active player is not in auto-pass
      if (!this.autoPassPlayers.has(this.currentPlayerId)) {
        break
      }

      // Stop if there's something on the stack (priority system takes over)
      if (this.hasSpellsOnStack()) {
        break
      }

      // Advance through cleanup to next turn
      if (this.currentStep === Step.CLEANUP) {
        this.performStepAdvance()
        break // Turn has ended
      }

      this.performStepAdvance()
    }
  }

  private clearAttackingState(): void {
    for (const [creatureId, state] of this.creatureStates.entries()) {
      this.creatureStates.set(creatureId, state.clearCombatState())
    }
  }

  private resetCreatureStatesForNewTurn(): void {
    for (const [creatureId, state] of this.creatureStates.entries()) {
      this.creatureStates.set(
        creatureId,
        state.clearCombatState().withHasAttackedThisTurn(false),
      )
    }
  }

  /**
   * Resolve combat damage during the COMBAT_DAMAGE step.
   *
   * This handles:
   * 1. Damage assignment from attackers to blockers and vice versa
   * 2. Damage from unblocked attackers to defending player
   *
   * Implementation follows the "store then apply" pattern:
   * - First pass: collect all damage assignments
   * - Second pass: apply all damage simultaneously
   *
   * This ensures true simultaneity and robustness if creatures disappear
   * before damage resolution (e.g., via instant-speed removal).
   *
   * MVP Assumptions:
   * - Two-player game
   * - All attackers target the defending player (no planeswalker targeting)
   * - Defending player is the opponent of the active player
   *
   * MVP Limitations (explicitly excluded):
   * - First strike / Double strike (TODO: implement damage assignment order)
   * - Trample (TODO: implement excess damage to player/planeswalker)
   * - Deathtouch (TODO: implement any-amount-is-lethal rule)
   * - Damage redirection (TODO: implement redirection to planeswalkers)
   * - Damage prevention (TODO: implement prevention effects)
   * - Multiple blockers per attacker (TODO: implement after MVP)
   */
  private resolveCombatDamage(): void {
    // Calculate damage assignments using domain service
    const damageAssignments = CombatResolution.calculateDamageAssignments(this)

    // Apply all damage simultaneously
    for (const assignment of damageAssignments) {
      if (assignment.isPlayer) {
        this.dealDamageToPlayer(assignment.targetId, assignment.amount)
      } else {
        this.markDamageOnCreature(assignment.targetId, assignment.amount)
      }
    }
  }

  /**
   * Safely get a creature's power, returning null if the creature no longer exists.
   *
   * This handles the case where a creature was removed before combat damage resolution
   * (e.g., via instant-speed removal or ability activation).
   * Used by domain services (e.g., CombatResolution).
   *
   * @param creatureId - The instance ID of the creature
   * @returns The creature's current power, or null if it no longer exists
   */
  getCreaturePowerSafe(creatureId: string): number | null {
    const state = this.creatureStates.get(creatureId)
    if (!state) return null

    // Verify the creature is actually on the battlefield
    const stillExists = this.findCreatureOnAnyBattlefield(creatureId)
    if (!stillExists) return null

    return this.getCurrentPower(creatureId)
  }

  /**
   * Check if a creature exists on any player's battlefield.
   *
   * @param creatureId - The instance ID of the creature
   * @returns true if the creature exists on a battlefield, false otherwise
   */
  private findCreatureOnAnyBattlefield(creatureId: string): boolean {
    for (const playerState of this.playerStates.values()) {
      const found = playerState.battlefield.cards.some(
        (card) => card.instanceId === creatureId,
      )
      if (found) return true
    }
    return false
  }

  /**
   * Mark damage on a creature without destroying it yet.
   * Damage accumulates in damageMarkedThisTurn.
   */
  private markDamageOnCreature(creatureId: string, damage: number): void {
    const state = this.creatureStates.get(creatureId)
    if (!state) return // Creature may have left battlefield

    const newDamage = state.damageMarkedThisTurn + damage
    this.creatureStates.set(creatureId, state.withDamage(newDamage))
  }

  /**
   * Deal damage to a player by reducing their life total.
   *
   * MVP: No loss condition checking (player can go below 0).
   * TODO: Implement state-based action for player losing at 0 life
   */
  private dealDamageToPlayer(playerId: string, damage: number): void {
    const player = this.playersById.get(playerId)
    if (!player) return

    player.adjustLifeTotal(-damage)
  }

  /**
   * Clear all damage marked on creatures.
   * This happens at the CLEANUP step.
   */
  private clearDamageOnAllCreatures(): void {
    for (const [creatureId, state] of this.creatureStates.entries()) {
      this.creatureStates.set(creatureId, state.clearDamage())
    }
  }

  /**
   * Perform state-based actions.
   *
   * Currently implements:
   * - Destroy creatures with lethal damage
   * - Destroy creatures with 0 or less toughness
   *
   * MVP Limitations:
   * - Indestructible not supported (TODO: implement indestructible check)
   * - Player loss condition not checked (TODO: implement player loss at 0 life)
   * - Legend rule not implemented (TODO: implement legendary uniqueness check)
   *
   * @see StateBasedActions service for detection logic
   */
  private performStateBasedActions(): void {
    const creaturesToDestroy = StateBasedActions.findCreaturesToDestroy(this)

    for (const creatureId of creaturesToDestroy) {
      this.movePermanentToGraveyard(creatureId, GraveyardReason.STATE_BASED)
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
    // Use TriggerEvaluation service to find matching triggers
    const triggersToExecute = TriggerEvaluation.findMatchingTriggers(
      this,
      event,
    )
    this.executeTriggeredAbilities(triggersToExecute)
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
    triggeredAbilities: TriggeredAbilityInfo[],
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
    return playerId === this._priorityPlayerId
  }

  private canPlayLand(playerId: string): boolean {
    return (
      playerId === this.currentPlayerId &&
      !this.hasPlayedLandThisTurn() &&
      this.isMainPhase()
    )
  }

  private hasPlayedLandThisTurn(): boolean {
    return this.turnState.hasPlayedLand()
  }

  private isMainPhase(): boolean {
    return this.turnState.isMainPhase()
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

  private isPlaneswalker(card: CardInstance): boolean {
    return card.definition.types.includes("PLANESWALKER")
  }

  /**
   * Checks if a permanent has a specific static ability keyword.
   *
   * MVP static abilities are consultative; no layers yet.
   * These keywords only affect rule checks and validations.
   *
   * @param card - The card instance to check
   * @param ability - The static ability to check for
   * @returns true if the card has the static ability, false otherwise
   */
  private hasStaticAbility(
    card: CardInstance,
    ability: StaticAbility,
  ): boolean {
    return card.definition.staticAbilities?.includes(ability) ?? false
  }

  // ============================================================================
  // PRIVATE - ASSERTIONS & HELPERS (Lowest-Level Utilities)
  // ============================================================================

  private getCreatureStateOrThrow(creatureId: string): CreatureStateVO {
    const state = this.creatureStates.get(creatureId)
    if (!state) {
      throw new PermanentNotFoundError(creatureId)
    }
    return state
  }

  private assertValidCounterAmount(amount: number): void {
    if (amount <= 0) {
      throw new InvalidCounterAmountError(amount)
    }
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

  /**
   * Gets the opponent of a player.
   * In a 2-player game, returns the other player.
   * Used by domain services (e.g., CombatResolution) for damage targeting.
   */
  getOpponentOf(playerId: string): string {
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
