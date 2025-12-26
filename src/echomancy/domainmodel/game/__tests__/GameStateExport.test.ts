import { describe, expect, it } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import { Step } from "../Steps"
import {
  addCreatureToBattlefield,
  addSpellToHand,
  advanceToStep,
  createGameInMainPhase,
  createStartedGame,
  createTestCreature,
  createTestSpell,
  resolveStack,
} from "./helpers"

describe("GameStateExport", () => {
  describe("Basic Export Structure", () => {
    it("should export complete game state at game start", () => {
      const { game, player1, player2 } = createStartedGame()

      const exported = game.exportState()

      // Verify global game state
      expect(exported.gameId).toBe(game.id)
      expect(exported.currentTurnNumber).toBe(1)
      expect(exported.currentPlayerId).toBe(player1.id)
      expect(exported.currentStep).toBe(Step.UNTAP)
      expect(exported.priorityPlayerId).toBe(player1.id)
      expect(exported.turnOrder).toEqual([player1.id, player2.id])
    })

    it("should export deterministically (same game state produces same export)", () => {
      const { game } = createStartedGame()

      const export1 = game.exportState()
      const export2 = game.exportState()

      expect(export1).toEqual(export2)
    })

    it("should not mutate game state during export", () => {
      const { game, player1 } = createStartedGame()

      const beforeExport = {
        currentPlayerId: game.currentPlayerId,
        currentStep: game.currentStep,
        stackSize: game.getStack().length,
        handSize: game.getPlayerState(player1.id).hand.cards.length,
      }

      game.exportState()

      const afterExport = {
        currentPlayerId: game.currentPlayerId,
        currentStep: game.currentStep,
        stackSize: game.getStack().length,
        handSize: game.getPlayerState(player1.id).hand.cards.length,
      }

      expect(afterExport).toEqual(beforeExport)
    })
  })

  describe("Turn Number Tracking", () => {
    it("should start with turn number 1", () => {
      const { game } = createStartedGame()

      const exported = game.exportState()

      expect(exported.currentTurnNumber).toBe(1)
    })

    it("should increment turn number when returning to first player", () => {
      const { game, player1, player2 } = createStartedGame()

      // Advance through player1's turn
      game.apply({ type: "END_TURN", playerId: player1.id })
      expect(game.exportState().currentTurnNumber).toBe(1)
      expect(game.exportState().currentPlayerId).toBe(player2.id)

      // Advance through player2's turn - should increment turn number
      game.apply({ type: "END_TURN", playerId: player2.id })
      const exported = game.exportState()

      expect(exported.currentTurnNumber).toBe(2)
      expect(exported.currentPlayerId).toBe(player1.id)
    })

    it("should continue incrementing turn number across multiple rounds", () => {
      const { game, player1, player2 } = createStartedGame()

      // Complete 3 full rounds (6 player turns)
      for (let round = 0; round < 3; round++) {
        game.apply({ type: "END_TURN", playerId: player1.id })
        game.apply({ type: "END_TURN", playerId: player2.id })
      }

      const exported = game.exportState()
      expect(exported.currentTurnNumber).toBe(4) // Started at 1, added 3
    })
  })

  describe("Player State Export", () => {
    it("should include all player data with life totals", () => {
      const { game, player1, player2 } = createStartedGame()

      const exported = game.exportState()

      expect(exported.players[player1.id]).toBeDefined()
      expect(exported.players[player1.id].lifeTotal).toBe(20)
      expect(exported.players[player2.id]).toBeDefined()
      expect(exported.players[player2.id].lifeTotal).toBe(20)
    })

    it("should reflect life total changes", () => {
      const { game, player1 } = createStartedGame()

      player1.adjustLifeTotal(-5)
      const exported = game.exportState()

      expect(exported.players[player1.id].lifeTotal).toBe(15)
    })

    it("should export mana pool for each player", () => {
      const { game, player1 } = createGameInMainPhase()

      game.addMana(player1.id, "W", 2)
      game.addMana(player1.id, "U", 1)
      game.addMana(player1.id, "R", 3)

      const exported = game.exportState()

      expect(exported.players[player1.id].manaPool).toEqual({
        W: 2,
        U: 1,
        B: 0,
        R: 3,
        G: 0,
        C: 0,
      })
    })

    it("should track played lands this turn for current player only", () => {
      const { game, player1, player2, dummyLandInstanceId } =
        createGameInMainPhase()

      // Player1 plays a land
      game.apply({
        type: "PLAY_LAND",
        playerId: player1.id,
        cardId: dummyLandInstanceId,
      })

      const exported = game.exportState()

      expect(exported.players[player1.id].playedLandsThisTurn).toBe(1)
      expect(exported.players[player2.id].playedLandsThisTurn).toBe(0)
    })

    it("should reset played lands when turn changes", () => {
      const { game, player1, player2, dummyLandInstanceId } =
        createGameInMainPhase()

      // Player1 plays a land
      game.apply({
        type: "PLAY_LAND",
        playerId: player1.id,
        cardId: dummyLandInstanceId,
      })

      // End turn
      game.apply({ type: "END_TURN", playerId: player1.id })

      const exported = game.exportState()

      // Player1 is no longer current player, so playedLandsThisTurn should be 0
      expect(exported.players[player1.id].playedLandsThisTurn).toBe(0)
      expect(exported.players[player2.id].playedLandsThisTurn).toBe(0)
    })
  })

  describe("Zone Export - Complete and Unfiltered", () => {
    it("should export hand with all cards (including hidden information)", () => {
      const { game, player1 } = createStartedGame()

      const spell = createTestSpell(player1.id, "spell-1")
      addSpellToHand(game, player1.id, spell)

      const exported = game.exportState()

      // Hand should contain dummy land + added spell
      expect(exported.players[player1.id].zones.hand.cards).toHaveLength(2)
      expect(
        exported.players[player1.id].zones.hand.cards.some(
          (c) => c.instanceId === spell.instanceId,
        ),
      ).toBe(true)
    })

    it("should export battlefield with all permanents", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature1 = createTestCreature(player1.id, "creature-1", 2, 2)
      const creature2 = createTestCreature(player1.id, "creature-2", 3, 3)

      addCreatureToBattlefield(game, player1.id, creature1)
      addCreatureToBattlefield(game, player1.id, creature2)

      const exported = game.exportState()

      expect(exported.players[player1.id].zones.battlefield.cards).toHaveLength(
        2,
      )
    })

    it("should export graveyard with all cards", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const spell = createTestSpell(player1.id, "spell-1")
      addSpellToHand(game, player1.id, spell)

      // Cast and resolve spell (goes to graveyard)
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: spell.instanceId,
        targets: [],
      })
      resolveStack(game, player2.id, player1.id)

      const exported = game.exportState()

      expect(exported.players[player1.id].zones.graveyard.cards).toHaveLength(1)
      expect(
        exported.players[player1.id].zones.graveyard.cards[0].instanceId,
      ).toBe(spell.instanceId)
    })
  })

  describe("Card Instance Export", () => {
    it("should export card instance with all required fields", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1", 2, 3)
      addCreatureToBattlefield(game, player1.id, creature)

      const exported = game.exportState()
      const exportedCreature =
        exported.players[player1.id].zones.battlefield.cards[0]

      expect(exportedCreature.instanceId).toBe(creature.instanceId)
      expect(exportedCreature.ownerId).toBe(player1.id)
      expect(exportedCreature.controllerId).toBe(player1.id)
      expect(exportedCreature.cardDefinitionId).toBe(creature.definition.id)
      expect(exportedCreature.types).toEqual(creature.definition.types)
      expect(exportedCreature.power).toBe(2)
      expect(exportedCreature.toughness).toBe(3)
    })

    it("should export static abilities when present", () => {
      const { game, player1 } = createGameInMainPhase()

      const flyingCreature: CardInstance = {
        instanceId: "flyer-1",
        ownerId: player1.id,
        definition: {
          id: "flying-creature",
          name: "Flying Creature",
          types: ["CREATURE"],
          power: 2,
          toughness: 2,
          staticAbilities: ["FLYING"],
        },
      }

      game.enterBattlefield(flyingCreature, player1.id)

      const exported = game.exportState()
      const exportedCreature =
        exported.players[player1.id].zones.battlefield.cards[0]

      expect(exportedCreature.staticAbilities).toEqual(["FLYING"])
    })

    it("should not include staticAbilities field when none present", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)

      const exported = game.exportState()
      const exportedCreature =
        exported.players[player1.id].zones.battlefield.cards[0]

      expect(exportedCreature.staticAbilities).toBeUndefined()
    })
  })

  describe("Creature State Export", () => {
    it("should export creature state for creatures on battlefield", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1", 2, 3)
      addCreatureToBattlefield(game, player1.id, creature)

      const exported = game.exportState()
      const exportedCreature =
        exported.players[player1.id].zones.battlefield.cards[0]

      expect(exportedCreature.creatureState).toBeDefined()
      expect(exportedCreature.creatureState?.isTapped).toBe(false)
      expect(exportedCreature.creatureState?.isAttacking).toBe(false)
      expect(exportedCreature.creatureState?.hasAttackedThisTurn).toBe(false)
      expect(exportedCreature.creatureState?.power).toBe(2)
      expect(exportedCreature.creatureState?.toughness).toBe(3)
      expect(exportedCreature.creatureState?.damageMarkedThisTurn).toBe(0)
      expect(exportedCreature.creatureState?.blockingCreatureId).toBeNull()
      expect(exportedCreature.creatureState?.blockedBy).toBeNull()
    })

    it("should export tapped state correctly", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)

      // Tap the creature
      game.tapPermanent(creature.instanceId)

      const exported = game.exportState()
      const exportedCreature =
        exported.players[player1.id].zones.battlefield.cards[0]

      expect(exportedCreature.creatureState?.isTapped).toBe(true)
    })

    it("should export attacking state correctly", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)

      // Advance to combat
      advanceToStep(game, Step.DECLARE_ATTACKERS)

      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: creature.instanceId,
      })

      const exported = game.exportState()
      const exportedCreature =
        exported.players[player1.id].zones.battlefield.cards[0]

      expect(exportedCreature.creatureState?.isAttacking).toBe(true)
      expect(exportedCreature.creatureState?.hasAttackedThisTurn).toBe(true)
    })

    it("should export +1/+1 counters and modified power/toughness", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1", 2, 3)
      addCreatureToBattlefield(game, player1.id, creature)

      game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 2)

      const exported = game.exportState()
      const exportedCreature =
        exported.players[player1.id].zones.battlefield.cards[0]

      expect(exportedCreature.creatureState?.counters.PLUS_ONE_PLUS_ONE).toBe(2)
      expect(exportedCreature.creatureState?.power).toBe(4) // 2 base + 2 counters
      expect(exportedCreature.creatureState?.toughness).toBe(5) // 3 base + 2 counters
    })

    it("should export blocking relationships", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const attacker = createTestCreature(player1.id, "attacker", 2, 2)
      const blocker = createTestCreature(player2.id, "blocker", 1, 1)

      addCreatureToBattlefield(game, player1.id, attacker)
      addCreatureToBattlefield(game, player2.id, blocker)

      advanceToStep(game, Step.DECLARE_ATTACKERS)
      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: attacker.instanceId,
      })

      advanceToStep(game, Step.DECLARE_BLOCKERS)
      game.apply({
        type: "DECLARE_BLOCKER",
        playerId: player2.id,
        blockerId: blocker.instanceId,
        attackerId: attacker.instanceId,
      })

      const exported = game.exportState()

      const exportedAttacker =
        exported.players[player1.id].zones.battlefield.cards[0]
      const exportedBlocker =
        exported.players[player2.id].zones.battlefield.cards[0]

      expect(exportedAttacker.creatureState?.blockedBy).toBe(blocker.instanceId)
      expect(exportedBlocker.creatureState?.blockingCreatureId).toBe(
        attacker.instanceId,
      )
    })
  })

  describe("Stack Export", () => {
    it("should export empty stack", () => {
      const { game } = createStartedGame()

      const exported = game.exportState()

      expect(exported.stack).toEqual([])
    })

    it("should export spell on stack with correct information", () => {
      const { game, player1 } = createGameInMainPhase()

      const spell = createTestSpell(player1.id, "spell-1")
      addSpellToHand(game, player1.id, spell)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: spell.instanceId,
        targets: [],
      })

      const exported = game.exportState()

      expect(exported.stack).toHaveLength(1)
      expect(exported.stack[0].kind).toBe("SPELL")
      expect(exported.stack[0].sourceCardInstanceId).toBe(spell.instanceId)
      expect(exported.stack[0].controllerId).toBe(player1.id)
      expect(exported.stack[0].targets).toEqual([])
    })

    it("should export activated ability on stack", () => {
      const { game, player1 } = createGameInMainPhase()

      const creatureWithAbility: CardInstance = {
        instanceId: "creature-with-ability",
        ownerId: player1.id,
        definition: {
          id: "test-creature-ability",
          name: "Test Creature with Ability",
          types: ["CREATURE"],
          power: 1,
          toughness: 1,
          activatedAbility: {
            cost: { type: "TAP" },
            effect: {
              resolve: () => {
                /* test ability */
              },
            },
          },
        },
      }

      game.enterBattlefield(creatureWithAbility, player1.id)

      game.apply({
        type: "ACTIVATE_ABILITY",
        playerId: player1.id,
        permanentId: creatureWithAbility.instanceId,
      })

      const exported = game.exportState()

      expect(exported.stack).toHaveLength(1)
      expect(exported.stack[0].kind).toBe("ACTIVATED_ABILITY")
      expect(exported.stack[0].sourceCardInstanceId).toBe(
        creatureWithAbility.instanceId,
      )
      expect(exported.stack[0].controllerId).toBe(player1.id)
    })

    it("should export multiple items on stack in correct order", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const spell1 = createTestSpell(player1.id, "spell-1")
      const spell2 = createTestSpell(player2.id, "spell-2")

      addSpellToHand(game, player1.id, spell1)
      addSpellToHand(game, player2.id, spell2)

      // Player 1 casts spell1
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: spell1.instanceId,
        targets: [],
      })

      // Priority passes to player2, who casts spell2
      game.apply({
        type: "CAST_SPELL",
        playerId: player2.id,
        cardId: spell2.instanceId,
        targets: [],
      })

      const exported = game.exportState()

      expect(exported.stack).toHaveLength(2)
      // Stack is LIFO, so spell2 should be on top (last in array)
      expect(exported.stack[0].sourceCardInstanceId).toBe(spell1.instanceId)
      expect(exported.stack[1].sourceCardInstanceId).toBe(spell2.instanceId)
    })
  })

  describe("Scheduled Steps Export", () => {
    it("should export empty scheduled steps by default", () => {
      const { game } = createStartedGame()

      const exported = game.exportState()

      expect(exported.scheduledSteps).toEqual([])
      expect(exported.resumeStepAfterScheduled).toBeUndefined()
    })

    it("should export scheduled steps when present", () => {
      const { game } = createStartedGame()

      // Schedule extra combat phase
      game.addScheduledSteps([
        Step.BEGINNING_OF_COMBAT,
        Step.DECLARE_ATTACKERS,
        Step.DECLARE_BLOCKERS,
        Step.COMBAT_DAMAGE,
        Step.END_OF_COMBAT,
      ])

      const exported = game.exportState()

      expect(exported.scheduledSteps).toHaveLength(5)
      expect(exported.scheduledSteps[0]).toBe(Step.BEGINNING_OF_COMBAT)
    })
  })

  describe("Planeswalker Export", () => {
    it("should export planeswalker with placeholder state", () => {
      const { game, player1 } = createGameInMainPhase()

      const planeswalker: CardInstance = {
        instanceId: "planeswalker-1",
        ownerId: player1.id,
        definition: {
          id: "test-planeswalker",
          name: "Test Planeswalker",
          types: ["PLANESWALKER"],
        },
      }

      game.enterBattlefield(planeswalker, player1.id)

      const exported = game.exportState()
      const exportedPlaneswalker =
        exported.players[player1.id].zones.battlefield.cards[0]

      expect(exportedPlaneswalker.types).toContain("PLANESWALKER")
      expect(exportedPlaneswalker.planeswalkerState).toEqual({})
    })
  })

  describe("Invariants", () => {
    it("should ensure every card instance is unique", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature1 = createTestCreature(player1.id, "creature-1", 2, 2)
      const creature2 = createTestCreature(player1.id, "creature-2", 3, 3)

      addCreatureToBattlefield(game, player1.id, creature1)
      addCreatureToBattlefield(game, player1.id, creature2)

      const exported = game.exportState()

      // Collect all instance IDs from all zones
      const allInstanceIds: string[] = []
      for (const playerState of Object.values(exported.players)) {
        allInstanceIds.push(
          ...playerState.zones.hand.cards.map((c) => c.instanceId),
        )
        allInstanceIds.push(
          ...playerState.zones.battlefield.cards.map((c) => c.instanceId),
        )
        allInstanceIds.push(
          ...playerState.zones.graveyard.cards.map((c) => c.instanceId),
        )
      }

      // Check for duplicates
      const uniqueIds = new Set(allInstanceIds)
      expect(uniqueIds.size).toBe(allInstanceIds.length)
    })

    it("should not include any UI-specific fields", () => {
      const { game } = createGameInMainPhase()

      const exported = game.exportState()

      // Check that export doesn't contain UI fields
      const exportKeys = Object.keys(exported)
      expect(exportKeys).not.toContain("allowedActions")
      expect(exportKeys).not.toContain("isVisible")
      expect(exportKeys).not.toContain("uiState")
    })
  })

  describe("Complex Game State Export", () => {
    it("should export complete state during active combat", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const attacker1 = createTestCreature(player1.id, "attacker-1", 2, 2)
      const attacker2 = createTestCreature(player1.id, "attacker-2", 3, 3)
      const blocker = createTestCreature(player2.id, "blocker", 2, 2)

      addCreatureToBattlefield(game, player1.id, attacker1)
      addCreatureToBattlefield(game, player1.id, attacker2)
      addCreatureToBattlefield(game, player2.id, blocker)

      advanceToStep(game, Step.DECLARE_ATTACKERS)

      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: attacker1.instanceId,
      })
      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: attacker2.instanceId,
      })

      advanceToStep(game, Step.DECLARE_BLOCKERS)

      game.apply({
        type: "DECLARE_BLOCKER",
        playerId: player2.id,
        blockerId: blocker.instanceId,
        attackerId: attacker1.instanceId,
      })

      const exported = game.exportState()

      // Verify global state
      expect(exported.currentStep).toBe(Step.DECLARE_BLOCKERS)
      expect(exported.currentPlayerId).toBe(player1.id)

      // Verify creature states
      const p1Creatures = exported.players[player1.id].zones.battlefield.cards
      const p2Creatures = exported.players[player2.id].zones.battlefield.cards

      expect(p1Creatures[0].creatureState?.isAttacking).toBe(true)
      expect(p1Creatures[1].creatureState?.isAttacking).toBe(true)
      expect(p1Creatures[0].creatureState?.blockedBy).toBe(blocker.instanceId)
      expect(p1Creatures[1].creatureState?.blockedBy).toBeNull()

      expect(p2Creatures[0].creatureState?.blockingCreatureId).toBe(
        attacker1.instanceId,
      )
    })
  })
})
