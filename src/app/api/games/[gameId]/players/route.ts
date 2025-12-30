import { NextResponse } from "next/server"
import {
  JoinGameCommand,
  JoinGameCommandHandler,
} from "@/echomancy/application/command/join-game/JoinGameCommand"
import {
  GameNotFoundError,
  InvalidPlayerIdError,
} from "@/echomancy/domainmodel/game/GameErrors"
import { InvalidGameIdError } from "@/echomancy/domainmodel/game/InvalidGameIdError"
import { gameRepository } from "@/lib/repositories"

type JoinGameRequest = {
  playerId: string
  playerName: string
}

type RouteParams = {
  params: Promise<{ gameId: string }>
}

/**
 * POST /api/games/[gameId]/players
 *
 * Adds a player to an existing game.
 *
 * Request body:
 *   { "playerId": "uuid", "playerName": "Player 1" }
 *
 * Response:
 *   201: { "data": { "playerId": "...", "playerName": "..." } }
 *   400: { "error": { "code": "INVALID_GAME_ID" | "INVALID_PLAYER_ID", "message": "..." } }
 *   404: { "error": { "code": "GAME_NOT_FOUND", "message": "..." } }
 */
export async function POST(
  request: Request,
  { params }: RouteParams,
): Promise<NextResponse> {
  try {
    const { gameId } = await params
    const body = (await request.json()) as JoinGameRequest
    const { playerId, playerName } = body

    const handler = new JoinGameCommandHandler(gameRepository)
    handler.handle(new JoinGameCommand(gameId, playerId, playerName))

    return NextResponse.json(
      { data: { playerId, playerName } },
      { status: 201 },
    )
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
    throw error
  }
}
