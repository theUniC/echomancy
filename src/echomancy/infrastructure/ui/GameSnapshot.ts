/**
 * GameSnapshot - UI Layer Contract
 *
 * This module defines the UI-facing game snapshot that provides:
 * - Player-relative visibility filtering
 * - UI-friendly data structures
 * - Read-only representation
 * - Convenience fields for rendering
 *
 * CRITICAL DESIGN PRINCIPLES:
 * - Lives OUTSIDE the engine
 * - Derived entirely from GameStateExport
 * - Contains NO rules logic
 * - Is immutable and reconstructible
 * - Applies visibility rules (hidden information filtering)
 * - Is player-specific (created FOR a specific viewer)
 *
 * DO NOT:
 * - Add rules logic or validation
 * - Mutate engine state
 * - Infer legality of actions
 * - Add speculative previews
 * - Store engine references
 *
 * @see GameStateExport - the raw, unfiltered engine output
 */

import type {
  CardType,
  StaticAbility,
} from "../domainmodel/cards/CardDefinition"
import type {
  CardInstanceExport,
  CounterTypeExport,
  GameStateExport,
  ManaPoolExport,
  StackItemExport,
} from "../domainmodel/game/GameStateExport"
import type { GameSteps } from "../domainmodel/game/Steps"

/**
 * Combat state for a card in the UI.
 * Provides a flattened, display-ready view of combat participation.
 *
 * Note: Both blockedBy and blocking are arrays per spec, even though
 * MVP only supports 1-to-1 blocking. This ensures UI contract compliance.
 */
export type CombatStateSnapshot = {
  isAttacking: boolean
  isBlocking: boolean
  blockedBy: readonly string[] // Instance IDs of blockers
  blocking: readonly string[] // Instance IDs of creatures being blocked
}

/**
 * Combat summary showing current combat statistics.
 * Null if not in combat.
 */
export type CombatSummary = {
  attackerCount: number
  blockerCount: number
}

/**
 * UI-friendly card representation.
 * Flattened and display-ready, with no engine coupling.
 *
 * NOTE: This includes the card name for UI convenience.
 * The name must be resolved from cardDefinitionId via a card registry.
 */
export type CardSnapshot = {
  instanceId: string
  name: string // Resolved from cardDefinitionId
  types: readonly CardType[]
  subtypes: readonly string[] // Currently empty in MVP

  controllerId: string
  ownerId: string

  // Creature-specific state (null if not a creature)
  tapped: boolean | null
  counters: Readonly<Record<CounterTypeExport, number>> | null
  damageMarked: number | null

  power: number | null
  toughness: number | null

  staticKeywords: readonly StaticAbility[]

  combatState: CombatStateSnapshot | null
}

/**
 * Public game state visible to ALL players.
 * No hidden information.
 */
export type PublicGameState = {
  turnNumber: number
  currentPlayerId: string
  activePlayerId: string // Same as currentPlayerId in current MVP
  priorityPlayerId: string | null

  currentPhase: string // Derived from currentStep
  currentStep: GameSteps

  combatSummary: CombatSummary | null

  stackSize: number
}

/**
 * Private player state for the VIEWER.
 * Full visibility of all zones including hand.
 */
export type PrivatePlayerState = {
  playerId: string
  lifeTotal: number
  poisonCounters: number // Always 0 in current MVP
  manaPool: ManaPoolExport

  hand: readonly CardSnapshot[]
  battlefield: readonly CardSnapshot[]
  graveyard: readonly CardSnapshot[]
  exile: readonly CardSnapshot[] // Empty in current MVP
}

/**
 * Opponent state with hidden information applied.
 * Hand is NOT visible, only hand SIZE.
 */
export type OpponentState = {
  playerId: string
  lifeTotal: number
  poisonCounters: number // Always 0 in current MVP
  manaPool: ManaPoolExport | null // Can be hidden or aggregated

  handSize: number // Number of cards, not the cards themselves
  battlefield: readonly CardSnapshot[]
  graveyard: readonly CardSnapshot[]
  exile: readonly CardSnapshot[] // Empty in current MVP
}

/**
 * Stack item snapshot with human-readable information.
 * Targets are resolved into descriptions.
 */
