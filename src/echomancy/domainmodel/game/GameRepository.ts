import type { Game } from "@/echomancy/domainmodel/game/Game"

export interface GameRepository {
  add(game: Game): void
  byId(gameId: string): Game | undefined
  all(): Game[]
}
