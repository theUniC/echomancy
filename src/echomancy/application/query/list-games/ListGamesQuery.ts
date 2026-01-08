import { GameLifecycleState } from "@/echomancy/domainmodel/game/Game"
import type { GameRepository } from "@/echomancy/domainmodel/game/GameRepository"

/**
 * Summary of a game for listing purposes.
 * Contains minimal information needed to display a list of games.
 */
export type GameSummary = {
  gameId: string
  status: "not_started" | "in_progress" | "finished"
  playerNames: string[]
  turnNumber: number | null
  currentPhase: string | null
}

export class ListGamesQuery {}

export class ListGamesQueryHandler {
  constructor(private gameRepository: GameRepository) {}

  handle(_query: ListGamesQuery): GameSummary[] {
    const games = this.gameRepository.all()

    return games.map((game) => {
      const status = this.mapLifecycleStatus(game.lifecycleState)
      const isStarted = game.lifecycleState !== GameLifecycleState.CREATED
      const players = game.getPlayers()

      return {
        gameId: game.id,
        status,
        playerNames: players.map((p) => p.name),
        turnNumber: isStarted ? game.currentTurnNumber : null,
        currentPhase: isStarted ? game.currentStep : null,
      }
    })
  }

  private mapLifecycleStatus(
    lifecycleState: GameLifecycleState,
  ): GameSummary["status"] {
    switch (lifecycleState) {
      case GameLifecycleState.CREATED:
        return "not_started"
      case GameLifecycleState.STARTED:
        return "in_progress"
      case GameLifecycleState.FINISHED:
        return "finished"
    }
  }
}
