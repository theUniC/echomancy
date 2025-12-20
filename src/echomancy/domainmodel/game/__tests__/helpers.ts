import { v4 as uuidv4 } from "uuid"
import type { CardInstance } from "../../cards/CardInstance"
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

export function createTestSpell(ownerId: string, instanceId?: string) {
  const id = instanceId || "test-spell-instance"
  return {
    instanceId: id,
    definition: {
      id: "test-spell",
      name: "Test Spell",
      category: "SPELL" as const,
      spellType: "INSTANT" as const,
    },
    ownerId,
  }
}

export function createSpell(
  instanceId: string,
  name: string,
  ownerId: string,
): CardInstance {
  return {
    instanceId,
    definition: {
      id: instanceId,
      name,
      category: "SPELL",
      spellType: "INSTANT",
    },
    ownerId,
  }
}

export function addSpellToHand(
  game: Game,
  playerId: string,
  spell: CardInstance,
): void {
  const playerState = game.getPlayerState(playerId)
  playerState.hand.cards.push(spell)
}

export function castSpellInMainPhase(game: Game, playerId: string) {
  const playerState = game.getPlayerState(playerId)
  const spellCard = createTestSpell(playerId)
  playerState.hand.cards.push(spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId,
    cardId: spellCard.instanceId,
    targets: [],
  })

  return spellCard
}

export function createTestCreature(
  ownerId: string,
  instanceId?: string,
): CardInstance {
  const id = instanceId || `test-creature-${Math.random()}`
  return {
    instanceId: id,
    definition: {
      id: "test-creature-def",
      name: "Test Creature",
      category: "SPELL" as const,
      spellType: "CREATURE" as const,
    },
    ownerId,
  }
}

export function addCreatureToHand(
  game: Game,
  playerId: string,
  creature: CardInstance,
): void {
  const playerState = game.getPlayerState(playerId)
  playerState.hand.cards.push(creature)
}

export function addCreatureToBattlefield(
  game: Game,
  playerId: string,
  creature: CardInstance,
): void {
  const playerState = game.getPlayerState(playerId)
  playerState.battlefield.cards.push(creature)
}
