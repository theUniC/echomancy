import { v4 as uuidv4 } from "uuid"
import type { CardInstance } from "../../cards/CardInstance"
import { ZoneNames } from "../../zones/Zone"
import { Game } from "../Game"
import { GameEventTypes } from "../GameEvents"
import { Player } from "../Player"
import type { AbilityOnStack, SpellOnStack, StackItem } from "../StackTypes"
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

export function createGameInMainPhase() {
  const setup = createStartedGame()
  advanceToStep(setup.game, Step.FIRST_MAIN)
  return setup
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

/**
 * Creates a creature with a simple ETB trigger that executes a callback.
 *
 * Useful for testing the trigger system without duplicating trigger definition code.
 *
 * @param instanceId - Unique ID for the creature
 * @param ownerId - Player who owns the creature
 * @param onETB - Callback to execute when trigger fires
 * @returns CardInstance with ETB trigger
 */
export function createCreatureWithETBTrigger(
  instanceId: string,
  ownerId: string,
  onETB: () => void,
): CardInstance {
  return {
    instanceId,
    definition: {
      id: "creature-with-etb",
      name: "Creature With ETB",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: (_game, event, source) =>
            event.card.instanceId === source.instanceId &&
            event.toZone === ZoneNames.BATTLEFIELD,
          effect: () => {
            onETB()
          },
        },
      ],
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

// ============================================================================
// ELF-THEMED CARD HELPERS (for trigger system validation)
// ============================================================================

/**
 * Creates Elvish Visionary card instance
 *
 * Real card text: "When Elvish Visionary enters the battlefield, draw a card."
 *
 * MVP implementation:
 * - ETB trigger fires when entering battlefield
 * - Calls game.drawCards() (currently no-op in MVP)
 * - No targeting required
 *
 * @param ownerId - Player who owns the card
 * @param drawCallback - Optional callback to track draw execution (for testing)
 */
export function createElvishVisionary(
  ownerId: string,
  drawCallback?: () => void,
): CardInstance {
  return {
    instanceId: `elvish-visionary-${Math.random()}`,
    definition: {
      id: "elvish-visionary",
      name: "Elvish Visionary",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: (_game, event, source) =>
            event.card.instanceId === source.instanceId &&
            event.toZone === ZoneNames.BATTLEFIELD,
          effect: (game, context) => {
            // Draw a card when entering battlefield
            game.drawCards(context.controllerId, 1)
            drawCallback?.()
          },
        },
      ],
    },
    ownerId,
  }
}

/**
 * Creates Llanowar Elves card instance
 *
 * Real card text: "{T}: Add {G}."
 *
 * MVP limitations:
 * - Activated ability NOT implemented (no mana system yet)
 * - Card serves as "another elf" for conditional triggers
 * - Still a valid creature on battlefield
 *
 * TODO: Implement activated abilities when mana system exists
 *
 * @param ownerId - Player who owns the card
 */
export function createLlanowarElves(ownerId: string): CardInstance {
  return {
    instanceId: `llanowar-elves-${Math.random()}`,
    definition: {
      id: "llanowar-elves",
      name: "Llanowar Elves",
      types: ["CREATURE"],
      // TODO: Add activatedAbility when mana system is implemented
      // activatedAbility: {
      //   cost: { type: "TAP" },
      //   effect: (game, context) => game.addMana(context.controllerId, "G", 1)
      // }
    },
    ownerId,
  }
}

/**
 * Creates Elvish Warrior card instance
 *
 * Real card: Vanilla 2/3 creature (no abilities)
 *
 * MVP purpose:
 * - Tests that creatures without triggers don't execute anything
 * - Serves as "another elf" for conditional triggers
 *
 * @param ownerId - Player who owns the card
 */
export function createElvishWarrior(ownerId: string): CardInstance {
  return {
    instanceId: `elvish-warrior-${Math.random()}`,
    definition: {
      id: "elvish-warrior",
      name: "Elvish Warrior",
      types: ["CREATURE"],
      // No triggers, no abilities - vanilla creature
    },
    ownerId,
  }
}

/**
 * Creates a test elf with conditional ETB trigger
 *
 * Conceptual card text:
 * "When this enters the battlefield, if you control another Elf, draw a card."
 *
 * Implementation notes:
 * - "Another" means "a different elf, not this card itself"
 * - Condition inspects battlefield state at trigger evaluation time
 * - Tests that triggers can have complex conditional logic
 *
 * @param ownerId - Player who owns the card
 * @param drawCallback - Optional callback to track draw execution (for testing)
 */
export function createConditionalElf(
  ownerId: string,
  drawCallback?: () => void,
): CardInstance {
  return {
    instanceId: `conditional-elf-${Math.random()}`,
    definition: {
      id: "conditional-elf",
      name: "Conditional Elf",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.ZONE_CHANGED,
          condition: (game, event, source) => {
            // Must be this card entering battlefield
            if (event.card.instanceId !== source.instanceId) return false
            if (event.toZone !== ZoneNames.BATTLEFIELD) return false

            // Check if controller has ANOTHER elf (excluding this one)
            const battlefield = game.getPlayerState(event.controllerId)
              .battlefield.cards

            const otherElves = battlefield.filter(
              (card) =>
                // Different card (not this one)
                card.instanceId !== source.instanceId &&
                // Is a creature
                card.definition.types.includes("CREATURE") &&
                // Card is an elf (MVP heuristic: check for "elf" or "elv" in ID or name)
                // In a real implementation, this would check creature subtypes
                (card.definition.id.includes("elf") ||
                  card.definition.id.includes("elv") ||
                  card.definition.name.toLowerCase().includes("elf") ||
                  card.definition.name.toLowerCase().includes("elv")),
            )

            return otherElves.length > 0
          },
          effect: (game, context) => {
            game.drawCards(context.controllerId, 1)
            drawCallback?.()
          },
        },
      ],
    },
    ownerId,
  }
}

/**
 * Creates an elf with attack trigger
 *
 * Conceptual card text:
 * "Whenever this creature attacks, draw a card."
 *
 * Tests that attack triggers work correctly.
 *
 * @param ownerId - Player who owns the card
 * @param attackCallback - Optional callback to track attack trigger execution
 */
export function createElfWithAttackTrigger(
  ownerId: string,
  attackCallback?: () => void,
): CardInstance {
  return {
    instanceId: `attacking-elf-${Math.random()}`,
    definition: {
      id: "attacking-elf",
      name: "Attacking Elf",
      types: ["CREATURE"],
      triggers: [
        {
          eventType: GameEventTypes.CREATURE_DECLARED_ATTACKER,
          condition: (_game, event, source) =>
            event.creature.instanceId === source.instanceId,
          effect: (game, context) => {
            game.drawCards(context.controllerId, 1)
            attackCallback?.()
          },
        },
      ],
    },
    ownerId,
  }
}
