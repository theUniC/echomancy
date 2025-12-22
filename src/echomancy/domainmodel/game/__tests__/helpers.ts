import { v4 as uuidv4 } from "uuid"
import type { CardInstance } from "../../cards/CardInstance"
import { type AbilityOnStack, Game, type SpellOnStack } from "../Game"
import { Player } from "../Player"
import { type GameSteps, Step } from "../Steps"

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

export function createTestSpell(
  ownerId: string,
  instanceId?: string,
): CardInstance {
  const id = instanceId || "test-spell-instance"
  return {
    instanceId: id,
    definition: {
      id: "test-spell",
      name: "Test Spell",
      types: ["INSTANT"],
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
      types: ["INSTANT"],
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
      types: ["CREATURE"],
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

/**
 * Adds a creature to the battlefield for the given player.
 * IMPORTANT: This helper MUST use game.enterBattlefield() to ensure
 * consistent ETB handling across all code paths (production and tests).
 */
export function addCreatureToBattlefield(
  game: Game,
  playerId: string,
  creature: CardInstance,
): void {
  game.enterBattlefield(creature, playerId)
}

export function setupCreatureInCombat(
  game: Game,
  playerId: string,
  creatureId?: string,
): CardInstance {
  const creature = createTestCreature(playerId, creatureId)
  addCreatureToBattlefield(game, playerId, creature)
  advanceToStep(game, Step.DECLARE_ATTACKERS)
  return creature
}

export function setupMultipleCreatures(
  game: Game,
  playerId: string,
  count: number,
): CardInstance[] {
  const creatures: CardInstance[] = []
  for (let i = 0; i < count; i++) {
    const creature = createTestCreature(playerId, `creature-${i + 1}`)
    addCreatureToBattlefield(game, playerId, creature)
    creatures.push(creature)
  }
  return creatures
}

export function scheduleExtraCombatPhase(game: Game): void {
  game.addScheduledSteps([
    Step.BEGINNING_OF_COMBAT,
    Step.DECLARE_ATTACKERS,
    Step.DECLARE_BLOCKERS,
    Step.COMBAT_DAMAGE,
    Step.END_OF_COMBAT,
  ])
}

export function resolveStack(
  game: Game,
  opponentId: string,
  controllerId: string,
): void {
  game.apply({ type: "PASS_PRIORITY", playerId: opponentId })
  game.apply({ type: "PASS_PRIORITY", playerId: controllerId })
}

/**
 * Asserts that a stack item at a given index is a spell and returns it with proper typing.
 * Throws an error if the item is not a spell.
 */
export function assertSpellAt(
  stack: readonly StackItem[],
  index: number,
): SpellOnStack {
  const item = stack[index]
  if (!item) {
    throw new Error(`No stack item found at index ${index}`)
  }
  if (item.kind !== "SPELL") {
    throw new Error(
      `Expected SPELL at stack index ${index}, but got ${item.kind}`,
    )
  }
  return item
}

/**
 * Asserts that a stack item at a given index is an ability and returns it with proper typing.
 * Throws an error if the item is not an ability.
 */
export function assertAbilityAt(
  stack: readonly StackItem[],
  index: number,
): AbilityOnStack {
  const item = stack[index]
  if (!item) {
    throw new Error(`No stack item found at index ${index}`)
  }
  if (item.kind !== "ABILITY") {
    throw new Error(
      `Expected ABILITY at stack index ${index}, but got ${item.kind}`,
    )
  }
  return item
}
