/**
 * Cost Implementations - Concrete cost types for the MVP
 *
 * Supports:
 * - ManaCost - Pay mana from pool
 * - TapSelfCost - Tap the permanent with this ability
 * - SacrificeSelfCost - Sacrifice the permanent with this ability
 */

import type { Game, ManaColor } from "../../game/Game"
import {
  PermanentAlreadyTappedError,
  PermanentNotControlledError,
  PermanentNotFoundError,
} from "../../game/GameErrors"
import type { Cost, CostContext } from "../Cost"

/**
 * ManaCost - Pay mana from the player's mana pool
 *
 * Example: { G: 2, W: 1 } means "pay 2 green and 1 white mana"
 *
 * TODO: Support generic mana costs (colorless that can be paid with any color)
 * TODO: Support hybrid mana costs
 * TODO: Support Phyrexian mana costs
 * TODO: Support X costs
 */
export class ManaCost implements Cost {
  constructor(private readonly amounts: Partial<Record<ManaColor, number>>) {}

  canPay(game: Game, context: CostContext): boolean {
    const pool = game.getManaPool(context.playerId)

    // Check if player has sufficient mana of each required color
    for (const [color, amount] of Object.entries(this.amounts)) {
      if (amount === undefined || amount === 0) continue

      const available = pool[color as ManaColor] ?? 0
      if (available < amount) {
        return false
      }
    }

    return true
  }

  pay(game: Game, context: CostContext): void {
    // Spend mana for each color
    for (const [color, amount] of Object.entries(this.amounts)) {
      if (amount === undefined || amount === 0) continue

      game.spendMana(context.playerId, color as ManaColor, amount)
    }
  }
}

/**
 * TapSelfCost - Tap the permanent that has this ability
 *
 * Validates:
 * - Permanent exists on battlefield
 * - Permanent is untapped
 * - Permanent is controlled by the player
 *
 * TODO: Support tapping other permanents (not just self)
 * TODO: Support tapping multiple permanents
 */
export class TapSelfCost implements Cost {
  canPay(game: Game, context: CostContext): boolean {
    // Find the permanent
    const playerState = game.getPlayerState(context.playerId)
    const permanent = playerState.battlefield.cards.find(
      (card) => card.instanceId === context.sourceId,
    )

    if (!permanent) {
      return false
    }

    // Check if permanent is a creature (MVP: only creatures have tap state)
    // TODO: Track tap state for all permanents, not just creatures
    const isCreature = permanent.definition.types.includes("CREATURE")
    if (!isCreature) {
      // For MVP, non-creatures are assumed to be untapped
      return true
    }

    // Check if creature is untapped
    const creatureState = game.getCreatureState(permanent.instanceId)
    return !creatureState.isTapped
  }

  pay(game: Game, context: CostContext): void {
    // Find the permanent on ANY battlefield
    const playerIds = game.getPlayersInTurnOrder()
    let permanent = null

    for (const playerId of playerIds) {
      const playerState = game.getPlayerState(playerId)
      const found = playerState.battlefield.cards.find(
        (card) => card.instanceId === context.sourceId,
      )
      if (found) {
        permanent = found
        break
      }
    }

    if (!permanent) {
      throw new PermanentNotFoundError(context.sourceId)
    }

    // Verify control
    if (permanent.ownerId !== context.playerId) {
      throw new PermanentNotControlledError(context.sourceId, context.playerId)
    }

    // Check if permanent is a creature (MVP: only creatures have tap state)
    const isCreature = permanent.definition.types.includes("CREATURE")
    if (!isCreature) {
      // For MVP, non-creatures are assumed to be untapped
      // In the future, all permanents will have tap state
      return
    }

    // Tap the creature
    const creatureState = game.getCreatureState(permanent.instanceId)
    if (creatureState.isTapped) {
      throw new PermanentAlreadyTappedError(context.sourceId)
    }

    creatureState.isTapped = true
  }
}

/**
 * SacrificeSelfCost - Sacrifice the permanent that has this ability
 *
 * Validates:
 * - Permanent exists on battlefield
 * - Permanent is controlled by the player
 *
 * Moves the permanent from battlefield to graveyard
 *
 * TODO: Support sacrificing other permanents (not just self)
 * TODO: Support sacrificing multiple permanents
 * TODO: Support sacrificing permanents with specific properties (e.g., "sacrifice a creature")
 */
export class SacrificeSelfCost implements Cost {
  canPay(game: Game, context: CostContext): boolean {
    // Find the permanent
    const playerState = game.getPlayerState(context.playerId)
    const permanent = playerState.battlefield.cards.find(
      (card) => card.instanceId === context.sourceId,
    )

    return permanent !== undefined
  }

  pay(game: Game, context: CostContext): void {
    // Find the permanent on ANY battlefield
    const playerIds = game.getPlayersInTurnOrder()
    let permanent = null
    let permanentOwnerState = null

    for (const playerId of playerIds) {
      const playerState = game.getPlayerState(playerId)
      const found = playerState.battlefield.cards.find(
        (card) => card.instanceId === context.sourceId,
      )
      if (found) {
        permanent = found
        permanentOwnerState = playerState
        break
      }
    }

    if (!permanent) {
      throw new PermanentNotFoundError(context.sourceId)
    }

    // Verify control
    if (permanent.ownerId !== context.playerId) {
      throw new PermanentNotControlledError(context.sourceId, context.playerId)
    }

    // Move from battlefield to graveyard
    const permanentIndex = permanentOwnerState?.battlefield.cards.findIndex(
      (card) => card.instanceId === context.sourceId,
    )
    permanentOwnerState?.battlefield.cards.splice(permanentIndex, 1)
    permanentOwnerState?.graveyard.cards.push(permanent)

    // TODO: Emit ZONE_CHANGED event for sacrifice
    // TODO: Handle triggered abilities that fire on sacrifice
  }
}