export type StackItemSnapshot = {
  sourceCardName: string
  controllerId: string
  kind: "SPELL" | "ACTIVATED_ABILITY" | "TRIGGERED_ABILITY"
  targetDescriptions: readonly string[]
}

/**
 * Stack snapshot ordered from top to bottom.
 * Top of stack is index 0.
 */
export type StackSnapshot = {
  items: readonly StackItemSnapshot[]
}

/**
 * Optional UI hints derived from engine output.
 * These MUST NOT encode rules logic.
 * They are purely convenience flags for UI rendering.
 *
 * Note: canPlayLand was removed because determining land legality
 * requires rules logic. UI must ask engine for "allowed actions" instead.
 */
export type UIHints = {
  canPassPriority: boolean
  highlightedAttackers: readonly string[] // Instance IDs
  highlightedBlockers: readonly string[] // Instance IDs
}

/**
 * Complete game snapshot for a specific player.
 * This is the main UI contract.
 *
 * INVARIANTS:
 * - Created FOR a specific viewer (viewerPlayerId)
 * - Immutable after creation
 * - Reconstructible from GameStateExport at any time
 * - Contains no engine references
 * - Applies visibility rules correctly
 */
export type GameSnapshot = {
  viewerPlayerId: string

  publicGameState: PublicGameState
  privatePlayerState: PrivatePlayerState
  opponentStates: readonly OpponentState[]

  visibleStack: StackSnapshot

  uiHints: UIHints | null
}

/**
 * Card registry interface for resolving card names.
 * The UI must provide this to resolve cardDefinitionId → name.
 */
export type CardRegistry = {
  getCardName(cardDefinitionId: string): string
}

/**
 * Creates a GameSnapshot for a specific viewer from GameStateExport.
 *
 * This is the primary transformation function between engine and UI.
 *
 * @param exportedState - The complete, unfiltered game state export
 * @param viewerPlayerId - The player ID for whom this snapshot is created
 * @param cardRegistry - Registry to resolve card definition IDs to names
 * @returns A player-relative, UI-friendly game snapshot
 *
 * @throws Error if viewerPlayerId is not in the game
 */
export function createGameSnapshot(
  exportedState: GameStateExport,
  viewerPlayerId: string,
  cardRegistry: CardRegistry,
): GameSnapshot {
  // Validate viewer is in the game
  if (!(viewerPlayerId in exportedState.players)) {
    throw new Error(`Player ${viewerPlayerId} not found in game state`)
  }

  const viewerState = exportedState.players[viewerPlayerId]
  if (!viewerState) {
    throw new Error(`Player state not found for ${viewerPlayerId}`)
  }

  // Build card snapshots with full visibility for viewer's zones
  const viewerHand = mapZoneToSnapshots(
    viewerState.zones.hand.cards,
    cardRegistry,
  )
  const viewerBattlefield = mapZoneToSnapshots(
    viewerState.zones.battlefield.cards,
    cardRegistry,
  )
  const viewerGraveyard = mapZoneToSnapshots(
    viewerState.zones.graveyard.cards,
    cardRegistry,
  )

  // Build opponent states with hidden information
  const opponentStates: OpponentState[] = []
  for (const [playerId, playerState] of Object.entries(exportedState.players)) {
    if (playerId === viewerPlayerId) continue

    opponentStates.push({
      playerId,
      lifeTotal: playerState.lifeTotal,
      poisonCounters: 0, // MVP: not yet implemented
      manaPool: playerState.manaPool, // Could be hidden in future

      handSize: playerState.zones.hand.cards.length,
      battlefield: mapZoneToSnapshots(
        playerState.zones.battlefield.cards,
        cardRegistry,
      ),
      graveyard: mapZoneToSnapshots(
        playerState.zones.graveyard.cards,
        cardRegistry,
      ),
      exile: [], // MVP: not yet implemented
    })
  }

  // Determine current phase from step
  const currentPhase = derivePhaseFromStep(exportedState.currentStep)

  // Build combat summary if in combat
  const combatSummary = createCombatSummary(exportedState)

  // Build public game state
  const publicGameState: PublicGameState = {
    turnNumber: exportedState.currentTurnNumber,
    currentPlayerId: exportedState.currentPlayerId,
    activePlayerId: exportedState.currentPlayerId, // Same in current MVP
    priorityPlayerId: exportedState.priorityPlayerId,
    currentPhase,
    currentStep: exportedState.currentStep,
    combatSummary,
    stackSize: exportedState.stack.length,
  }

  // Build private player state
  const privatePlayerState: PrivatePlayerState = {
    playerId: viewerPlayerId,
    lifeTotal: viewerState.lifeTotal,
    poisonCounters: 0, // MVP: not yet implemented
    manaPool: viewerState.manaPool,
    hand: viewerHand,
    battlefield: viewerBattlefield,
    graveyard: viewerGraveyard,
    exile: [], // MVP: not yet implemented
  }

  // Build stack snapshot
  const visibleStack = createStackSnapshot(
    exportedState.stack,
    cardRegistry,
    exportedState,
  )

  // Build UI hints (basic implementation)
  const uiHints = createUIHints(exportedState, viewerPlayerId)

  return {
    viewerPlayerId,
    publicGameState,
    privatePlayerState,
    opponentStates,
    visibleStack,
    uiHints,
  }
}

