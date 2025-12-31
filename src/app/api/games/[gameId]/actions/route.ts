import { NextResponse } from "next/server"
import {
  ApplyActionCommand,
  ApplyActionCommandHandler,
} from "@/echomancy/application/command/apply-action/ApplyActionCommand"
import {
  GameError,
  GameNotFoundError,
  InvalidGameIdError,
} from "@/echomancy/application/errors"
import {
  GetGameStateQuery,
  GetGameStateQueryHandler,
} from "@/echomancy/application/query/get-game-state/GetGameStateQuery"
import type { Actions } from "@/echomancy/application/types"
import { gameRepository } from "@/lib/repositories"

type RouteParams = {
  params: Promise<{ gameId: string }>
}

/**
 * POST /api/games/[gameId]/actions
 *
 * Applies a game action via ApplyActionCommand.
 *
 * Request body:
 *   Any valid action JSON, e.g.:
 *   { "type": "ADVANCE_STEP", "playerId": "uuid" }
 *   { "type": "PLAY_LAND", "playerId": "uuid", "cardId": "card-id" }
 *
 * Response:
 *   200: { "data": { "state": {...} } } - Returns full game state after action
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
    const action = (await request.json()) as Actions

    // Apply the action
    const applyHandler = new ApplyActionCommandHandler(gameRepository)
    applyHandler.handle(new ApplyActionCommand(gameId, action))

    // Get updated state
    const queryHandler = new GetGameStateQueryHandler(gameRepository)
    const state = queryHandler.handle(new GetGameStateQuery(gameId))

    return NextResponse.json({ data: { state } }, { status: 200 })
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
