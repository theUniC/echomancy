import { v4 as uuidv4 } from "uuid"
import type { CardInstance } from "../../cards/CardInstance"
import { PrebuiltDecks } from "../../cards/PrebuiltDecks"
import { ZoneNames } from "../../zones/Zone"
import { Game } from "../Game"
import { GameEventTypes } from "../GameEvents"
import { Player } from "../Player"
import type { AbilityOnStack, SpellOnStack, StackItem } from "../StackTypes"
import { type GameSteps, Step } from "../Steps"

/**
 * Creates a test player with a UUID and optional custom name.
 * This helper encapsulates player creation to reduce coupling in tests.
 *
 * @param name - Optional custom name for the player (defaults to "Test Player")
 * @param id - Optional custom UUID (defaults to generated UUID)
 * @returns A new Player instance
 */
export function createTestPlayer(name?: string, id?: string): Player {
  return new Player(id ?? uuidv4(), name ?? "Test Player")
}

/**
 * Create a game that has been fully started.
 *
 * Uses the new lifecycle API:
 * 1. Game.create()
 * 2. game.addPlayer()
 * 3. game.start()
 *
 * This is the preferred way to create test games going forward.
 *
 * NOTE: Creates games with EMPTY libraries. If your test advances to turn 2+
 * and goes through a draw step, use createStartedGameWithDecks() instead to
 * avoid triggering empty library loss.
 */
export function createStartedGame() {
  const player1 = createTestPlayer("Player 1")
  const player2 = createTestPlayer("Player 2")

  const game = Game.create(uuidv4())
  game.addPlayer(player1)
  game.addPlayer(player2)
  game.start(player1.id)

  return { game, player1, player2 }
}

/**
 * Create a game that has been fully started with prebuilt decks.
 *
 * This version creates players with 60-card decks loaded into their libraries.
 * After drawing 7-card opening hands, each player has 53 cards in library.
 *
 * Use this when your test needs to advance through multiple turns without
 * triggering empty library loss conditions.
 */
export function createStartedGameWithDecks() {
  const player1 = createTestPlayer("Player 1")
  const player2 = createTestPlayer("Player 2")

  const game = Game.create(uuidv4())
  game.addPlayer(player1)
  game.addPlayer(player2)

  const deck1 = PrebuiltDecks.greenDeck(player1.id)
  const deck2 = PrebuiltDecks.redDeck(player2.id)

  game.start(player1.id, {
    decks: {
      [player1.id]: deck1,
      [player2.id]: deck2,
    },
  })

  return { game, player1, player2 }
}

export function createGameInMainPhase() {
  const setup = createStartedGame()
  advanceToStep(setup.game, Step.FIRST_MAIN)
  return setup
}

/**
 * Creates a game in FIRST_MAIN phase with a land in player1's hand.
 * This is a common test setup pattern to reduce boilerplate.
 *
 * @returns Game setup with land included
 */
