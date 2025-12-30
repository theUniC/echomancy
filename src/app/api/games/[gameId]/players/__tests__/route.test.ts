import { v4 as uuidv4 } from "uuid"
import { beforeEach, describe, expect, test } from "vitest"
import { Game } from "@/echomancy/domainmodel/game/Game"
import { gameRepository } from "@/lib/repositories"
import { POST } from "../route"

describe("POST /api/games/[gameId]/players", () => {
  let gameId: string

  beforeEach(() => {
    // Create a new game for each test
    gameId = uuidv4()
    const game = Game.create(gameId)
    gameRepository.add(game)
  })

  test("adds a player to the game and returns 201", async () => {
    const playerId = uuidv4()
    const playerName = "Player 1"

    const request = new Request(
      `http://localhost/api/games/${gameId}/players`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ playerId, playerName }),
      },
    )

    const response = await POST(request, {
      params: Promise.resolve({ gameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(201)
    expect(body).toEqual({ data: { playerId, playerName } })

    // Verify player was actually added to the game
    const game = gameRepository.byId(gameId)
    expect(game?.hasPlayer(playerId)).toBe(true)
  })

  test("returns 404 when game does not exist", async () => {
    const nonExistentGameId = uuidv4()
    const playerId = uuidv4()
    const playerName = "Player 1"

    const request = new Request(
      `http://localhost/api/games/${nonExistentGameId}/players`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ playerId, playerName }),
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
    const playerId = uuidv4()
    const playerName = "Player 1"

    const request = new Request(
      `http://localhost/api/games/invalid-uuid/players`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ playerId, playerName }),
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

  test("returns 400 for invalid playerId UUID", async () => {
    const playerName = "Player 1"

    const request = new Request(
      `http://localhost/api/games/${gameId}/players`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ playerId: "invalid-player-id", playerName }),
      },
    )

    const response = await POST(request, {
      params: Promise.resolve({ gameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(400)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("INVALID_PLAYER_ID")
  })

  test("returns 400 for missing playerId", async () => {
    const playerName = "Player 1"

    const request = new Request(
      `http://localhost/api/games/${gameId}/players`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ playerName }),
      },
    )

    const response = await POST(request, {
      params: Promise.resolve({ gameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(400)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("INVALID_PLAYER_ID")
  })
})
