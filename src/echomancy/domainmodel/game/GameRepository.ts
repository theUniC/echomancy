import { Game } from "./Game";

export interface GameRepository {
    add(game: Game): void
    byId(gameId: string): Game|undefined
}