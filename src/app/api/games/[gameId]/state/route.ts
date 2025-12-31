import { NextResponse } from "next/server"
import {
  GameNotFoundError,
  InvalidGameIdError,
} from "@/echomancy/application/errors"
import {
  GetGameStateQuery,
  GetGameStateQueryHandler,
} from "@/echomancy/application/query/get-game-state/GetGameStateQuery"
import { gameRepository } from "@/lib/repositories"

type RouteParams = {
  params: Promise<{ gameId: string }>
}

/**
 * GET /api/games/[gameId]/state
 *
 * Returns the current game state via GetGameStateQuery.
 *
 * Response:
 *   200: { "data": {...state} }
 *   400: { "error": { "code": "INVALID_GAME_ID", "message": "..." } }
 *   404: { "error": { "code": "GAME_NOT_FOUND", "message": "..." } }
 */
export async function GET(
  _request: Request,
  { params }: RouteParams,
): Promise<NextResponse> {
  try {
    const { gameId } = await params

    const handler = new GetGameStateQueryHandler(gameRepository)
    const state = handler.handle(new GetGameStateQuery(gameId))

    return NextResponse.json({ data: state }, { status: 200 })
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
    throw error
  }
}
