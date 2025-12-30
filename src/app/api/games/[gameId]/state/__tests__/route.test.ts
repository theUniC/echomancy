import { v4 as uuidv4 } from "uuid"
import { beforeEach, describe, expect, test } from "vitest"
import { Game } from "@/echomancy/domainmodel/game/Game"
import { Player } from "@/echomancy/domainmodel/game/Player"
import { Step } from "@/echomancy/domainmodel/game/Steps"
import { gameRepository } from "@/lib/repositories"
import { GET } from "../route"

describe("GET /api/games/[gameId]/state", () => {
  let gameId: string
  let player1Id: string
  let player2Id: string

  beforeEach(() => {
    // Create a started game with two players for each test
    gameId = uuidv4()
    player1Id = uuidv4()
    player2Id = uuidv4()

    const game = Game.create(gameId)
    game.addPlayer(new Player(player1Id, "Player 1"))
    game.addPlayer(new Player(player2Id, "Player 2"))
    game.start(player1Id)
    gameRepository.add(game)
  })

  test("returns game state with 200", async () => {
    const request = new Request(`http://localhost/api/games/${gameId}/state`, {
      method: "GET",
    })

    const response = await GET(request, { params: Promise.resolve({ gameId }) })
    const body = await response.json()

    expect(response.status).toBe(200)
    expect(body.data).toBeDefined()

    // Verify the state structure
    expect(body.data.gameId).toBe(gameId)
    expect(body.data.currentPlayerId).toBe(player1Id)
    expect(body.data.currentStep).toBe(Step.UNTAP)
    expect(body.data.currentTurnNumber).toBe(1)
    expect(body.data.turnOrder).toEqual([player1Id, player2Id])
    expect(body.data.players).toBeDefined()
    expect(body.data.players[player1Id]).toBeDefined()
    expect(body.data.players[player2Id]).toBeDefined()
    expect(body.data.stack).toEqual([])
  })

  test("returns 404 when game does not exist", async () => {
    const nonExistentGameId = uuidv4()

    const request = new Request(
      `http://localhost/api/games/${nonExistentGameId}/state`,
      {
        method: "GET",
      },
    )

    const response = await GET(request, {
      params: Promise.resolve({ gameId: nonExistentGameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(404)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("GAME_NOT_FOUND")
  })

  test("returns 400 for invalid gameId UUID", async () => {
    const request = new Request(
      `http://localhost/api/games/invalid-uuid/state`,
      {
        method: "GET",
      },
    )

    const response = await GET(request, {
      params: Promise.resolve({ gameId: "invalid-uuid" }),
    })
    const body = await response.json()

    expect(response.status).toBe(400)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("INVALID_GAME_ID")
  })

  test("returns player zones in state", async () => {
    const request = new Request(`http://localhost/api/games/${gameId}/state`, {
      method: "GET",
    })

    const response = await GET(request, { params: Promise.resolve({ gameId }) })
    const body = await response.json()

    expect(response.status).toBe(200)

    // Check player state structure
    const player1State = body.data.players[player1Id]
    expect(player1State).toBeDefined()
    expect(player1State.zones).toBeDefined()
    expect(player1State.zones.hand).toBeDefined()
    expect(player1State.zones.battlefield).toBeDefined()
    expect(player1State.zones.graveyard).toBeDefined()
    expect(player1State.manaPool).toBeDefined()
    expect(player1State.lifeTotal).toBeDefined()
  })
})
