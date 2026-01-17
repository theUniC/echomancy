import { NextResponse } from "next/server"
import {
  GameNotFoundError,
  InvalidGameIdError,
} from "@/echomancy/application/errors"
import {
  GetAllowedActionsQuery,
  GetAllowedActionsQueryHandler,
} from "@/echomancy/application/query/get-allowed-actions/GetAllowedActionsQuery"
import { gameRepository } from "@/lib/repositories"

type RouteParams = {
  params: Promise<{ gameId: string }>
}

/**
 * GET /api/games/[gameId]/allowed-actions?playerId={playerId}
 *
 * Returns the allowed actions for a specific player via GetAllowedActionsQuery.
 * For now, this focuses on PLAY_LAND actions (which land cards can be played).
 *
 * Query Parameters:
 *   - playerId (required): The ID of the player to check allowed actions for
 *
 * Response:
 *   200: { "data": { "playableLands": ["cardId1", "cardId2"] } }
 *   400: { "error": { "code": "INVALID_GAME_ID" | "MISSING_PLAYER_ID", "message": "..." } }
 *   404: { "error": { "code": "GAME_NOT_FOUND", "message": "..." } }
 */
export async function GET(
  request: Request,
  { params }: RouteParams,
): Promise<NextResponse> {
  try {
    const { gameId } = await params
    const { searchParams } = new URL(request.url)
    const playerId = searchParams.get("playerId")

    if (!playerId) {
      return NextResponse.json(
        {
          error: {
            code: "MISSING_PLAYER_ID",
            message: "Query parameter 'playerId' is required",
          },
        },
        { status: 400 },
      )
    }

    const handler = new GetAllowedActionsQueryHandler(gameRepository)
    const result = handler.handle(new GetAllowedActionsQuery(gameId, playerId))

    return NextResponse.json({ data: result }, { status: 200 })
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
