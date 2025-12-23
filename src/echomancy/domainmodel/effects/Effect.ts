import type { Game } from "../game/Game"
import type { EffectContext } from "./EffectContext"

/**
 * Effect - The executable part of an ability
 *
 * When an ability resolves, its effect is what actually happens.
 * Examples: DrawCardsEffect, DamageEffect (future), CreateTokenEffect (future)
 *
 * IMPLEMENTATION RULES (CRITICAL):
 * MUST use:
 * - game.apply() for state mutations (never direct mutation)
 * - game.enterBattlefield() for permanents (never array.push)
 * - context.controllerId for the controlling player
 *
 * MUST NOT:
 * - Mutate state directly, subscribe to events, access external state
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
   * Resolves this effect. Use game methods (game.apply, etc.) to mutate state.
   * Never mutate state directly.
   */
  resolve(game: Game, context: EffectContext): void
}