export function createGameInMainPhaseWithLand() {
  const setup = createGameInMainPhase()
  const land = addTestLandToHand(setup.game, setup.player1.id)
  return { ...setup, land }
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

/**
 * Creates a test spell with a mana cost.
 * This is a convenience helper for testing mana payment during spell casting.
 *
 * @param ownerId - Player who owns the spell
 * @param manaCostString - Mana cost in string format (e.g., "2UU", "BBB", "4")
 * @param instanceId - Optional custom instance ID
 * @returns CardInstance with the specified mana cost
 *
 * @example
 * const spell = createTestSpellWithManaCost(player1.id, "2UU")
 * // Creates a spell costing 2 generic + 2 blue mana
 */
export function createTestSpellWithManaCost(
  ownerId: string,
  manaCostString: string,
  instanceId?: string,
): CardInstance {
  const { ManaCostParser } = require("../valueobjects/ManaCost")
  const id = instanceId || `test-spell-${Math.random()}`
  return {
    instanceId: id,
    definition: {
      id: "test-spell-with-cost",
      name: "Test Spell With Cost",
      types: ["INSTANT"],
      manaCost: ManaCostParser.parse(manaCostString),
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

export function createTestLand(
  ownerId: string,
  instanceId?: string,
): CardInstance {
  const id = instanceId || `test-land-${Math.random()}`
  return {
    instanceId: id,
    definition: {
      id: "test-land-def",
      name: "Test Land",
      types: ["LAND"],
    },
    ownerId,
  }
}

export function addLandToHand(
  game: Game,
  playerId: string,
  land: CardInstance,
): void {
  const playerState = game.getPlayerState(playerId)
  playerState.hand.cards.push(land)
}

/**
 * Creates a test land and adds it to the player's hand.
 * This is a convenience helper to reduce boilerplate in tests.
 *
 * @param game - The game instance
 * @param playerId - The player who will receive the land
 * @param instanceId - Optional custom instance ID for the land
 * @returns The created land card
 */
export function addTestLandToHand(
  game: Game,
  playerId: string,
  instanceId?: string,
): CardInstance {
  const land = createTestLand(playerId, instanceId)
  addLandToHand(game, playerId, land)
  return land
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
  power?: number,
  toughness?: number,
): CardInstance {
  const id = instanceId || `test-creature-${Math.random()}`
  return {
    instanceId: id,
    definition: {
      id: "test-creature-def",
      name: "Test Creature",
      types: ["CREATURE"],
      power,
      toughness,
    },
    ownerId,
  }
}

/**
 * Creates a creature with the Flash static ability.
 * Flash allows a creature to be cast any time you could cast an instant.
 *
 * @param ownerId - Player who owns the creature
 * @param instanceId - Optional unique ID for the creature
 * @param power - Optional power value
 * @param toughness - Optional toughness value
 * @returns CardInstance with FLASH ability
 */
export function createCreatureWithFlash(
  ownerId: string,
  instanceId?: string,
  power?: number,
  toughness?: number,
): CardInstance {
  const id = instanceId || `flash-creature-${Math.random()}`
  return {
    instanceId: id,
    definition: {
      id: "flash-creature-def",
      name: "Flash Creature",
      types: ["CREATURE"],
      power,
      toughness,
      staticAbilities: ["FLASH"],
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
  power?: number,
  toughness?: number,
): CardInstance {
  return {
    instanceId,
    definition: {
      id: "creature-with-etb",
      name: "Creature With ETB",
      types: ["CREATURE"],
      power,
      toughness,
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
 *
 * NOTE: This helper clears summoning sickness by default for test convenience.
 * Most tests were written before summoning sickness was implemented and expect
 * creatures to be able to attack immediately. Use addCreatureToBattlefieldWithSummoningSickness
 * if you need to test summoning sickness behavior.
 */
export function addCreatureToBattlefield(
  game: Game,
  playerId: string,
  creature: CardInstance,
): void {
  game.enterBattlefield(creature, playerId)

  // Clear summoning sickness for test convenience (most tests were written
  // before summoning sickness was implemented)
  // Use private method access to get mutable state
  // biome-ignore lint/suspicious/noExplicitAny: Test helper needs private access
  const permanentState = (game as any).getPermanentStateOrThrow(
    creature.instanceId,
  )
  if (permanentState) {
    // biome-ignore lint/suspicious/noExplicitAny: Test helper needs private access
    ;(game as any).permanentStates.set(
      creature.instanceId,
      permanentState.withSummoningSickness(false),
    )
  }
}

/**
 * Adds a creature to the battlefield WITH summoning sickness.
 * Use this when you need to test summoning sickness behavior.
 */
export function addCreatureToBattlefieldWithSummoningSickness(
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
      power: 1,
      toughness: 1,
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
 * - Activated ability NOT YET ADDED (mana pool exists, but not wired to this card yet)
 * - Card serves as "another elf" for conditional triggers
 * - Still a valid creature on battlefield
 *
 * TODO: Add the activated ability now that mana pool exists
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
      power: 1,
      toughness: 1,
      // TODO: Add activatedAbility now that mana pool is implemented
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
      power: 2,
      toughness: 3,
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
      power: 1,
      toughness: 1,
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
      power: 2,
      toughness: 2,
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

// ============================================================================
// PERMANENT TYPE HELPERS (Artifacts, Enchantments, Planeswalkers)
// ============================================================================

/**
 * Creates a basic artifact card instance
 *
 * @param ownerId - Player who owns the artifact
 * @param instanceId - Optional unique ID for the artifact
 * @returns CardInstance with ARTIFACT type
 */
export function createTestArtifact(
  ownerId: string,
  instanceId?: string,
): CardInstance {
  const id = instanceId || `test-artifact-${Math.random()}`
  return {
    instanceId: id,
    definition: {
      id: "test-artifact-def",
      name: "Test Artifact",
      types: ["ARTIFACT"],
    },
    ownerId,
  }
}

/**
 * Creates a basic enchantment card instance
 *
 * @param ownerId - Player who owns the enchantment
 * @param instanceId - Optional unique ID for the enchantment
 * @returns CardInstance with ENCHANTMENT type
 */
export function createTestEnchantment(
  ownerId: string,
  instanceId?: string,
): CardInstance {
  const id = instanceId || `test-enchantment-${Math.random()}`
  return {
    instanceId: id,
    definition: {
      id: "test-enchantment-def",
      name: "Test Enchantment",
      types: ["ENCHANTMENT"],
    },
    ownerId,
  }
}

/**
 * Creates a basic planeswalker card instance
 *
 * MVP Note: Planeswalkers exist as permanents but do not have loyalty
 * counters or loyalty abilities in the MVP. This is a placeholder for
 * future expansion.
 *
 * @param ownerId - Player who owns the planeswalker
 * @param instanceId - Optional unique ID for the planeswalker
 * @returns CardInstance with PLANESWALKER type
 */
export function createTestPlaneswalker(
  ownerId: string,
  instanceId?: string,
): CardInstance {
  const id = instanceId || `test-planeswalker-${Math.random()}`
  return {
    instanceId: id,
    definition: {
      id: "test-planeswalker-def",
      name: "Test Planeswalker",
      types: ["PLANESWALKER"],
    },
    ownerId,
  }
}

/**
 * Creates an artifact creature (multiple types on single card)
 *
 * @param ownerId - Player who owns the artifact creature
 * @param instanceId - Optional unique ID for the artifact creature
 * @returns CardInstance with both ARTIFACT and CREATURE types
 */
export function createTestArtifactCreature(
  ownerId: string,
  instanceId?: string,
): CardInstance {
  const id = instanceId || `test-artifact-creature-${Math.random()}`
  return {
    instanceId: id,
    definition: {
      id: "test-artifact-creature-def",
      name: "Test Artifact Creature",
      types: ["ARTIFACT", "CREATURE"],
    },
    ownerId,
  }
}

/**
 * Adds an artifact to the battlefield for the given player.
 * Uses game.enterBattlefield() to ensure consistent ETB handling.
 *
 * @param game - The game instance
 * @param playerId - The player who controls the artifact
 * @param artifact - The artifact card instance
 */
export function addArtifactToBattlefield(
  game: Game,
  playerId: string,
  artifact: CardInstance,
): void {
  game.enterBattlefield(artifact, playerId)
}

/**
 * Adds an enchantment to the battlefield for the given player.
 * Uses game.enterBattlefield() to ensure consistent ETB handling.
 *
 * @param game - The game instance
 * @param playerId - The player who controls the enchantment
 * @param enchantment - The enchantment card instance
 */
export function addEnchantmentToBattlefield(
  game: Game,
  playerId: string,
  enchantment: CardInstance,
): void {
  game.enterBattlefield(enchantment, playerId)
}

/**
 * Adds a planeswalker to the battlefield for the given player.
 * Uses game.enterBattlefield() to ensure consistent ETB handling.
 *
 * @param game - The game instance
 * @param playerId - The player who controls the planeswalker
 * @param planeswalker - The planeswalker card instance
 */
export function addPlaneswalkerToBattlefield(
  game: Game,
  playerId: string,
  planeswalker: CardInstance,
): void {
  game.enterBattlefield(planeswalker, playerId)
}
