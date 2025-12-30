import { v4 as uuidv4 } from "uuid"
import { beforeEach, describe, expect, test } from "vitest"
import { Game } from "@/echomancy/domainmodel/game/Game"
import { Player } from "@/echomancy/domainmodel/game/Player"
import { Step } from "@/echomancy/domainmodel/game/Steps"
import { gameRepository } from "@/lib/repositories"
import { POST } from "../route"

describe("POST /api/games/[gameId]/actions", () => {
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

  test("applies ADVANCE_STEP action and returns state with 200", async () => {
    const request = new Request(
      `http://localhost/api/games/${gameId}/actions`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          type: "ADVANCE_STEP",
          playerId: player1Id,
        }),
      },
    )

    const response = await POST(request, {
      params: Promise.resolve({ gameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(200)
    expect(body.data).toBeDefined()
    expect(body.data.state).toBeDefined()
    expect(body.data.state.gameId).toBe(gameId)

    // Verify the step actually advanced (UNTAP -> UPKEEP)
    const game = gameRepository.byId(gameId)
    expect(game?.currentStep).toBe(Step.UPKEEP)
  })

  test("applies END_TURN action and returns state with 200", async () => {
    const request = new Request(
      `http://localhost/api/games/${gameId}/actions`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          type: "END_TURN",
          playerId: player1Id,
        }),
      },
    )

    const response = await POST(request, {
      params: Promise.resolve({ gameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(200)
    expect(body.data.state).toBeDefined()
    // END_TURN sets auto-pass mode; the turn advances through CLEANUP to the next turn
    // The current player should now be player2 (since player1 ended their turn)
    expect(body.data.state.currentPlayerId).toBe(player2Id)
  })

  test("returns 404 when game does not exist", async () => {
    const nonExistentGameId = uuidv4()

    const request = new Request(
      `http://localhost/api/games/${nonExistentGameId}/actions`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          type: "ADVANCE_STEP",
          playerId: player1Id,
        }),
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
      `http://localhost/api/games/invalid-uuid/actions`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          type: "ADVANCE_STEP",
          playerId: player1Id,
        }),
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

  test("returns 422 when action is invalid (wrong player)", async () => {
    // Player 2 trying to advance step when it's Player 1's turn
    const request = new Request(
      `http://localhost/api/games/${gameId}/actions`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          type: "ADVANCE_STEP",
          playerId: player2Id,
        }),
      },
    )

    const response = await POST(request, {
      params: Promise.resolve({ gameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(422)
    expect(body.error).toBeDefined()
    expect(body.error.code).toBe("InvalidPlayerActionError")
  })

  test("returns 422 when action violates game rules", async () => {
    // Try to play a land when we don't have one in hand
    const request = new Request(
      `http://localhost/api/games/${gameId}/actions`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          type: "PLAY_LAND",
          playerId: player1Id,
          cardId: "non-existent-card",
        }),
      },
    )

    const response = await POST(request, {
      params: Promise.resolve({ gameId }),
    })
    const body = await response.json()

    expect(response.status).toBe(422)
    expect(body.error).toBeDefined()
    // Either InvalidPlayLandStepError or CardNotFoundInHandError
  })
})
