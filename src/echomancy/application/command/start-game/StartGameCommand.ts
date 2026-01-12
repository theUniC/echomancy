import { validate as isValidUUID, v4 as uuidv4 } from "uuid"
import type { CardInstance } from "@/echomancy/domainmodel/cards/CardInstance"
import type { Game } from "@/echomancy/domainmodel/game/Game"
import {
  GameNotFoundError,
  InvalidPlayerIdError,
} from "@/echomancy/domainmodel/game/GameErrors"
import type { GameRepository } from "@/echomancy/domainmodel/game/GameRepository"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"

// ============================================================================
// Card Factory Functions
// ============================================================================
// These are temporary bootstrap functions to provide starting hands.
// They will be removed when deck/library systems are implemented.

function createForest(ownerId: string): CardInstance {
  return {
    instanceId: uuidv4(),
    definition: {
      id: "forest",
      name: "Forest",
      types: ["LAND"],
    },
    ownerId,
  }
}

function createPlains(ownerId: string): CardInstance {
  return {
    instanceId: uuidv4(),
    definition: {
      id: "plains",
      name: "Plains",
      types: ["LAND"],
    },
    ownerId,
  }
}

function createGrizzlyBears(ownerId: string): CardInstance {
  return {
    instanceId: uuidv4(),
    definition: {
      id: "grizzly-bears",
      name: "Grizzly Bears",
      types: ["CREATURE"],
      power: 2,
      toughness: 2,
    },
    ownerId,
  }
}

function createEliteVanguard(ownerId: string): CardInstance {
  return {
    instanceId: uuidv4(),
    definition: {
      id: "elite-vanguard",
      name: "Elite Vanguard",
      types: ["CREATURE"],
      power: 2,
      toughness: 1,
    },
    ownerId,
  }
}

function createGiantSpider(ownerId: string): CardInstance {
  return {
    instanceId: uuidv4(),
    definition: {
      id: "giant-spider",
      name: "Giant Spider",
      types: ["CREATURE"],
      power: 2,
      toughness: 4,
      staticAbilities: ["REACH"],
    },
    ownerId,
  }
}

function createSerraAngel(ownerId: string): CardInstance {
  return {
    instanceId: uuidv4(),
    definition: {
      id: "serra-angel",
      name: "Serra Angel",
      types: ["CREATURE"],
      power: 4,
      toughness: 4,
      staticAbilities: ["FLYING", "VIGILANCE"],
    },
    ownerId,
  }
}

function createLlanowarElves(ownerId: string): CardInstance {
  return {
    instanceId: uuidv4(),
    definition: {
      id: "llanowar-elves",
      name: "Llanowar Elves",
      types: ["CREATURE"],
      power: 1,
      toughness: 1,
    },
    ownerId,
  }
}

/**
 * Populates starting hands for all players with a predetermined set of cards.
 * Each player receives:
 * - 2 lands (Forest, Plains)
 * - 5 creatures (Grizzly Bears, Elite Vanguard, Giant Spider, Serra Angel, Llanowar Elves)
 *
 * This is a temporary bootstrap mechanism to enable UI development.
 * It will be removed when deck/library systems are implemented.
 *
 * @param game - The game instance to populate hands for
 */
function populateStartingHands(game: Game): void {
  for (const player of game.getPlayers()) {
    const playerState = game.getPlayerState(player.id)

    // Add 2 lands
    playerState.hand.cards.push(createForest(player.id))
    playerState.hand.cards.push(createPlains(player.id))

    // Add 5 creatures
    playerState.hand.cards.push(createGrizzlyBears(player.id))
    playerState.hand.cards.push(createEliteVanguard(player.id))
    playerState.hand.cards.push(createGiantSpider(player.id))
    playerState.hand.cards.push(createSerraAngel(player.id))
    playerState.hand.cards.push(createLlanowarElves(player.id))
  }
}

// ============================================================================
// Command and Handler
// ============================================================================

export class StartGameCommand {
  constructor(
    public gameId: string,
    public startingPlayerId: string,
  ) {}
}

export class StartGameCommandHandler {
  constructor(private gameRepository: GameRepository) {}

  handle({ gameId, startingPlayerId }: StartGameCommand) {
    // 1. Input validation
    if (!isValidUUID(gameId)) {
      throw new InvalidGameIdError(gameId)
    }

    if (!isValidUUID(startingPlayerId)) {
      throw new InvalidPlayerIdError(startingPlayerId)
    }

    // 2. Existence check
    const game = this.gameRepository.byId(gameId)
    if (!game) {
      throw new GameNotFoundError(gameId)
    }

    // 3. Domain logic (game.start validates player count, player exists, etc.)
    game.start(startingPlayerId)

    // 4. Bootstrap: Populate starting hands (temporary until deck/library exists)
    populateStartingHands(game)
  }
}
