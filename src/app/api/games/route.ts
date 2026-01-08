import { NextResponse } from "next/server"
import {
  CreateGameCommand,
  CreateGameCommandHandler,
} from "@/echomancy/application/command/create-game/CreateGameCommand"
import { InvalidGameIdError } from "@/echomancy/application/errors"
import {
  ListGamesQuery,
  ListGamesQueryHandler,
} from "@/echomancy/application/query/list-games/ListGamesQuery"
import { gameRepository } from "@/lib/repositories"

type CreateGameRequest = {
  gameId: string
}

/**
 * POST /api/games
 *
 * Creates a new game with the provided game ID.
 *
 * Request body:
 *   { "gameId": "uuid" }
 *
 * Response:
 *   201: { "data": { "gameId": "..." } }
 *   400: { "error": { "code": "INVALID_GAME_ID", "message": "..." } }
 */
export async function POST(request: Request): Promise<NextResponse> {
  try {
    const body = (await request.json()) as CreateGameRequest
    const { gameId } = body

    const handler = new CreateGameCommandHandler(gameRepository)
    handler.handle(new CreateGameCommand(gameId))

    return NextResponse.json({ data: { gameId } }, { status: 201 })
  } catch (error) {
    if (error instanceof InvalidGameIdError) {
      return NextResponse.json(
        { error: { code: "INVALID_GAME_ID", message: error.message } },
        { status: 400 },
      )
    }
    throw error
  }
}

/**
 * GET /api/games
 *
 * Lists all games with their summaries.
 *
 * Response:
 *   200: { "data": GameSummary[] }
 */
export async function GET(): Promise<NextResponse> {
  const handler = new ListGamesQueryHandler(gameRepository)
  const games = handler.handle(new ListGamesQuery())

  return NextResponse.json({ data: games })
}
