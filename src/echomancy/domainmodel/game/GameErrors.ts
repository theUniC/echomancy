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
