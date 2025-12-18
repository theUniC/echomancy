import { v4 as uuidv4 } from "uuid"
import { Game } from "../Game"
import { Player } from "../Player"
import type { GameSteps } from "../Steps"

export function createStartedGame() {
  const player1 = new Player("p1")
  const player2 = new Player("p2")

  const game = Game.start({
    id: uuidv4(),
    players: [player1, player2],
    startingPlayerId: player1.id,
  })

  // The instanceId of the dummy land card in player1's hand
  const dummyLandInstanceId = `${player1.id}-dummy-land-instance`

  return { game, player1, player2, dummyLandInstanceId }
}

export function advanceToStep(game: Game, targetStep: GameSteps): void {
  while (game.currentStep !== targetStep) {
    game.apply({ type: "ADVANCE_STEP", playerId: game.currentPlayerId })
  }
}
