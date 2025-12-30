import { v4 as uuidv4 } from "uuid"
import { beforeEach, describe, expect, test } from "vitest"
import { Game } from "@/echomancy/domainmodel/game/Game"
import { Player } from "@/echomancy/domainmodel/game/Player"
import { gameRepository } from "@/lib/repositories"
import { POST } from "../route"

describe("POST /api/games/[gameId]/start", () => {
  let gameId: string
  let player1Id: string
  let player2Id: string

  beforeEach(() => {
    // Create a game with two players for each test
    gameId = uuidv4()
    player1Id = uuidv4()
    player2Id = uuidv4()

    const game = Game.create(gameId)
    game.addPlayer(new Player(player1Id, "Player 1"))
    game.addPlayer(new Player(player2Id, "Player 2"))
    gameRepository.add(game)
  })

  test("starts the game with valid starting player and returns 200", async () => {
    const request = new Request(`http://localhost/api/games/${gameId}/start`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ startingPlayerId: player1Id }),
    })

    const response = await POST(request, {
      params: Promise.resolve({ gameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(200)
    expect(body).toEqual({ data: { started: true } })

    // Verify game was actually started
    const game = gameRepository.byId(gameId)
    expect(game?.currentPlayerId).toBe(player1Id)
  })

  test("returns 404 when game does not exist", async () => {
    const nonExistentGameId = uuidv4()

    const request = new Request(
      `http://localhost/api/games/${nonExistentGameId}/start`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ startingPlayerId: player1Id }),
      },
    )

    const response = await POST(request, {
      params: Promise.resolve({ gameId: nonExistentGameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(404)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("GAME_NOT_FOUND")
  })

  test("returns 400 for invalid gameId UUID", async () => {
    const request = new Request(
      `http://localhost/api/games/invalid-uuid/start`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ startingPlayerId: player1Id }),
      },
    )

    const response = await POST(request, {
      params: Promise.resolve({ gameId: "invalid-uuid" }),
    })
    const body = await response.json()

    expect(response.status).toBe(400)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("INVALID_GAME_ID")
  })

  test("returns 422 when starting player is not in the game", async () => {
    const unknownPlayerId = uuidv4()

    const request = new Request(`http://localhost/api/games/${gameId}/start`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ startingPlayerId: unknownPlayerId }),
    })

    const response = await POST(request, {
      params: Promise.resolve({ gameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(422)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("InvalidStartingPlayerError")
  })

  test("returns 422 when game has insufficient players", async () => {
    // Create a game with only one player
    const singlePlayerGameId = uuidv4()
    const soloPlayerId = uuidv4()
    const game = Game.create(singlePlayerGameId)
    game.addPlayer(new Player(soloPlayerId, "Solo Player"))
    gameRepository.add(game)

    const request = new Request(
      `http://localhost/api/games/${singlePlayerGameId}/start`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ startingPlayerId: soloPlayerId }),
      },
    )

    const response = await POST(request, {
      params: Promise.resolve({ gameId: singlePlayerGameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(422)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("InvalidPlayerCountError")
  })

  test("returns 422 when game is already started", async () => {
    // Start the game first
    const game = gameRepository.byId(gameId)
    game?.start(player1Id)

    const request = new Request(`http://localhost/api/games/${gameId}/start`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ startingPlayerId: player1Id }),
    })

    const response = await POST(request, {
      params: Promise.resolve({ gameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(422)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("GameAlreadyStartedError")
  })
})
