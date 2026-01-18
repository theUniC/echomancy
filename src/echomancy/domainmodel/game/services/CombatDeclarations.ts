/**
 * CombatDeclarations Domain Service
 *
 * Validates attacker and blocker declarations according to MTG rules.
 * Returns validated state changes for Game.ts to apply.
 */

import type { StaticAbility } from "../../cards/CardDefinition"
import { StaticAbilities } from "../../cards/CardDefinition"
import type { CardInstance } from "../../cards/CardInstance"
import {
  AttackerAlreadyBlockedError,
  CannotBlockFlyingCreatureError,
  CannotBlockNonAttackingCreatureError,
  CreatureAlreadyAttackedError,
  CreatureAlreadyBlockingError,
  CreatureHasSummoningSicknessError,
  InvalidPlayerActionError,
  PermanentNotFoundError,
  TappedCreatureCannotAttackError,
  TappedCreatureCannotBlockError,
} from "../GameErrors"
import type { GameSteps } from "../Steps"
import { Step } from "../Steps"
import type { CreatureState } from "../valueobjects/CreatureState"

/**
 * Read-only context interface for combat validation.
 * Game implements this interface to provide combat validation access.
 */
export type CombatValidationContext = {
  readonly currentStep: GameSteps
  readonly currentPlayerId: string

  getOpponentOf(playerId: string): string
  getBattlefieldCards(playerId: string): readonly CardInstance[]
  isCreature(card: CardInstance): boolean
  hasStaticAbility(card: CardInstance, ability: StaticAbility): boolean
  getCreatureState(instanceId: string): CreatureState | undefined
}

/**
 * Result of validating a declare attacker action.
 */
export type DeclareAttackerResult = {
  creature: CardInstance
  newCreatureState: CreatureState
}

/**
 * Result of validating a declare blocker action.
 */
export type DeclareBlockerResult = {
  blocker: CardInstance
  attacker: CardInstance
  newBlockerState: CreatureState
  newAttackerState: CreatureState
}

/**
 * Validates a declare attacker action and returns the state changes.
 * @throws Various errors if validation fails
 */
export function validateDeclareAttacker(
  ctx: CombatValidationContext,
  playerId: string,
  creatureId: string,
): DeclareAttackerResult {
  // Verify it's DECLARE_ATTACKERS step
  if (ctx.currentStep !== Step.DECLARE_ATTACKERS) {
    throw new InvalidPlayerActionError(playerId, "DECLARE_ATTACKER")
  }

  // Verify player is the current player
  if (playerId !== ctx.currentPlayerId) {
    throw new InvalidPlayerActionError(playerId, "DECLARE_ATTACKER")
  }

  // Verify creature exists on battlefield and is controlled by player
  const battlefieldCards = ctx.getBattlefieldCards(playerId)
  const creature = battlefieldCards.find(
    (card) => card.instanceId === creatureId,
  )

  if (!creature) {
    throw new PermanentNotFoundError(creatureId)
  }

  if (!ctx.isCreature(creature)) {
    throw new PermanentNotFoundError(creatureId)
  }

  const creatureState = ctx.getCreatureState(creatureId)
  if (!creatureState) {
    throw new PermanentNotFoundError(creatureId)
  }

  // Verify creature does not have summoning sickness (unless it has Haste)
  if (
    creatureState.hasSummoningSickness &&
    !ctx.hasStaticAbility(creature, StaticAbilities.HASTE)
  ) {
    throw new CreatureHasSummoningSicknessError(creatureId)
  }

  // Verify creature is not tapped
  if (creatureState.isTapped) {
    throw new TappedCreatureCannotAttackError(creatureId)
  }

  // Verify creature has not attacked this turn
  if (creatureState.hasAttackedThisTurn) {
    throw new CreatureAlreadyAttackedError(creatureId)
  }

  // Calculate new state
  let newCreatureState = creatureState
    .withAttacking(true)
    .withHasAttackedThisTurn(true)

  // Tap the creature unless it has Vigilance
  if (!ctx.hasStaticAbility(creature, StaticAbilities.VIGILANCE)) {
    newCreatureState = newCreatureState.withTapped(true)
  }

  return {
    creature,
    newCreatureState,
  }
}

/**
 * Validates a declare blocker action and returns the state changes.
 * @throws Various errors if validation fails
 */
export function validateDeclareBlocker(
  ctx: CombatValidationContext,
  playerId: string,
  blockerId: string,
  attackerId: string,
): DeclareBlockerResult {
  // Verify it's DECLARE_BLOCKERS step
  if (ctx.currentStep !== Step.DECLARE_BLOCKERS) {
    throw new InvalidPlayerActionError(playerId, "DECLARE_BLOCKER")
  }

  // Verify the acting player is the defending player (not the active player)
  // MVP: In a 2-player game, the defending player is the opponent of the active player
  const defendingPlayer = ctx.getOpponentOf(ctx.currentPlayerId)
  if (playerId !== defendingPlayer) {
    throw new InvalidPlayerActionError(playerId, "DECLARE_BLOCKER")
  }

  // Find the blocker on the defending player's battlefield
  const defenderBattlefield = ctx.getBattlefieldCards(playerId)
  const blocker = defenderBattlefield.find(
    (card) => card.instanceId === blockerId,
  )

  if (!blocker) {
    throw new PermanentNotFoundError(blockerId)
  }

  if (!ctx.isCreature(blocker)) {
    throw new PermanentNotFoundError(blockerId)
  }

  const blockerState = ctx.getCreatureState(blockerId)
  if (!blockerState) {
    throw new PermanentNotFoundError(blockerId)
  }

  // Verify blocker is not tapped
  if (blockerState.isTapped) {
    throw new TappedCreatureCannotBlockError(blockerId)
  }

  // Verify blocker is not already blocking
  if (blockerState.blockingCreatureId !== null) {
    throw new CreatureAlreadyBlockingError(blockerId)
  }

  // Find the attacker and verify it's actually attacking
  const attackerState = ctx.getCreatureState(attackerId)
  if (!attackerState) {
    throw new PermanentNotFoundError(attackerId)
  }

  if (!attackerState.isAttacking) {
    throw new CannotBlockNonAttackingCreatureError(attackerId)
  }

  // MVP: Only one blocker per attacker allowed
  if (attackerState.blockedBy !== null) {
    throw new AttackerAlreadyBlockedError(attackerId)
  }

  // Find the attacker card instance to check for flying
  // MVP assumption: In 2-player games, attackers are always controlled by the active player
  const activePlayerBattlefield = ctx.getBattlefieldCards(ctx.currentPlayerId)
  const attacker = activePlayerBattlefield.find(
    (card) => card.instanceId === attackerId,
  )

  if (!attacker) {
    throw new PermanentNotFoundError(attackerId)
  }

  // MVP static abilities: Flying/Reach blocking restriction
  // A creature with Flying can only be blocked by creatures with Flying or Reach
  if (ctx.hasStaticAbility(attacker, StaticAbilities.FLYING)) {
    const blockerHasFlyingOrReach =
      ctx.hasStaticAbility(blocker, StaticAbilities.FLYING) ||
      ctx.hasStaticAbility(blocker, StaticAbilities.REACH)

    if (!blockerHasFlyingOrReach) {
      throw new CannotBlockFlyingCreatureError(blockerId, attackerId)
    }
  }

  // Calculate new states
  const newBlockerState = blockerState.withBlockingCreatureId(attackerId)
  const newAttackerState = attackerState.withBlockedBy(blockerId)

  return {
    blocker,
    attacker,
    newBlockerState,
    newAttackerState,
  }
}
