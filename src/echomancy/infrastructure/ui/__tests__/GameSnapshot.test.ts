import { describe, expect, it } from "vitest"
import { createGameSnapshot, type CardRegistry } from "../GameSnapshot"
import {
  addCreatureToBattlefield,
  addSpellToHand,
  advanceToStep,
  createGameInMainPhase,
  createStartedGame,
  createTestCreature,
  createTestSpell,
} from "@/echomancy/domainmodel/game/__tests__/helpers"
import { Step } from "@/echomancy/domainmodel/game/Steps"

/**
 * Mock card registry for testing.
 * Maps card definition IDs to human-readable names.
 */
const mockCardRegistry: CardRegistry = {
  getCardName(cardDefinitionId: string): string {
    const nameMap: Record<string, string> = {
      "test-spell": "Test Spell",
      "test-creature": "Test Creature",
      "test-land": "Test Land",
      "flying-creature": "Flying Creature",
      "test-creature-ability": "Creature with Ability",
    }
    return nameMap[cardDefinitionId] ?? cardDefinitionId
  },
}

describe("GameSnapshot", () => {
  describe("Basic Snapshot Creation", () => {
    it("should create a snapshot for a specific viewer", () => {
      const { game, player1, player2 } = createStartedGame()

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.viewerPlayerId).toBe(player1.id)
      expect(snapshot.publicGameState).toBeDefined()
      expect(snapshot.privatePlayerState).toBeDefined()
      expect(snapshot.opponentStates).toHaveLength(1)
      expect(snapshot.opponentStates[0].playerId).toBe(player2.id)
    })

    it("should throw error if viewer is not in the game", () => {
      const { game } = createStartedGame()

      const exported = game.exportState()

      expect(() => createGameSnapshot(exported, "invalid-player", mockCardRegistry)).toThrow(
        "Player invalid-player not found in game state",
      )
    })

    it("should be reconstructible from the same export", () => {
      const { game, player1 } = createStartedGame()

      const exported = game.exportState()
      const snapshot1 = createGameSnapshot(exported, player1.id, mockCardRegistry)
      const snapshot2 = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot1).toEqual(snapshot2)
    })
  })

  describe("Public Game State", () => {
    it("should include public game state visible to all players", () => {
      const { game, player1, player2 } = createStartedGame()

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.publicGameState.turnNumber).toBe(1)
      expect(snapshot.publicGameState.currentPlayerId).toBe(player1.id)
      expect(snapshot.publicGameState.activePlayerId).toBe(player1.id)
      expect(snapshot.publicGameState.priorityPlayerId).toBe(player1.id)
      expect(snapshot.publicGameState.currentStep).toBe(Step.UNTAP)
      expect(snapshot.publicGameState.currentPhase).toBe("Beginning")
      expect(snapshot.publicGameState.stackSize).toBe(0)
    })

    it("should derive correct phase from step", () => {
      const { game, player1 } = createGameInMainPhase()

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.publicGameState.currentPhase).toBe("Precombat Main")
    })

    it("should include combat summary during combat", () => {
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
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.publicGameState.combatSummary).not.toBeNull()
      expect(snapshot.publicGameState.combatSummary?.attackerCount).toBe(1)
      expect(snapshot.publicGameState.combatSummary?.blockerCount).toBe(1)
    })

    it("should have null combat summary outside combat", () => {
      const { game, player1 } = createGameInMainPhase()

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.publicGameState.combatSummary).toBeNull()
    })
  })

  describe("Private Player State (Viewer)", () => {
    it("should include full visibility of viewer's zones", () => {
      const { game, player1 } = createGameInMainPhase()

      const spell = createTestSpell(player1.id, "spell-1")
      const creature = createTestCreature(player1.id, "creature-1", 2, 2)

      addSpellToHand(game, player1.id, spell)
      addCreatureToBattlefield(game, player1.id, creature)

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      // Hand should be fully visible
      expect(snapshot.privatePlayerState.hand.length).toBeGreaterThan(0)
      const spellInHand = snapshot.privatePlayerState.hand.find(
        (c) => c.instanceId === spell.instanceId,
      )
      expect(spellInHand).toBeDefined()
      expect(spellInHand?.name).toBe("Test Spell")

      // Battlefield should be visible
      expect(snapshot.privatePlayerState.battlefield).toHaveLength(1)
      expect(snapshot.privatePlayerState.battlefield[0].instanceId).toBe(creature.instanceId)
    })

    it("should include viewer's life total and mana pool", () => {
      const { game, player1 } = createGameInMainPhase()

      player1.adjustLifeTotal(-5)
      game.addMana(player1.id, "W", 2)
      game.addMana(player1.id, "U", 1)

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.privatePlayerState.lifeTotal).toBe(15)
      expect(snapshot.privatePlayerState.manaPool).toEqual({
        W: 2,
        U: 1,
        B: 0,
        R: 0,
        G: 0,
        C: 0,
      })
    })

    it("should include viewer's graveyard", () => {
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

      // Resolve stack
      game.apply({ type: "PASS_PRIORITY", playerId: player2.id })
      game.apply({ type: "PASS_PRIORITY", playerId: player1.id })

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.privatePlayerState.graveyard).toHaveLength(1)
      expect(snapshot.privatePlayerState.graveyard[0].instanceId).toBe(spell.instanceId)
    })
  })

  describe("Opponent State (Hidden Information)", () => {
    it("should hide opponent's hand but show hand size", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const spell = createTestSpell(player2.id, "opponent-spell")
      addSpellToHand(game, player2.id, spell)

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      const opponentState = snapshot.opponentStates.find((o) => o.playerId === player2.id)

      expect(opponentState).toBeDefined()
      // Hand size is visible
      expect(opponentState?.handSize).toBeGreaterThan(0)
      // But hand cards are NOT visible in opponent state
      expect(opponentState).not.toHaveProperty("hand")
    })

    it("should show opponent's battlefield", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const creature = createTestCreature(player2.id, "opponent-creature", 3, 3)
      addCreatureToBattlefield(game, player2.id, creature)

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      const opponentState = snapshot.opponentStates.find((o) => o.playerId === player2.id)

      expect(opponentState?.battlefield).toHaveLength(1)
      expect(opponentState?.battlefield[0].instanceId).toBe(creature.instanceId)
    })

    it("should show opponent's graveyard", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const spell = createTestSpell(player2.id, "opponent-spell")
      addSpellToHand(game, player2.id, spell)

      // Player 2 casts spell
      game.apply({
        type: "CAST_SPELL",
        playerId: player2.id,
        cardId: spell.instanceId,
        targets: [],
      })

      // Resolve stack
      game.apply({ type: "PASS_PRIORITY", playerId: player1.id })
      game.apply({ type: "PASS_PRIORITY", playerId: player2.id })

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      const opponentState = snapshot.opponentStates.find((o) => o.playerId === player2.id)

      expect(opponentState?.graveyard).toHaveLength(1)
      expect(opponentState?.graveyard[0].instanceId).toBe(spell.instanceId)
    })

    it("should show opponent's life total", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      player2.adjustLifeTotal(-7)

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      const opponentState = snapshot.opponentStates.find((o) => o.playerId === player2.id)

      expect(opponentState?.lifeTotal).toBe(13)
    })
  })

  describe("Card Snapshots", () => {
    it("should resolve card names from card registry", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      const cardSnapshot = snapshot.privatePlayerState.battlefield[0]
      expect(cardSnapshot.name).toBe("Test Creature")
    })

    it("should include creature state in card snapshot", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1", 2, 3)
      addCreatureToBattlefield(game, player1.id, creature)

      game.tapPermanent(creature.instanceId)

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      const cardSnapshot = snapshot.privatePlayerState.battlefield[0]

      expect(cardSnapshot.tapped).toBe(true)
      expect(cardSnapshot.power).toBe(2)
      expect(cardSnapshot.toughness).toBe(3)
      expect(cardSnapshot.combatState).not.toBeNull()
      expect(cardSnapshot.combatState?.isAttacking).toBe(false)
      expect(cardSnapshot.combatState?.isBlocking).toBe(false)
    })

    it("should include counters in card snapshot", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)

      game.addCounters(creature.instanceId, "PLUS_ONE_PLUS_ONE", 3)

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      const cardSnapshot = snapshot.privatePlayerState.battlefield[0]

      expect(cardSnapshot.counters?.PLUS_ONE_PLUS_ONE).toBe(3)
      expect(cardSnapshot.power).toBe(5) // 2 base + 3 counters
      expect(cardSnapshot.toughness).toBe(5) // 2 base + 3 counters
    })

    it("should include combat state for attacking creatures", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)

      advanceToStep(game, Step.DECLARE_ATTACKERS)
      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: creature.instanceId,
      })

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      const cardSnapshot = snapshot.privatePlayerState.battlefield[0]

      expect(cardSnapshot.combatState?.isAttacking).toBe(true)
      expect(cardSnapshot.combatState?.isBlocking).toBe(false)
      expect(cardSnapshot.combatState?.blockedBy).toEqual([])
    })

    it("should include combat state for blocking creatures", () => {
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
      const snapshot = createGameSnapshot(exported, player2.id, mockCardRegistry)

      const blockerSnapshot = snapshot.privatePlayerState.battlefield[0]

      expect(blockerSnapshot.combatState?.isBlocking).toBe(true)
      expect(blockerSnapshot.combatState?.blocking).toBe(attacker.instanceId)
    })

    it("should include static keywords", () => {
      const { game, player1 } = createGameInMainPhase()

      const flyingCreature = {
        instanceId: "flying-1",
        ownerId: player1.id,
        definition: {
          id: "flying-creature",
          name: "Flying Bird",
          types: ["CREATURE"] as const,
          power: 2,
          toughness: 2,
          staticAbilities: ["FLYING"] as const,
        },
      }

      game.enterBattlefield(flyingCreature, player1.id)

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      const cardSnapshot = snapshot.privatePlayerState.battlefield[0]

      expect(cardSnapshot.staticKeywords).toContain("FLYING")
    })
  })

  describe("Stack Snapshot", () => {
    it("should create empty stack snapshot", () => {
      const { game, player1 } = createStartedGame()

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.visibleStack.items).toEqual([])
    })

    it("should create stack snapshot with resolved card names", () => {
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
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.visibleStack.items).toHaveLength(1)
      expect(snapshot.visibleStack.items[0].sourceCardName).toBe("Test Spell")
      expect(snapshot.visibleStack.items[0].kind).toBe("SPELL")
      expect(snapshot.visibleStack.items[0].controllerId).toBe(player1.id)
    })

    it("should resolve target descriptions for spells with targets", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const targetCreature = createTestCreature(player2.id, "target", 2, 2)
      addCreatureToBattlefield(game, player2.id, targetCreature)

      const spell = {
        instanceId: "targeted-spell",
        ownerId: player1.id,
        definition: {
          id: "test-spell",
          name: "Target Spell",
          types: ["INSTANT"] as const,
          effect: {
            resolve: () => {
              /* test */
            },
          },
        },
      }

      addSpellToHand(game, player1.id, spell)

      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: spell.instanceId,
        targets: [targetCreature.instanceId],
      })

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.visibleStack.items[0].targetDescriptions).toHaveLength(1)
      expect(snapshot.visibleStack.items[0].targetDescriptions[0]).toBe("Test Creature")
    })
  })

  describe("UI Hints", () => {
    it("should indicate if viewer can pass priority", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const exported1 = game.exportState()
      const snapshot1 = createGameSnapshot(exported1, player1.id, mockCardRegistry)

      expect(snapshot1.uiHints?.canPassPriority).toBe(true)

      const exported2 = game.exportState()
      const snapshot2 = createGameSnapshot(exported2, player2.id, mockCardRegistry)

      expect(snapshot2.uiHints?.canPassPriority).toBe(false)
    })

    it("should indicate if viewer can play land", () => {
      const { game, player1 } = createGameInMainPhase()

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      // In main phase, no lands played yet
      expect(snapshot.uiHints?.canPlayLand).toBe(true)
    })

    it("should indicate can't play land after one land played", () => {
      const { game, player1, dummyLandInstanceId } = createGameInMainPhase()

      game.apply({
        type: "PLAY_LAND",
        playerId: player1.id,
        cardId: dummyLandInstanceId,
      })

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.uiHints?.canPlayLand).toBe(false)
    })

    it("should highlight attacking creatures", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)

      advanceToStep(game, Step.DECLARE_ATTACKERS)
      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: creature.instanceId,
      })

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      expect(snapshot.uiHints?.highlightedAttackers).toContain(creature.instanceId)
    })

    it("should highlight blocking creatures", () => {
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
      const snapshot = createGameSnapshot(exported, player2.id, mockCardRegistry)

      expect(snapshot.uiHints?.highlightedBlockers).toContain(blocker.instanceId)
    })
  })

  describe("Player Perspective Symmetry", () => {
    it("should create different snapshots for different viewers", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const p1Spell = createTestSpell(player1.id, "p1-spell")
      const p2Spell = createTestSpell(player2.id, "p2-spell")

      addSpellToHand(game, player1.id, p1Spell)
      addSpellToHand(game, player2.id, p2Spell)

      const exported = game.exportState()
      const snapshot1 = createGameSnapshot(exported, player1.id, mockCardRegistry)
      const snapshot2 = createGameSnapshot(exported, player2.id, mockCardRegistry)

      // Player 1 sees their own hand
      expect(snapshot1.privatePlayerState.hand.some((c) => c.instanceId === p1Spell.instanceId)).toBe(
        true,
      )
      // Player 1 sees player 2 as opponent
      expect(snapshot1.opponentStates[0].playerId).toBe(player2.id)
      // Player 1 does NOT see player 2's hand cards
      expect(snapshot1.opponentStates[0]).not.toHaveProperty("hand")

      // Player 2 sees their own hand
      expect(snapshot2.privatePlayerState.hand.some((c) => c.instanceId === p2Spell.instanceId)).toBe(
        true,
      )
      // Player 2 sees player 1 as opponent
      expect(snapshot2.opponentStates[0].playerId).toBe(player1.id)
    })

    it("should show same public state to all viewers", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const exported = game.exportState()
      const snapshot1 = createGameSnapshot(exported, player1.id, mockCardRegistry)
      const snapshot2 = createGameSnapshot(exported, player2.id, mockCardRegistry)

      expect(snapshot1.publicGameState).toEqual(snapshot2.publicGameState)
    })
  })

  describe("Invariants", () => {
    it("should be immutable after creation", () => {
      const { game, player1 } = createStartedGame()

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      const originalTurnNumber = snapshot.publicGameState.turnNumber

      // Attempting to mutate should not affect the snapshot
      // TypeScript types should prevent this, but verify at runtime
      expect(() => {
        // @ts-expect-error - Testing immutability
        snapshot.publicGameState.turnNumber = 999
      }).toThrow()

      expect(snapshot.publicGameState.turnNumber).toBe(originalTurnNumber)
    })

    it("should contain no engine references", () => {
      const { game, player1 } = createStartedGame()

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      // Snapshot should be plain data only
      expect(snapshot).not.toHaveProperty("game")
      expect(snapshot).not.toHaveProperty("apply")
      expect(snapshot).not.toHaveProperty("getPlayerState")
    })

    it("should apply visibility rules correctly", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      const p1Secret = createTestSpell(player1.id, "secret-1")
      const p2Secret = createTestSpell(player2.id, "secret-2")

      addSpellToHand(game, player1.id, p1Secret)
      addSpellToHand(game, player2.id, p2Secret)

      const exported = game.exportState()
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      // Viewer sees their own hidden information
      const hasP1Secret = snapshot.privatePlayerState.hand.some(
        (c) => c.instanceId === p1Secret.instanceId,
      )
      expect(hasP1Secret).toBe(true)

      // Viewer does NOT see opponent's hidden information (hand cards)
      // Opponent state should only have handSize
      const opponentState = snapshot.opponentStates.find((o) => o.playerId === player2.id)
      expect(opponentState?.handSize).toBeGreaterThan(0)
      // There should be no 'hand' property on opponent state
      expect(opponentState).not.toHaveProperty("hand")
    })
  })

  describe("Complex Game State Snapshot", () => {
    it("should create complete snapshot during active combat", () => {
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
      const snapshot = createGameSnapshot(exported, player1.id, mockCardRegistry)

      // Verify public state
      expect(snapshot.publicGameState.currentPhase).toBe("Combat")
      expect(snapshot.publicGameState.currentStep).toBe(Step.DECLARE_BLOCKERS)
      expect(snapshot.publicGameState.combatSummary?.attackerCount).toBe(2)
      expect(snapshot.publicGameState.combatSummary?.blockerCount).toBe(1)

      // Verify viewer's creatures
      const viewerCreatures = snapshot.privatePlayerState.battlefield
      expect(viewerCreatures).toHaveLength(2)

      const attacker1Snapshot = viewerCreatures.find((c) => c.instanceId === attacker1.instanceId)
      const attacker2Snapshot = viewerCreatures.find((c) => c.instanceId === attacker2.instanceId)

      expect(attacker1Snapshot?.combatState?.isAttacking).toBe(true)
      expect(attacker1Snapshot?.combatState?.blockedBy).toContain(blocker.instanceId)

      expect(attacker2Snapshot?.combatState?.isAttacking).toBe(true)
      expect(attacker2Snapshot?.combatState?.blockedBy).toEqual([])

      // Verify opponent's creatures
      const opponentCreatures = snapshot.opponentStates[0].battlefield
      expect(opponentCreatures).toHaveLength(1)

      const blockerSnapshot = opponentCreatures[0]
      expect(blockerSnapshot.combatState?.isBlocking).toBe(true)
      expect(blockerSnapshot.combatState?.blocking).toBe(attacker1.instanceId)
    })
  })
})
