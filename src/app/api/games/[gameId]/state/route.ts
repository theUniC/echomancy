import { NextResponse } from "next/server"
import { validate as isValidUUID } from "uuid"
import { GameNotFoundError } from "@/echomancy/domainmodel/game/GameErrors"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
import { gameRepository } from "@/lib/repositories"

type RouteParams = {
  params: Promise<{ gameId: string }>
}

/**
 * GET /api/games/[gameId]/state
 *
 * Returns the current game state via game.exportState().
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

    if (!isValidUUID(gameId)) {
      throw new InvalidGameIdError(gameId)
    }

    const game = gameRepository.byId(gameId)
    if (!game) {
      throw new GameNotFoundError(gameId)
    }

    const state = game.exportState()

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
