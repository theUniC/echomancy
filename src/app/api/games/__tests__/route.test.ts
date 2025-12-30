import { v4 as uuidv4 } from "uuid"
import { beforeEach, describe, expect, test } from "vitest"
import { gameRepository } from "@/lib/repositories"
import { POST } from "../route"

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
