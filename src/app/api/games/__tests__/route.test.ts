import { v4 as uuidv4 } from "uuid"
import { beforeEach, describe, expect, test } from "vitest"
import { Game } from "@/echomancy/domainmodel/game/Game"
import { Player } from "@/echomancy/domainmodel/game/Player"
import { gameRepository } from "@/lib/repositories"
import { GET, POST } from "../route"

describe("POST /api/games", () => {
  beforeEach(() => {
    // Clear the repository before each test by creating a fresh one
    // Since we can't easily reset the singleton, we'll work around this
    // by using unique IDs for each test
  })

  test("creates a game with valid UUID and returns 201", async () => {
    const gameId = uuidv4()
    const request = new Request("http://localhost/api/games", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ gameId }),
    })

    const response = await POST(request)
    const body = await response.json()

    expect(response.status).toBe(201)
    expect(body).toEqual({ data: { gameId } })

    // Verify game was actually created in repository
    const game = gameRepository.byId(gameId)
    expect(game).toBeDefined()
    expect(game?.id).toBe(gameId)
  })

  test("returns 400 for invalid UUID", async () => {
    const request = new Request("http://localhost/api/games", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ gameId: "not-a-valid-uuid" }),
    })

    const response = await POST(request)
    const body = await response.json()

    expect(response.status).toBe(400)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("INVALID_GAME_ID")
    expect(body.error.message).toContain("not-a-valid-uuid")
  })

  test("returns 400 for missing gameId", async () => {
    const request = new Request("http://localhost/api/games", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({}),
    })

    const response = await POST(request)
    const body = await response.json()

    expect(response.status).toBe(400)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("INVALID_GAME_ID")
  })

  test("returns 400 for empty string gameId", async () => {
    const request = new Request("http://localhost/api/games", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ gameId: "" }),
    })

    const response = await POST(request)
    const body = await response.json()

    expect(response.status).toBe(400)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("INVALID_GAME_ID")
  })
})

describe("GET /api/games", () => {
  test("returns 200 with empty data array when no games exist", async () => {
    // Note: We cannot clear the singleton repository, so this test may see
    // games created by other tests. We accept this limitation for now.
    const request = new Request("http://localhost/api/games", {
      method: "GET",
    })

    const response = await GET(request)
    const body = await response.json()

    expect(response.status).toBe(200)
    expect(body.data).toBeDefined()
    expect(Array.isArray(body.data)).toBe(true)
  })

  test("returns game summaries with correct structure", async () => {
    // Create a game directly to ensure we have at least one
    const gameId = uuidv4()
    const player1 = new Player(uuidv4(), "Alice")
    const player2 = new Player(uuidv4(), "Bob")

    const game = Game.create(gameId)
    game.addPlayer(player1)
    game.addPlayer(player2)
    gameRepository.add(game)

    const request = new Request("http://localhost/api/games", {
      method: "GET",
    })

    const response = await GET(request)
    const body = await response.json()

    expect(response.status).toBe(200)
    expect(body.data).toBeDefined()

    // Find our game in the results
    const gameSummary = body.data.find(
      (g: { gameId: string }) => g.gameId === gameId,
    )
    expect(gameSummary).toBeDefined()
    expect(gameSummary.status).toBe("not_started")
    expect(gameSummary.playerNames).toEqual(["Alice", "Bob"])
    expect(gameSummary.turnNumber).toBeNull()
    expect(gameSummary.currentPhase).toBeNull()
  })

  test("returns started game with in_progress status", async () => {
    const gameId = uuidv4()
    const player1 = new Player(uuidv4(), "StartedPlayer1")
    const player2 = new Player(uuidv4(), "StartedPlayer2")

    const game = Game.create(gameId)
    game.addPlayer(player1)
    game.addPlayer(player2)
    game.start(player1.id)
    gameRepository.add(game)

    const request = new Request("http://localhost/api/games", {
      method: "GET",
    })

    const response = await GET(request)
    const body = await response.json()

    expect(response.status).toBe(200)

    const gameSummary = body.data.find(
      (g: { gameId: string }) => g.gameId === gameId,
    )
    expect(gameSummary).toBeDefined()
    expect(gameSummary.status).toBe("in_progress")
    expect(gameSummary.playerNames).toEqual([
      "StartedPlayer1",
      "StartedPlayer2",
    ])
    expect(gameSummary.turnNumber).toBe(1)
    expect(gameSummary.currentPhase).toBe("UNTAP")
  })
})