/**
 * Maps a zone's cards to card snapshots.
 * Helper to reduce duplication when processing zones.
 *
 * @param cards - Array of exported card instances
 * @param cardRegistry - Registry to resolve card names
 * @returns Array of card snapshots
 */
function mapZoneToSnapshots(
  cards: readonly CardInstanceExport[],
  cardRegistry: CardRegistry,
): readonly CardSnapshot[] {
  return cards.map((card) => createCardSnapshot(card, cardRegistry))
}

/**
 * Creates a UI-friendly card snapshot from an exported card instance.
 *
 * @param card - The exported card instance
 * @param cardRegistry - Registry to resolve card names
 * @returns A flattened, display-ready card snapshot
 */
function createCardSnapshot(
  card: CardInstanceExport,
  cardRegistry: CardRegistry,
): CardSnapshot {
  const name = cardRegistry.getCardName(card.cardDefinitionId)

  // Extract creature-specific state if present
  const creatureState = card.creatureState
  const tapped = creatureState?.isTapped ?? null
  const counters = creatureState?.counters ?? null
  const damageMarked = creatureState?.damageMarkedThisTurn ?? null
  const power = creatureState?.power ?? card.power ?? null
  const toughness = creatureState?.toughness ?? card.toughness ?? null

  // Build combat state if creature is in combat
  const combatState = creatureState
    ? {
        isAttacking: creatureState.isAttacking,
        isBlocking: creatureState.blockingCreatureId !== null,
        blockedBy: creatureState.blockedBy ? [creatureState.blockedBy] : [],
        blocking: creatureState.blockingCreatureId
          ? [creatureState.blockingCreatureId]
          : [],
      }
    : null

  return {
    instanceId: card.instanceId,
    name,
    types: card.types,
    subtypes: [], // MVP: not yet implemented
    controllerId: card.controllerId,
    ownerId: card.ownerId,
    tapped,
    counters,
    damageMarked,
    power,
    toughness,
    staticKeywords: card.staticAbilities ?? [],
    combatState,
  }
}

/**
 * Creates a stack snapshot with resolved target descriptions.
 *
 * IMPORTANT: Engine stores stack with top at END of array (uses push/pop).
 * Spec requires TOP→BOTTOM ordering where index 0 is top.
 * Therefore we must REVERSE the engine's array.
 *
 * @param stack - The exported stack items (bottom→top from engine)
 * @param cardRegistry - Registry to resolve card names
 * @param exportedState - Full exported state for resolving targets
 * @returns A stack snapshot with human-readable information (top→bottom)
 */
function createStackSnapshot(
  stack: readonly StackItemExport[],
  cardRegistry: CardRegistry,
  exportedState: GameStateExport,
): StackSnapshot {
  // Reverse to match spec: index 0 = TOP of stack
  const items = [...stack].reverse().map((item) => {
    // Use the sourceCardDefinitionId directly to resolve the name
    const sourceCardName = cardRegistry.getCardName(item.sourceCardDefinitionId)

    // Resolve target descriptions
    const targetDescriptions = item.targets.map((targetId) => {
      const targetCard = findCardInExport(exportedState, targetId)
      if (targetCard) {
        return cardRegistry.getCardName(targetCard.cardDefinitionId)
      }
      // Could be a player
      if (targetId in exportedState.players) {
        return `Player ${targetId}`
      }
      return "Unknown target"
    })

    return {
      sourceCardName,
      controllerId: item.controllerId,
      kind: item.kind,
      targetDescriptions,
    }
  })

  return { items }
}

