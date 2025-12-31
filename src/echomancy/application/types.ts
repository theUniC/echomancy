/**
 * Re-exports domain types for use by API routes.
 *
 * API routes can only import from `application/` and `infrastructure/`.
 * This module provides access to domain types without violating that rule.
 */

export type { Actions } from "@/echomancy/domainmodel/game/GameActions"
export type { GameStateExport } from "@/echomancy/domainmodel/game/GameStateExport"
