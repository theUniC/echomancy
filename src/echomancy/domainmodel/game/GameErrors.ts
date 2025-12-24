export class GameError extends Error {
  constructor(message: string) {
    super(message)
    this.name = this.constructor.name
  }
}

export class InvalidPlayerCountError extends GameError {
  constructor(playerCount: number) {
    super(`Game requires at least 2 players, but got ${playerCount}`)
  }
}

export class InvalidStartingPlayerError extends GameError {
  constructor(playerId: string) {
    super(`Starting player with id '${playerId}' is not in the player list`)
  }
}

export class InvalidPlayerActionError extends GameError {
  constructor(playerId: string, action: string) {
    super(
      `Player '${playerId}' cannot perform action '${action}': only the current player can advance the step`,
    )
  }
}

export class PlayerNotFoundError extends GameError {
  constructor(playerId: string) {
    super(`Player with id '${playerId}' not found in game`)
  }
}

export class InvalidEndTurnError extends GameError {
  constructor() {
    super("Cannot end turn from CLEANUP step")
  }
}

export class InvalidPlayLandStepError extends GameError {
  constructor() {
    super("Can only play lands during main phases")
  }
}

export class LandLimitExceededError extends GameError {
  constructor() {
    super("Cannot play more than one land per turn")
  }
}

export class CardNotFoundInHandError extends GameError {
  constructor(cardId: string, playerId: string) {
    super(`Card '${cardId}' not found in hand of player '${playerId}'`)
  }
}

export class CardIsNotLandError extends GameError {
  constructor(cardId: string) {
    super(`Card '${cardId}' is not a land`)
  }
}

export class InvalidCastSpellStepError extends GameError {
  constructor() {
    super("Can only cast spells during main phases")
  }
}

export class CardIsNotSpellError extends GameError {
  constructor(cardId: string) {
    super(`Card '${cardId}' is not a spell`)
  }
}

export class InvalidEffectTargetError extends GameError {
  constructor(effectName: string, reason: string) {
    super(`Effect '${effectName}' failed: ${reason}`)
  }
}

export class PermanentNotFoundError extends GameError {
  constructor(permanentId: string) {
    super(`Permanent '${permanentId}' not found on battlefield`)
  }
}

export class CreatureAlreadyAttackedError extends GameError {
  constructor(creatureId: string) {
    super(`Creature '${creatureId}' has already attacked this turn`)
  }
}

export class TappedCreatureCannotAttackError extends GameError {
  constructor(creatureId: string) {
    super(`Creature '${creatureId}' is tapped and cannot attack`)
  }
}

export class PermanentHasNoActivatedAbilityError extends GameError {
  constructor(permanentId: string) {
    super(`Permanent '${permanentId}' has no activated ability`)
  }
}

export class CannotPayActivationCostError extends GameError {
  constructor(permanentId: string, reason: string) {
    super(`Cannot activate ability of '${permanentId}': ${reason}`)
  }
}

export class InsufficientManaError extends GameError {
  constructor(
    playerId: string,
    color: string,
    requested: number,
    available: number,
  ) {
    super(
      `Player '${playerId}' has insufficient ${color} mana: requested ${requested}, available ${available}`,
    )
  }
}

export class InvalidManaAmountError extends GameError {
  constructor(amount: number) {
    super(`Invalid mana amount: ${amount}. Amount must be greater than 0.`)
  }
}

// ============================================================================
// Cost System Errors
// ============================================================================

export class PermanentAlreadyTappedError extends GameError {
  constructor(permanentId: string) {
    super(`Permanent '${permanentId}' is already tapped`)
  }
}

export class PermanentNotControlledError extends GameError {
  constructor(permanentId: string, playerId: string) {
    super(`Player '${playerId}' does not control permanent '${permanentId}'`)
  }
}

export class CannotPayCostsError extends GameError {
  constructor(reason: string) {
    super(`Cannot pay costs: ${reason}`)
  }
}

// ============================================================================
// Creature Stats System Errors
// ============================================================================

export class InvalidCounterAmountError extends GameError {
  constructor(amount: number) {
    super(`Invalid counter amount: ${amount}. Amount must be greater than 0.`)
  }
}

// ============================================================================
// Combat System Errors
// ============================================================================

export class TappedCreatureCannotBlockError extends GameError {
  constructor(creatureId: string) {
    super(`Creature '${creatureId}' is tapped and cannot block`)
  }
}

export class CreatureAlreadyBlockingError extends GameError {
  constructor(creatureId: string) {
    super(`Creature '${creatureId}' is already blocking another creature`)
  }
}

export class CannotBlockNonAttackingCreatureError extends GameError {
  constructor(attackerId: string) {
    super(`Creature '${attackerId}' is not attacking and cannot be blocked`)
  }
}

export class AttackerAlreadyBlockedError extends GameError {
  constructor(attackerId: string) {
    super(
      `Creature '${attackerId}' is already blocked (MVP: only one blocker per attacker)`,
    )
  }
}
