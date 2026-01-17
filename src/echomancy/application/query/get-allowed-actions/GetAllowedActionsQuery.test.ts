import { v4 as uuidv4 } from "uuid"
import { describe, expect, it } from "vitest"
import { Game } from "@/echomancy/domainmodel/game/Game"
import { GameNotFoundError } from "@/echomancy/domainmodel/game/GameErrors"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
import { Player } from "@/echomancy/domainmodel/game/Player"
import { InMemoryGameRepository } from "@/echomancy/infrastructure/persistence/InMemoryGameRepository"
import {
  GetAllowedActionsQuery,
  GetAllowedActionsQueryHandler,
} from "./GetAllowedActionsQuery"

describe("GetAllowedActionsQuery", () => {
  it("throws InvalidGameIdError when game ID is not a valid UUID", () => {
    const repository = new InMemoryGameRepository()
    const handler = new GetAllowedActionsQueryHandler(repository)
    const playerId = uuidv4()

    expect(() => {
      handler.handle(new GetAllowedActionsQuery("invalid-id", playerId))
    }).toThrow(InvalidGameIdError)
  })

  it("throws GameNotFoundError when game does not exist", () => {
    const repository = new InMemoryGameRepository()
    const handler = new GetAllowedActionsQueryHandler(repository)
    const nonExistentGameId = uuidv4()
    const playerId = uuidv4()

    expect(() => {
      handler.handle(new GetAllowedActionsQuery(nonExistentGameId, playerId))
    }).toThrow(GameNotFoundError)
  })

  it("returns empty playableLands when not in main phase", () => {
    const repository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1 = new Player(uuidv4(), "Player 1")
    const player2 = new Player(uuidv4(), "Player 2")

    const game = Game.create(gameId)
    game.addPlayer(player1)
    game.addPlayer(player2)
    game.start(player1.id)
    repository.add(game)

    // Game starts in UNTAP step, not main phase
    const handler = new GetAllowedActionsQueryHandler(repository)
    const result = handler.handle(
      new GetAllowedActionsQuery(gameId, player1.id),
    )

    expect(result.playableLands).toEqual([])
  })

  it("returns land cards from hand when in main phase and no land played", () => {
    const repository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1 = new Player(uuidv4(), "Player 1")
    const player2 = new Player(uuidv4(), "Player 2")

    const game = Game.create(gameId)
    game.addPlayer(player1)
    game.addPlayer(player2)
    game.start(player1.id)

    // Add a land to player1's hand
    const landCard = {
      instanceId: uuidv4(),
      ownerId: player1.id,
      definition: {
        name: "Forest",
        types: ["LAND"],
        subtypes: ["Forest"],
        manaCost: "",
        text: "Add G",
      },
    }
    game.getPlayerState(player1.id).hand.cards.push(landCard)

    // Advance to first main phase
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
    // Now in FIRST_MAIN

    repository.add(game)

    const handler = new GetAllowedActionsQueryHandler(repository)
    const result = handler.handle(
      new GetAllowedActionsQuery(gameId, player1.id),
    )

    expect(result.playableLands).toContain(landCard.instanceId)
    expect(result.playableLands).toHaveLength(1)
  })

  it("returns empty playableLands when land already played this turn", () => {
    const repository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1 = new Player(uuidv4(), "Player 1")
    const player2 = new Player(uuidv4(), "Player 2")

    const game = Game.create(gameId)
    game.addPlayer(player1)
    game.addPlayer(player2)
    game.start(player1.id)

    // Add two lands to player1's hand
    const landCard1 = {
      instanceId: uuidv4(),
      ownerId: player1.id,
      definition: {
        name: "Forest",
        types: ["LAND"],
        subtypes: ["Forest"],
        manaCost: "",
        text: "Add G",
      },
    }
    const landCard2 = {
      instanceId: uuidv4(),
      ownerId: player1.id,
      definition: {
        name: "Plains",
        types: ["LAND"],
        subtypes: ["Plains"],
        manaCost: "",
        text: "Add W",
      },
    }
    game.getPlayerState(player1.id).hand.cards.push(landCard1, landCard2)

    // Advance to first main phase
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

    // Play one land
    game.apply({
      type: "PLAY_LAND",
      playerId: player1.id,
      cardId: landCard1.instanceId,
    })

    repository.add(game)

    const handler = new GetAllowedActionsQueryHandler(repository)
    const result = handler.handle(
      new GetAllowedActionsQuery(gameId, player1.id),
    )

    expect(result.playableLands).toEqual([])
  })

  it("returns empty playableLands when not current player", () => {
    const repository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1 = new Player(uuidv4(), "Player 1")
    const player2 = new Player(uuidv4(), "Player 2")

    const game = Game.create(gameId)
    game.addPlayer(player1)
    game.addPlayer(player2)
    game.start(player1.id) // player1 is current player

    // Add a land to player2's hand
    const landCard = {
      instanceId: uuidv4(),
      ownerId: player2.id,
      definition: {
        name: "Forest",
        types: ["LAND"],
        subtypes: ["Forest"],
        manaCost: "",
        text: "Add G",
      },
    }
    game.getPlayerState(player2.id).hand.cards.push(landCard)

    // Advance to first main phase
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

    repository.add(game)

    const handler = new GetAllowedActionsQueryHandler(repository)
    const result = handler.handle(
      new GetAllowedActionsQuery(gameId, player2.id), // player2 querying
    )

    expect(result.playableLands).toEqual([])
  })

  it("filters out non-land cards from hand", () => {
    const repository = new InMemoryGameRepository()
    const gameId = uuidv4()
    const player1 = new Player(uuidv4(), "Player 1")
    const player2 = new Player(uuidv4(), "Player 2")

    const game = Game.create(gameId)
    game.addPlayer(player1)
    game.addPlayer(player2)
    game.start(player1.id)

    // Add both land and non-land cards
    const landCard = {
      instanceId: uuidv4(),
      ownerId: player1.id,
      definition: {
        name: "Forest",
        types: ["LAND"],
        subtypes: ["Forest"],
        manaCost: "",
        text: "Add G",
      },
    }
    const creatureCard = {
      instanceId: uuidv4(),
      ownerId: player1.id,
      definition: {
        name: "Grizzly Bears",
        types: ["Creature"],
        subtypes: ["Bear"],
        manaCost: "1G",
        text: "",
        power: 2,
        toughness: 2,
      },
    }
    game.getPlayerState(player1.id).hand.cards.push(landCard, creatureCard)

    // Advance to first main phase
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
    game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

    repository.add(game)

    const handler = new GetAllowedActionsQueryHandler(repository)
    const result = handler.handle(
      new GetAllowedActionsQuery(gameId, player1.id),
    )

    expect(result.playableLands).toContain(landCard.instanceId)
    expect(result.playableLands).not.toContain(creatureCard.instanceId)
    expect(result.playableLands).toHaveLength(1)
  })
})
