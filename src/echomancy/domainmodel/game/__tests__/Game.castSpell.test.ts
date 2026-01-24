import { describe, expect, test } from "vitest"
import {
  CardIsNotSpellError,
  CardNotFoundInHandError,
  InvalidPlayerActionError,
  NotMainPhaseError,
  NotYourTurnError,
  StackNotEmptyError,
} from "../GameErrors"
import { Step } from "../Steps"
import {
  addSpellToHand,
  advanceToStep,
  assertSpellAt,
  createCreatureWithFlash,
  createGameInMainPhaseWithLand,
  createStartedGame,
  createTestCreature,
  createTestSpell,
} from "./helpers"

test("it moves a spell card from hand to stack when casting a spell", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard = createTestSpell(player1.id)
  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
    targets: [],
  })

  const stateAfter = game.getPlayerState(player1.id)
  const stack = game.getStack()

  expect(stateAfter.hand.cards).toHaveLength(0) // Spell moved to stack
  expect(stack).toHaveLength(1)
  expect(stateAfter.battlefield.cards).toHaveLength(0)
})

test("it pushes the same card instance onto the stack", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard = createTestSpell(player1.id)
  addSpellToHand(game, player1.id, spellCard)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
    targets: [],
  })

  const stack = game.getStack()
  const spell = assertSpellAt(stack, 0)

  expect(spell.card.instanceId).toBe(spellCard.instanceId)
  expect(spell.card.definition.id).toBe(spellCard.definition.id)
  expect(spell.controllerId).toBe(player1.id)
})

test("it throws error when trying to cast sorcery-speed spell outside main phases", () => {
  const { game, player1 } = createStartedGame()

  // Create a sorcery (not an instant)
  const sorcery = {
    instanceId: "test-sorcery",
    definition: {
      id: "test-sorcery",
      name: "Test Sorcery",
      types: ["SORCERY" as const],
    },
    ownerId: player1.id,
  }
  addSpellToHand(game, player1.id, sorcery)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: sorcery.instanceId,
      targets: [],
    })
  }).toThrow(NotMainPhaseError)
})

test("it throws error when non-current player tries to cast spell", () => {
  const { game, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard = createTestSpell(player2.id)
  addSpellToHand(game, player2.id, spellCard)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player2.id,
      cardId: spellCard.instanceId,
      targets: [],
    })
  }).toThrow(InvalidPlayerActionError)
})

test("it throws error when trying to cast a card that is not in hand", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: "non-existent-card",
      targets: [],
    })
  }).toThrow(CardNotFoundInHandError)
})

test("it throws error when trying to cast a non-spell card", () => {
  const { game, player1, land } = createGameInMainPhaseWithLand()

  expect(() => {
    game.apply({
      type: "CAST_SPELL",
      playerId: player1.id,
      cardId: land.instanceId,
      targets: [],
    })
  }).toThrow(CardIsNotSpellError)
})

test("it does not show CAST_SPELL action if player has no spells in hand", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).not.toContain("CAST_SPELL")
})

test("it shows CAST_SPELL action when player has spell in hand during main phase", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spellCard = createTestSpell(player1.id)
  addSpellToHand(game, player1.id, spellCard)

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toContain("CAST_SPELL")
})

test("it allows casting spell in second main phase", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.SECOND_MAIN)

  const spellCard = createTestSpell(player1.id)
  addSpellToHand(game, player1.id, spellCard)

  const actions = game.getAllowedActionsFor(player1.id)
  expect(actions).toContain("CAST_SPELL")

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spellCard.instanceId,
    targets: [],
  })

  const stateAfter = game.getPlayerState(player1.id)
  expect(stateAfter.hand.cards).not.toContainEqual(spellCard)
})

