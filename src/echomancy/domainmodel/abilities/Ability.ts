/**
 * ABILITY CONTRACT – MVP Definition
 *
 * This file defines the formal contract for Abilities in Echomancy.
 *
 * ============================================================================
 * 1. WHAT IS AN ABILITY?
 * ============================================================================
 *
 * An Ability is a rule unit attached to a card or permanent that may produce
 * an effect when a specific condition is met.
 *
 * CORE PRINCIPLES:
 * - Abilities belong to exactly one card/permanent
 * - Abilities have a clearly defined trigger or activation condition
 * - Abilities resolve via the Game, never mutating state directly
 * - Abilities are DECLARATIVE, not reactive
 *
 * ABILITIES DO NOT:
 * - Subscribe to global events or maintain listeners
 * - Mutate Game state outside their resolution
 * - Execute automatically outside defined evaluation points
 * - Have their own internal state or lifecycle
 *
 * ============================================================================
 * 2. TYPES OF ABILITIES – MVP SCOPE
 * ============================================================================
 *
 * SUPPORTED IN MVP:
 *
 * A. ActivatedAbility
 *    - Player explicitly activates it (e.g., "{T}: Draw a card")
 *    - Has an activation cost (currently only {T} supported)
 *    - Goes on the stack as an AbilityOnStack
 *    - Resolves independently (Last Known Information)
 *
 * B. TriggeredAbility (via Trigger type)
 *    - Triggered by a specific Game event (e.g., "When ~ enters the battlefield")
 *    - Automatically detected by Game when events occur
 *    - MVP LIMITATION: Currently executes immediately (does NOT go on stack)
 *    - TODO: Should create StackItem for triggered abilities
 *
 * NOT SUPPORTED IN MVP (Explicitly out of scope):
 *
 * C. StaticAbility
 *    - Continuous effects that don't use the stack
 *    - Examples: "Creatures you control get +1/+1"
 *    - TODO: Requires continuous effect system
 *
 * D. ReplacementEffect
 *    - Modifies how events occur (e.g., "If a creature would die, exile it instead")
 *    - TODO: Requires replacement effect framework
 *
 * E. PreventionEffect
 *    - Prevents damage or other events
 *    - TODO: Requires prevention framework
 *
 * F. ManaAbility
 *    - Produces mana, doesn't use the stack
 *    - TODO: Requires mana system
 *
 * ============================================================================
 * 3. STACK INTERACTION RULES
 * ============================================================================
 *
 * When an ActivatedAbility is activated:
 * 1. Create an AbilityOnStack item
 * 2. Add it to the stack
 * 3. Store the effect for Last Known Information
 * 4. Resolve via normal stack resolution (LIFO)
 *
 * When a TriggeredAbility triggers:
 * MVP CURRENT BEHAVIOR:
 * - Effect executes immediately when trigger condition is met
 * - No StackItem created
 * - No priority round for triggered abilities
 *
 * MVP FUTURE BEHAVIOR (TODO):
 * - Create a TriggeredAbilityOnStack item
 * - Add it to the stack
 * - Allow players to respond
 * - Resolve via normal stack resolution (LIFO)
 * - Implement APNAP ordering for simultaneous triggers
 *
 * ============================================================================
 * 4. WHEN ABILITIES ARE EVALUATED (CRITICAL)
 * ============================================================================
 *
 * The Game is the ONLY authority that evaluates abilities.
 *
 * ACTIVATED ABILITIES:
 * - Evaluated when player uses ACTIVATE_ABILITY action
 * - Requires priority
 * - Can be activated at instant speed (MVP limitation - no timing restrictions)
 *
 * TRIGGERED ABILITIES:
 * - Evaluated ONLY at these specific points:
 *   1. After enterBattlefield() → ZONE_CHANGED event
 *   2. After declareAttacker() → CREATURE_DECLARED_ATTACKER event
 *   3. After resolveSpell() → SPELL_RESOLVED event
 *   4. On step transition → STEP_STARTED, COMBAT_ENDED events
 *
 * - The Game calls evaluateTriggers(event) at these points
 * - Triggers are NEVER evaluated continuously or reactively
 * - No event subscription or observer pattern
 *
 * IMPORTANT:
 * - Abilities do not "watch" for events
 * - Cards declare triggers, Game evaluates them
 * - This keeps the engine deterministic and testable
 *
 * ============================================================================
 * 5. ABILITY CONTEXT
 * ============================================================================
 *
 * All abilities receive an EffectContext when executed:
 *
 * ALWAYS PRESENT:
 * - controllerId: The player who controls the ability
 *
 * OPTIONAL:
 * - source: The card/permanent that has the ability
 *   - May be undefined for effects that outlive their source
 * - targets: Array of targets chosen for this ability
 *   - MVP LIMITATION: Empty array for most abilities (targeting not implemented)
 *
 * IMPORTANT NOTES:
 * - ETB abilities DO NOT reuse spell targets
 * - Each ability must define its own targeting requirements (future)
 * - Context is captured when ability is activated/triggered
 * - Context includes Last Known Information for the source
 *
 * ============================================================================
 * 6. ETB (ENTER THE BATTLEFIELD) ABILITIES
 * ============================================================================
 *
 * ETB abilities are TriggeredAbilities with specific characteristics:
 *
 * DEFINITION:
 * - Trigger on ZONE_CHANGED event
 * - Condition checks: toZone === "BATTLEFIELD"
 * - Often also checks: card.instanceId === source.instanceId
 *
 * SUPPORTED:
 * - Creatures entering via spell resolution
 * - Permanents entering via spell resolution
 * - Checking that the entering permanent is the source of the trigger
 *
 * NOT SUPPORTED YET:
 * - ETB replacement effects (e.g., "~ enters with a +1/+1 counter")
 * - Optional triggers (e.g., "you may...")
 * - Conditional ETBs with complex state checks
 * - Targeting on ETB (requires target selection infrastructure)
 * - "Whenever another creature enters..." (other permanent triggers)
 *
 * EXECUTION ORDER:
 * 1. Spell resolves (spell effect executes if defined)
 * 2. Permanent enters battlefield (if applicable)
 * 3. ZONE_CHANGED event emitted
 * 4. Game evaluates all triggers for ZONE_CHANGED
 * 5. ETB triggers execute (in collection order - TODO: APNAP)
 *
 * ============================================================================
 * 7. EXPLICIT NON-GOALS FOR MVP
 * ============================================================================
 *
 * The following are intentionally NOT implemented:
 *
 * - Mana costs for abilities
 * - Timing restrictions beyond priority (e.g., "Activate only during combat")
 * - Choice-based abilities (e.g., "Choose one —")
 * - "May" abilities (optional triggers/activations)
 * - Cost reduction for abilities
 * - Trigger ordering control (APNAP not implemented)
 * - Replacement effects
 * - Prevention effects
 * - Static abilities
 * - Continuous effects
 * - Duration tracking ("until end of turn")
 * - Delayed triggered abilities
 * - State-based triggers
 * - Intervening-if clauses
 * - Loyalty abilities (planeswalkers)
 * - Multiple costs combined (e.g., "{T}, Pay 2 life")
 * - X costs
 * - Alternative costs
 *
 * These must be marked with TODO comments where relevant.
 *
 * ============================================================================
 * 8. VALIDATION RULES
 * ============================================================================
 *
 * When implementing abilities, ensure:
 *
 * 1. Effects use game.apply() for state mutations (never direct mutation)
 * 2. Effects use game.enterBattlefield() for permanents (never array.push)
 * 3. Triggers are pure predicates (no side effects in conditions)
 * 4. Abilities do not store mutable state
 * 5. All trigger eventTypes use GameEventTypes constants
 * 6. All zone checks use ZoneNames constants
 * 7. Effects receive Game and EffectContext, nothing more
 * 8. No direct event subscription or observer pattern
 *
 * ============================================================================
 * 9. FUTURE EVOLUTION
 * ============================================================================
 *
 * When expanding the ability system, preserve these principles:
 *
 * 1. Game remains the single source of truth
 * 2. Abilities remain declarative (no active listeners)
 * 3. Stack is the only execution mechanism (except mana abilities)
 * 4. Evaluation points remain explicit and deterministic
 * 5. Abilities do not modify other abilities
 * 6. No global event bus or publish/subscribe
 *
 * ============================================================================
 */

import type { Trigger } from "../triggers/Trigger"
import type { ActivatedAbility } from "./ActivatedAbility"

/**
 * Union type for all ability types in the system.
 *
 * Currently includes:
 * - ActivatedAbility (defined in ActivatedAbility.ts)
 * - Trigger (defined in triggers/Trigger.ts)
 *
 * Future additions:
 * - StaticAbility (TODO)
 * - ManaAbility (TODO)
 */
export type Ability = ActivatedAbility | { trigger: Trigger }

/**
 * Type guard to check if an ability is an ActivatedAbility
 */
export function isActivatedAbility(
  ability: Ability,
): ability is ActivatedAbility {
  return "cost" in ability && "effect" in ability
}

/**
 * Type guard to check if an ability is a Trigger
 */
export function isTrigger(ability: Ability): ability is { trigger: Trigger } {
  return "trigger" in ability
}
