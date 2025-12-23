import type { Game } from "../game/Game"
import type { EffectContext } from "./EffectContext"

/**
 * EFFECT INTERFACE – Ability Effect Resolution
 *
 * See abilities/Ability.ts for the full Ability contract.
 *
 * ============================================================================
 * DEFINITION
 * ============================================================================
 *
 * An Effect is the executable part of an ability. When an ability resolves
 * from the stack (or executes immediately for MVP triggers), its effect is
 * what actually happens.
 *
 * Examples:
 * - "Draw a card" → DrawCardsEffect
 * - "Target player draws a card" → DrawTargetPlayerEffect
 * - "Deal 3 damage to any target" → DamageEffect (future)
 *
 * ============================================================================
 * IMPLEMENTATION RULES
 * ============================================================================
 *
 * Effects MUST:
 * 1. Use game.apply() for all state mutations (never direct mutation)
 * 2. Use game.enterBattlefield() for permanents (never array.push)
 * 3. Be stateless (no instance variables beyond configuration)
 * 4. Only mutate game state via the Game instance
 * 5. Use context.controllerId to determine the controller
 * 6. Use context.source for Last Known Information about the source
 *
 * Effects MUST NOT:
 * 1. Mutate game state directly (always use game methods)
 * 2. Subscribe to events or maintain listeners
 * 3. Have side effects beyond game state changes
 * 4. Access external state (files, network, etc.)
 * 5. Have their own lifecycle or internal state
 *
 * ============================================================================
 * EXECUTION CONTEXT
 * ============================================================================
 *
 * The resolve method receives:
 * - game: Full Game instance with read/write access
 * - context: EffectContext with controller, source, and targets
 *
 * The effect can:
 * - Read game state (game.getPlayerState, etc.)
 * - Mutate game state (game.apply, game.drawCards, etc.)
 * - Use context.controllerId to determine the controlling player
 * - Use context.source for Last Known Information (may be undefined)
 * - Use context.targets for selected targets (usually empty in MVP)
 *
 * ============================================================================
 * MVP LIMITATIONS
 * ============================================================================
 *
 * The following are NOT supported yet:
 * - Targeting (context.targets usually empty)
 * - Duration tracking ("until end of turn")
 * - Conditional effects based on choices
 * - Modal effects (choose one)
 * - X values or variable parameters
 * - Damage (no damage system yet)
 * - Life gain/loss (no life system yet)
 * - Token creation (no token system yet)
 *
 * See abilities/Ability.ts section 7 for complete non-goals list.
 *
 * ============================================================================
 * IMPLEMENTATION LOCATIONS
 * ============================================================================
 *
 * New effects should be placed in:
 * - src/echomancy/domainmodel/effects/impl/
 *
 * Examples:
 * - DrawCardsEffect.ts (draw N cards for controller)
 * - DrawTargetPlayerEffect.ts (target player draws N cards)
 * - NoOpEffect.ts (do nothing, for testing)
 *
 * ============================================================================
 */
export interface Effect {
  /**
   * Resolves this effect.
   *
   * @param game - The Game instance (can read and mutate state)
   * @param context - The execution context (controller, source, targets)
   *
   * The effect should use game.apply() and other Game methods to mutate
   * state, never mutating state directly.
   */
  resolve(game: Game, context: EffectContext): void
}
