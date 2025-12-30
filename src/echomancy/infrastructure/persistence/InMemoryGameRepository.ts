import type { Game } from "@/echomancy/domainmodel/game/Game"
import type { GameRepository } from "@/echomancy/domainmodel/game/GameRepository"

export class InMemoryGameRepository implements GameRepository {
  private games: Map<string, Game> = new Map()

  add(game: Game): void {
    this.games.set(game.id, game)
  }

  byId(gameId: string): Game | undefined {
    return this.games.get(gameId)
  }
}
