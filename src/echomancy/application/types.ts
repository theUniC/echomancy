/**
 * Re-exports domain types for use by API routes.
 *
 * EXCEPTION: API routes normally cannot import from domainmodel/.
 * These specific types are allowed as exceptions because they define
 * the API contract (input/output shapes) without exposing domain logic.
 *
 * Allowed re-exports:
 * - Actions: Type for incoming action requests
 * - GameStateExport: Type for state responses
 */

export type { Actions } from "@/echomancy/domainmodel/game/GameActions"
export type { GameStateExport } from "@/echomancy/domainmodel/game/GameStateExport"