describe("Timing Validation", () => {
  describe("Sorcery-speed timing", () => {
    test("throws NotYourTurnError when casting sorcery on opponent turn", () => {
      const { game, player1, player2 } = createStartedGame()

      // Advance to player1's main phase
      advanceToStep(game, Step.FIRST_MAIN)
      expect(game.currentPlayerId).toBe(player1.id)

      // Player1 casts an instant to give player2 priority
      const instant = createTestSpell(player1.id)
      addSpellToHand(game, player1.id, instant)
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: instant.instanceId,
        targets: [],
      })

      // Now player2 has priority on player1's turn
      // Player2 tries to cast a sorcery
      const sorcery = {
        instanceId: "test-sorcery",
        definition: {
          id: "test-sorcery",
          name: "Test Sorcery",
          types: ["SORCERY" as const],
        },
        ownerId: player2.id,
      }
      addSpellToHand(game, player2.id, sorcery)

      expect(() => {
        game.apply({
          type: "CAST_SPELL",
          playerId: player2.id,
          cardId: sorcery.instanceId,
          targets: [],
        })
      }).toThrow(NotYourTurnError)
    })

    test("throws NotMainPhaseError when casting sorcery outside main phase", () => {
      const { game, player1 } = createStartedGame()

      // Advance to combat phase
      advanceToStep(game, Step.BEGINNING_OF_COMBAT)
      expect(game.currentPlayerId).toBe(player1.id)

      const sorcery = {
        instanceId: "test-sorcery",
        definition: {
          id: "test-sorcery",
          name: "Test Sorcery",
          types: ["SORCERY" as const],
        },
        ownerId: player1.id,
      }
      addSpellToHand(game, player1.id, sorcery)

      expect(() => {
        game.apply({
          type: "CAST_SPELL",
          playerId: player1.id,
          cardId: sorcery.instanceId,
          targets: [],
        })
      }).toThrow(NotMainPhaseError)
    })

    test("throws StackNotEmptyError when casting sorcery with non-empty stack", () => {
      const { game, player1, player2 } = createStartedGame()

      // Advance to main phase
      advanceToStep(game, Step.FIRST_MAIN)

      // Player1 casts an instant (stack is now non-empty)
      const instant = createTestSpell(player1.id)
      addSpellToHand(game, player1.id, instant)
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: instant.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(1)

      // Player2 passes priority back to player1
      game.apply({ type: "PASS_PRIORITY", playerId: player2.id })

      // Now player1 has priority again with non-empty stack
      // Player1 tries to cast a sorcery while stack is not empty
      const sorcery = {
        instanceId: "test-sorcery",
        definition: {
          id: "test-sorcery",
          name: "Test Sorcery",
          types: ["SORCERY" as const],
        },
        ownerId: player1.id,
      }
      addSpellToHand(game, player1.id, sorcery)

      expect(() => {
        game.apply({
          type: "CAST_SPELL",
          playerId: player1.id,
          cardId: sorcery.instanceId,
          targets: [],
        })
      }).toThrow(StackNotEmptyError)
    })

    test("allows casting sorcery during main phase with empty stack on own turn", () => {
      const { game, player1 } = createStartedGame()

      advanceToStep(game, Step.FIRST_MAIN)

      const sorcery = {
        instanceId: "test-sorcery",
        definition: {
          id: "test-sorcery",
          name: "Test Sorcery",
          types: ["SORCERY" as const],
        },
        ownerId: player1.id,
      }
      addSpellToHand(game, player1.id, sorcery)

      // Should not throw
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: sorcery.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(1)
    })

    test("creature without Flash follows sorcery-speed timing", () => {
      const { game, player1, player2 } = createStartedGame()

      // Advance to player1's main phase
      advanceToStep(game, Step.FIRST_MAIN)

      // Player1 casts instant to give player2 priority
      const instant = createTestSpell(player1.id)
      addSpellToHand(game, player1.id, instant)
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: instant.instanceId,
        targets: [],
      })

      // Player2 tries to cast creature without Flash
      const creature = createTestCreature(player2.id)
      const playerState = game.getPlayerState(player2.id)
      playerState.hand.cards.push(creature)

      expect(() => {
        game.apply({
          type: "CAST_SPELL",
          playerId: player2.id,
          cardId: creature.instanceId,
          targets: [],
        })
      }).toThrow(NotYourTurnError)

      // Error message should mention Flash
      try {
        game.apply({
          type: "CAST_SPELL",
          playerId: player2.id,
          cardId: creature.instanceId,
          targets: [],
        })
      } catch (err) {
        expect((err as Error).message).toContain("Flash")
      }
    })
  })

  describe("Instant-speed timing", () => {
    test("allows casting instant on opponent turn", () => {
      const { game, player1, player2 } = createStartedGame()

      // Advance to player1's main phase
      advanceToStep(game, Step.FIRST_MAIN)

      // Player1 casts instant to give player2 priority
      const instant1 = createTestSpell(player1.id, "instant-1")
      addSpellToHand(game, player1.id, instant1)
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: instant1.instanceId,
        targets: [],
      })

      // Player2 casts instant on player1's turn - should work
      const instant2 = createTestSpell(player2.id, "instant-2")
      addSpellToHand(game, player2.id, instant2)

      game.apply({
        type: "CAST_SPELL",
        playerId: player2.id,
        cardId: instant2.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(2)
    })

    test("allows casting instant during combat", () => {
      const { game, player1 } = createStartedGame()

      // Advance to combat
      advanceToStep(game, Step.BEGINNING_OF_COMBAT)

      const instant = createTestSpell(player1.id)
      addSpellToHand(game, player1.id, instant)

      // Should not throw
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: instant.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(1)
    })

    test("allows casting instant with non-empty stack", () => {
      const { game, player1, player2 } = createStartedGame()

      advanceToStep(game, Step.FIRST_MAIN)

      // Cast first instant
      const instant1 = createTestSpell(player1.id, "instant-1")
      addSpellToHand(game, player1.id, instant1)
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: instant1.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(1)

      // Player2 passes priority back to player1
      game.apply({ type: "PASS_PRIORITY", playerId: player2.id })

      // Cast second instant while stack is not empty - should work
      const instant2 = createTestSpell(player1.id, "instant-2")
      addSpellToHand(game, player1.id, instant2)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: instant2.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(2)
    })
  })

  describe("Flash keyword", () => {
    test("allows casting Flash creature on opponent turn", () => {
      const { game, player1, player2 } = createStartedGame()

      // Advance to player1's main phase
      advanceToStep(game, Step.FIRST_MAIN)

      // Player1 casts instant to give player2 priority
      const instant = createTestSpell(player1.id)
      addSpellToHand(game, player1.id, instant)
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: instant.instanceId,
        targets: [],
      })

      // Player2 casts Flash creature on player1's turn - should work
      const flashCreature = createCreatureWithFlash(player2.id)
      const playerState = game.getPlayerState(player2.id)
      playerState.hand.cards.push(flashCreature)

      game.apply({
        type: "CAST_SPELL",
        playerId: player2.id,
        cardId: flashCreature.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(2)
    })

    test("allows casting Flash creature during combat", () => {
      const { game, player1 } = createStartedGame()

      // Advance to combat
      advanceToStep(game, Step.DECLARE_BLOCKERS)

      const flashCreature = createCreatureWithFlash(player1.id)
      const playerState = game.getPlayerState(player1.id)
      playerState.hand.cards.push(flashCreature)

      // Should not throw
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: flashCreature.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(1)
    })

    test("allows casting Flash creature with non-empty stack", () => {
      const { game, player1, player2 } = createStartedGame()

      advanceToStep(game, Step.FIRST_MAIN)

      // Cast instant first
      const instant = createTestSpell(player1.id)
      addSpellToHand(game, player1.id, instant)
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: instant.instanceId,
        targets: [],
      })

      // Player2 passes priority back to player1
      game.apply({ type: "PASS_PRIORITY", playerId: player2.id })

      // Cast Flash creature while stack is not empty - should work
      const flashCreature = createCreatureWithFlash(player1.id)
      const playerState = game.getPlayerState(player1.id)
      playerState.hand.cards.push(flashCreature)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: flashCreature.instanceId,
        targets: [],
      })

      expect(game.getStack()).toHaveLength(2)
    })
  })

  describe("Game state unchanged on timing failure", () => {
    test("card remains in hand when timing validation fails", () => {
      const { game, player2 } = createStartedGame()

      // Advance to player1's main phase
      advanceToStep(game, Step.FIRST_MAIN)

      const sorcery = {
        instanceId: "test-sorcery",
        definition: {
          id: "test-sorcery",
          name: "Test Sorcery",
          types: ["SORCERY" as const],
        },
        ownerId: player2.id,
      }
      addSpellToHand(game, player2.id, sorcery)

      const handBefore = game.getPlayerState(player2.id).hand.cards.length
      const stackBefore = game.getStack().length

      try {
        game.apply({
          type: "CAST_SPELL",
          playerId: player2.id,
          cardId: sorcery.instanceId,
          targets: [],
        })
      } catch {
        // Expected to throw
      }

      const handAfter = game.getPlayerState(player2.id).hand.cards.length
      const stackAfter = game.getStack().length

      expect(handAfter).toBe(handBefore)
      expect(stackAfter).toBe(stackBefore)
    })
  })
})
