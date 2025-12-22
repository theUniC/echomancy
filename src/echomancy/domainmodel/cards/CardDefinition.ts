import type { ActivatedAbility } from "../abilities/ActivatedAbility"
import type { Effect } from "../effects/Effect"
import type { Trigger } from "../triggers/Trigger"

export type CardType =
  | "CREATURE"
  | "INSTANT"
  | "SORCERY"
  | "ARTIFACT"
  | "ENCHANTMENT"
  | "PLANESWALKER"
  | "LAND"

export type CardDefinition = {
  id: string
  name: string
  types: CardType[]
  effect?: Effect
  activatedAbility?: ActivatedAbility
  /**
   * Triggered abilities
   *
   * Triggers are declarative: they specify WHEN and IF an ability fires,
   * but they do NOT execute actively or subscribe to events.
   *
   * The Game evaluates all triggers on all permanents whenever an event occurs.
   *
   * Example (ETB trigger that draws a card):
   * ```typescript
   * import { GameEventTypes } from "../game/GameEvents"
   * import { ZoneNames } from "../zones/Zone"
   *
   * triggers: [{
   *   eventType: GameEventTypes.ZONE_CHANGED,
   *   condition: (game, event, source) =>
   *     event.card.instanceId === source.instanceId &&
   *     event.toZone === ZoneNames.BATTLEFIELD,
   *   effect: (game, context) =>
   *     game.drawCards(context.controllerId, 1)
   * }]
   * ```
   *
   * NOTE: Always use GameEventTypes and ZoneNames constants to avoid magic strings.
   */
  triggers?: Trigger[]
}
