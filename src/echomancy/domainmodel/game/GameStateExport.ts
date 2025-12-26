/**
 * Game State Export - Core Contract (Pre-Snapshot)
 *
 * This module defines the contract for exporting the current game state from the core engine.
 *
 * IMPORTANT: This export is:
 * - Neutral (not UI-oriented)
 * - Complete (no hiding of information)
 * - Not filtered (includes hidden information like hands, libraries)
 * - Not player-specific
 * - Plain data only (no methods, no behavior)
 *
 * This is an intermediate representation between:
 * - Game (domain logic)
 * - GameSnapshot (UI / network / replay view - NOT implemented here)
 *
 * DO NOT:
 * - Hide information (hands, libraries, etc.)
 * - Add "allowed actions"
 * - Add UI helpers
 * - Filter by visibility
 * - Add validation logic
 *
 * @see Game State Export — Core Contract (Pre-Snapshot)
 */

import type { CardType, StaticAbility } from "../cards/CardDefinition"
import type { GameSteps } from "./Steps"

/**
 * Counter types supported by the game engine.
 * Exported as a plain object for serialization.
 */
export type CounterTypeExport = "PLUS_ONE_PLUS_ONE"

/**
 * Mana color representation for export.
 */
export type ManaColorExport = "W" | "U" | "B" | "R" | "G" | "C"

/**
 * Mana pool state for a player.
 * Represents the current mana available in each color.
 */
export type ManaPoolExport = {
  W: number
  U: number
  B: number
  R: number
  G: number
  C: number
}

/**
 * Creature-specific state export.
 * Only present for cards with type CREATURE.
 */
export type CreatureStateExport = {
  isTapped: boolean
  isAttacking: boolean
  hasAttackedThisTurn: boolean
  power: number
  toughness: number
  counters: Record<CounterTypeExport, number>
  damageMarkedThisTurn: number
  blockingCreatureId: string | null
  blockedBy: string | null
}

/**
 * Planeswalker-specific state export (MVP - placeholder only).
 * Reserved for future expansion.
 */
export type PlaneswalkerStateExport = Record<string, never>

/**
 * Card instance export representation.
 * Represents a specific card in the game with all its state.
 */
export type CardInstanceExport = {
  instanceId: string
  ownerId: string
  controllerId: string
  cardDefinitionId: string
  name: string
  types: readonly CardType[]
  staticAbilities?: readonly StaticAbility[]
  power?: number
  toughness?: number
  creatureState?: CreatureStateExport
  planeswalkerState?: PlaneswalkerStateExport
}

/**
 * Zone export representation.
 * Contains all cards in the zone, unfiltered.
 */
export type ZoneExport = {
  cards: readonly CardInstanceExport[]
}

/**
 * Stack item export representation.
 * Represents a spell or ability on the stack.
 */
export type StackItemExport = {
  kind: "SPELL" | "ACTIVATED_ABILITY" | "TRIGGERED_ABILITY"
  sourceCardInstanceId: string
  controllerId: string
  targets: readonly string[] // Target instance IDs
}

/**
 * Player state export representation.
 * Contains all zones and mana pool for a player.
 * INCLUDES HIDDEN INFORMATION (hand, library).
 */
export type PlayerStateExport = {
  lifeTotal: number
  manaPool: ManaPoolExport
  playedLandsThisTurn: number
  zones: {
    hand: ZoneExport
    battlefield: ZoneExport
    graveyard: ZoneExport
    library?: ZoneExport // Optional - may not be implemented in MVP
  }
}

/**
 * Complete game state export.
 * This is the top-level export structure.
 *
 * INVARIANTS:
 * - Every card instance referenced exists exactly once
 * - No derived or computed UI state
 * - No validation logic
 * - No mutation after creation
 */
export type GameStateExport = {
  gameId: string
  currentTurnNumber: number
  currentPlayerId: string
  currentStep: GameSteps
  priorityPlayerId: string | null
  turnOrder: readonly string[]
  players: Readonly<Record<string, PlayerStateExport>>
  stack: readonly StackItemExport[]
  scheduledSteps: readonly GameSteps[]
  resumeStepAfterScheduled?: GameSteps
}
