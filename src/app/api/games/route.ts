import { NextResponse } from "next/server"
import {
  CreateGameCommand,
  CreateGameCommandHandler,
} from "@/echomancy/application/command/create-game/CreateGameCommand"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
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
