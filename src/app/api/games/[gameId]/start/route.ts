import { NextResponse } from "next/server"
import {
  StartGameCommand,
  StartGameCommandHandler,
} from "@/echomancy/application/command/start-game/StartGameCommand"
import {
  GameError,
  GameNotFoundError,
  InvalidGameIdError,
  InvalidPlayerIdError,
} from "@/echomancy/application/errors"
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
 *   400: { "error": { "code": "INVALID_GAME_ID" | "INVALID_PLAYER_ID", "message": "..." } }
 *   404: { "error": { "code": "GAME_NOT_FOUND", "message": "..." } }
 *   422: { "error": { "code": "...", "message": "..." } } - Domain validation errors
 */
export async function POST(
  request: Request,
  { params }: RouteParams,
): Promise<NextResponse> {
  try {
    const { gameId } = await params
    const body = (await request.json()) as StartGameRequest
    const { startingPlayerId } = body

    const handler = new StartGameCommandHandler(gameRepository)
    handler.handle(new StartGameCommand(gameId, startingPlayerId))

    return NextResponse.json({ data: { started: true } }, { status: 200 })
  } catch (error) {
    if (error instanceof InvalidGameIdError) {
      return NextResponse.json(
        { error: { code: "INVALID_GAME_ID", message: error.message } },
        { status: 400 },
      )
    }
    if (error instanceof InvalidPlayerIdError) {
      return NextResponse.json(
        { error: { code: "INVALID_PLAYER_ID", message: error.message } },
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
