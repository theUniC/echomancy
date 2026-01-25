/**
 * Card Catalog
 *
 * A collection of basic card definitions for game setup and testing.
 * These are placeholder cards with no effects - they serve as mechanical
 * objects for the game engine.
 *
 * MVP Scope:
 * - 5 basic lands (no mana abilities yet)
 * - 2 creatures (vanilla, no abilities)
 * - 3 spells (no effects)
 *
 * These cards are used by PrebuiltDecks to create deck configurations.
 *
 * @see PrebuiltDecks
 */

import type { CardDefinition } from "./CardDefinition"

/**
 * CardCatalog provides basic card definitions.
 * All cards are read-only singletons.
 */
export const CardCatalog = {
  // ============================================================================
  // BASIC LANDS
  // ============================================================================

  Forest: {
    id: "forest",
    name: "Forest",
    types: ["LAND"],
  } as const satisfies CardDefinition,

  Mountain: {
    id: "mountain",
    name: "Mountain",
    types: ["LAND"],
  } as const satisfies CardDefinition,

  Plains: {
    id: "plains",
    name: "Plains",
    types: ["LAND"],
  } as const satisfies CardDefinition,

  Island: {
    id: "island",
    name: "Island",
    types: ["LAND"],
  } as const satisfies CardDefinition,

  Swamp: {
    id: "swamp",
    name: "Swamp",
    types: ["LAND"],
  } as const satisfies CardDefinition,

  // ============================================================================
  // CREATURES
  // ============================================================================

  Bear: {
    id: "bear",
    name: "Bear",
    types: ["CREATURE"],
    power: 2,
    toughness: 2,
  } as const satisfies CardDefinition,

  EliteVanguard: {
    id: "elite-vanguard",
    name: "Elite Vanguard",
    types: ["CREATURE"],
    power: 2,
    toughness: 1,
  } as const satisfies CardDefinition,

  // ============================================================================
  // SPELLS
  // ============================================================================

  GiantGrowth: {
    id: "giant-growth",
    name: "Giant Growth",
    types: ["INSTANT"],
  } as const satisfies CardDefinition,

  LightningStrike: {
    id: "lightning-strike",
    name: "Lightning Strike",
    types: ["INSTANT"],
  } as const satisfies CardDefinition,

  Divination: {
    id: "divination",
    name: "Divination",
    types: ["SORCERY"],
  } as const satisfies CardDefinition,
} as const