/**
 * Finds a card instance in the exported state by instance ID.
 * Searches all zones for all players.
 *
 * @param exportedState - The exported game state
 * @param instanceId - The card instance ID to find
 * @returns The card instance if found, null otherwise
 */
function findCardInExport(
  exportedState: GameStateExport,
  instanceId: string,
): CardInstanceExport | null {
  for (const playerState of Object.values(exportedState.players)) {
    // Search hand
    const inHand = playerState.zones.hand.cards.find(
      (c) => c.instanceId === instanceId,
    )
    if (inHand) return inHand

    // Search battlefield
    const inBattlefield = playerState.zones.battlefield.cards.find(
      (c) => c.instanceId === instanceId,
    )
    if (inBattlefield) return inBattlefield

    // Search graveyard
    const inGraveyard = playerState.zones.graveyard.cards.find(
      (c) => c.instanceId === instanceId,
    )
    if (inGraveyard) return inGraveyard

    // Search library if present
    if (playerState.zones.library) {
      const inLibrary = playerState.zones.library.cards.find(
        (c) => c.instanceId === instanceId,
      )
      if (inLibrary) return inLibrary
    }
  }

  return null
}

/**
 * Derives the current phase name from a game step.
 *
 * @param step - The current game step
 * @returns The phase name
 */
function derivePhaseFromStep(step: GameSteps): string {
  switch (step) {
    case "UNTAP":
    case "UPKEEP":
    case "DRAW":
      return "Beginning"
    case "FIRST_MAIN":
      return "Precombat Main"
    case "BEGINNING_OF_COMBAT":
    case "DECLARE_ATTACKERS":
    case "DECLARE_BLOCKERS":
    case "COMBAT_DAMAGE":
    case "END_OF_COMBAT":
      return "Combat"
    case "SECOND_MAIN":
      return "Postcombat Main"
    case "END_STEP":
    case "CLEANUP":
      return "Ending"
    default:
      return "Unknown"
  }
}

/**
 * Creates a combat summary from the exported state.
 * Returns null if not in combat or no combat is occurring.
 *
 * @param exportedState - The exported game state
 * @returns Combat summary or null
 */
function createCombatSummary(
  exportedState: GameStateExport,
): CombatSummary | null {
  const combatSteps: GameSteps[] = [
    "BEGINNING_OF_COMBAT",
    "DECLARE_ATTACKERS",
    "DECLARE_BLOCKERS",
    "COMBAT_DAMAGE",
    "END_OF_COMBAT",
  ]

  if (!combatSteps.includes(exportedState.currentStep)) {
    return null
  }

  let attackerCount = 0
  let blockerCount = 0

  for (const playerState of Object.values(exportedState.players)) {
    for (const card of playerState.zones.battlefield.cards) {
      if (card.creatureState?.isAttacking) {
        attackerCount++
      }
      if (card.creatureState?.blockingCreatureId) {
        blockerCount++
      }
    }
  }

  return { attackerCount, blockerCount }
}

/**
 * Creates UI hints from the exported state.
 * These are convenience flags for the UI.
 *
 * @param exportedState - The exported game state
 * @param viewerPlayerId - The viewer's player ID
 * @returns UI hints or null
 */
function createUIHints(
  exportedState: GameStateExport,
  viewerPlayerId: string,
): UIHints | null {
  const viewerState = exportedState.players[viewerPlayerId]
  if (!viewerState) return null

  // Can pass priority if viewer has priority
  const canPassPriority = exportedState.priorityPlayerId === viewerPlayerId

  // Highlight attacking/blocking creatures
  const highlightedAttackers: string[] = []
  const highlightedBlockers: string[] = []

  for (const playerState of Object.values(exportedState.players)) {
    for (const card of playerState.zones.battlefield.cards) {
      if (card.creatureState?.isAttacking) {
        highlightedAttackers.push(card.instanceId)
      }
      if (card.creatureState?.blockingCreatureId) {
        highlightedBlockers.push(card.instanceId)
      }
    }
  }

  return {
    canPassPriority,
    highlightedAttackers,
    highlightedBlockers,
  }
}
