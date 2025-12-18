import { v4 as uuidv4 } from "uuid"
import { Game } from "../Game"
import type { GameSteps } from "../Steps"
import { Player } from "../Player"

export const DUMMY_CARD_ID = "test-card-id"

export function createStartedGame() {
  const player1 = new Player("p1")
  const player2 = new Player("p2")

  const game = Game.start({
    id: uuidv4(),
    players: [player1, player2],
    startingPlayerId: player1.id,
  })

  return { game, player1, player2 }
}

export function advanceToStep(game: Game, targetStep: GameSteps): void {
  while (game.currentStep !== targetStep) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }
}
