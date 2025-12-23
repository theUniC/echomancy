import type { Game } from "../game/Game"
import type { EffectContext } from "./EffectContext"

/**
 * Effect - The executable part of an ability
 *
 * When an ability resolves, its effect is what actually happens.
 * Examples: DrawCardsEffect, DamageEffect (future), CreateTokenEffect (future)
 *
 * IMPLEMENTATION RULES (CRITICAL):
 * MUST use Game methods for mutations:
 * - game.drawCards() for drawing
 * - game.enterBattlefield() for permanents (never array.push)
 * - game.dealDamage() for damage (future)
 * - context.controllerId for the controlling player
 *
 * MUST NOT:
 * - Mutate state directly (no array.push, property assignment, etc.)
 * - Use game.apply() (that's for player actions, not effects)
 * - Subscribe to events or access external state
 * - Have instance variables or lifecycle (effects are stateless)
 *
 * MVP Limitations:
 * - No targeting, no duration tracking, no modal effects
 * - No damage, life, or token systems yet
 *
 * See ABILITY_CONTRACT_MVP.md for complete implementation guidelines.
 * Place new effects in: src/echomancy/domainmodel/effects/impl/
 */
export interface Effect {
  /**
   * Resolves this effect. Use Game methods (drawCards, enterBattlefield, etc.).
   * Never mutate state directly or use game.apply() (that's for player actions).
   */
  resolve(game: Game, context: EffectContext): void
}
