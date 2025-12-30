import { NextResponse } from "next/server"
import { validate as isValidUUID } from "uuid"
import {
  GameError,
  GameNotFoundError,
} from "@/echomancy/domainmodel/game/GameErrors"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
import { gameRepository } from "@/lib/repositories"

type StartGameRequest = {
  startingPlayerId: string
}

type RouteParams = {
  params: Promise<{ gameId: string }>
}

/**
 * POST /api/games/[gameId]/start
 *
 * Starts a game with the specified starting player.
 *
 * Request body:
 *   { "startingPlayerId": "uuid" }
 *
 * Response:
 *   200: { "data": { "started": true } }
 *   400: { "error": { "code": "INVALID_GAME_ID", "message": "..." } }
 *   404: { "error": { "code": "GAME_NOT_FOUND", "message": "..." } }
 *   422: { "error": { "code": "...", "message": "..." } } - Domain validation errors
 */
export async function POST(
  request: Request,
  { params }: RouteParams,
): Promise<NextResponse> {
  try {
    const { gameId } = await params

    if (!isValidUUID(gameId)) {
      throw new InvalidGameIdError(gameId)
    }

    const game = gameRepository.byId(gameId)
    if (!game) {
      throw new GameNotFoundError(gameId)
    }

    const body = (await request.json()) as StartGameRequest
    const { startingPlayerId } = body

    game.start(startingPlayerId)

    return NextResponse.json({ data: { started: true } }, { status: 200 })
  } catch (error) {
    if (error instanceof InvalidGameIdError) {
      return NextResponse.json(
        { error: { code: "INVALID_GAME_ID", message: error.message } },
        { status: 400 },
      )
    }
    if (error instanceof GameNotFoundError) {
      return NextResponse.json(
        { error: { code: "GAME_NOT_FOUND", message: error.message } },
        { status: 404 },
      )
    }
    if (error instanceof GameError) {
      return NextResponse.json(
        { error: { code: error.name, message: error.message } },
        { status: 422 },
      )
    }
    throw error
  }
}
