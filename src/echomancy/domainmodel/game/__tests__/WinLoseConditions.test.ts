import { describe, expect, test } from "vitest"
import { GameLifecycleState } from "../Game"
import {
  addCreatureToBattlefield,
  createStartedGame,
  createTestCreature,
} from "./helpers"

describe("Win/Lose Conditions - State-Based Actions", () => {
  describe("Life Total Loss", () => {
    test("player with life <= 0 loses immediately during SBA check", () => {
      const { game, player1, player2 } = createStartedGame()

      // Set player1's life to 0
      player1.adjustLifeTotal(-20)
      expect(player1.lifeTotal).toBe(0)

      // Trigger state-based actions
      game.performStateBasedActions()

      // Game should be finished
      expect(game.getLifecycleState()).toBe(GameLifecycleState.FINISHED)

      // Player2 should be the winner
      const outcome = game.getOutcome()
      expect(outcome).not.toBeNull()
      expect(outcome?.type).toBe("WIN")
      if (outcome?.type === "WIN") {
        expect(outcome.winnerId).toBe(player2.id)
        expect(outcome.reason).toBe("LIFE_TOTAL")
      }
    })

    test("player with negative life loses immediately", () => {
      const { game, player1, player2 } = createStartedGame()

      // Set player1's life to negative
      player1.adjustLifeTotal(-25)
      expect(player1.lifeTotal).toBe(-5)

      // Trigger state-based actions
      game.performStateBasedActions()

      // Game should be finished with player2 as winner
      expect(game.getLifecycleState()).toBe(GameLifecycleState.FINISHED)
      const outcome = game.getOutcome()
      expect(outcome?.type).toBe("WIN")
      if (outcome?.type === "WIN") {
        expect(outcome.winnerId).toBe(player2.id)
        expect(outcome.reason).toBe("LIFE_TOTAL")
      }
    })

    test("player loses during opponent's turn", () => {
      const { game, player1, player2 } = createStartedGame()

      // Player2 is active player, player1 loses
      player1.adjustLifeTotal(-20)

      game.performStateBasedActions()

      // Player2 should win
      expect(game.getLifecycleState()).toBe(GameLifecycleState.FINISHED)
      const outcome = game.getOutcome()
      expect(outcome?.type).toBe("WIN")
      if (outcome?.type === "WIN") {
        expect(outcome.winnerId).toBe(player2.id)
      }
    })

    test("player at exactly 0 life loses", () => {
      const { game, player1, player2 } = createStartedGame()

      player1.adjustLifeTotal(-20)
      expect(player1.lifeTotal).toBe(0)

      game.performStateBasedActions()

      expect(game.getLifecycleState()).toBe(GameLifecycleState.FINISHED)
      const outcome = game.getOutcome()
      expect(outcome?.type).toBe("WIN")
      if (outcome?.type === "WIN") {
        expect(outcome.winnerId).toBe(player2.id)
      }
    })

    test("player at 1 life does not lose", () => {
      const { game, player1 } = createStartedGame()

      player1.adjustLifeTotal(-19)
      expect(player1.lifeTotal).toBe(1)

      game.performStateBasedActions()

      // Game should still be active
      expect(game.getLifecycleState()).toBe(GameLifecycleState.STARTED)
      expect(game.getOutcome()).toBeNull()
    })
  })

  describe("Empty Library Loss", () => {
    test("player who attempted draw from empty library loses during SBA check", () => {
      const { game, player1, player2 } = createStartedGame()

      // Attempt to draw from empty library
      game.drawCards(player1.id, 1)
      expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(true)

      // Trigger state-based actions
      game.performStateBasedActions()

      // Game should be finished
      expect(game.getLifecycleState()).toBe(GameLifecycleState.FINISHED)

      // Player2 should be the winner
      const outcome = game.getOutcome()
      expect(outcome).not.toBeNull()
      expect(outcome?.type).toBe("WIN")
      if (outcome?.type === "WIN") {
        expect(outcome.winnerId).toBe(player2.id)
        expect(outcome.reason).toBe("EMPTY_LIBRARY")
      }
    })

    test("empty library loss flag is cleared after game ends", () => {
      const { game, player1 } = createStartedGame()

      game.drawCards(player1.id, 1)
      game.performStateBasedActions()

      // Flag should be cleared after game ends
      expect(game.hasAttemptedDrawFromEmptyLibrary(player1.id)).toBe(false)
    })
  })

  describe("Simultaneous Loss (Draw)", () => {
    test("both players losing simultaneously results in DRAW", () => {
      const { game, player1, player2 } = createStartedGame()

      // Both players at 0 life
      player1.adjustLifeTotal(-20)
      player2.adjustLifeTotal(-20)

      game.performStateBasedActions()

      // Game should be finished with a draw
      expect(game.getLifecycleState()).toBe(GameLifecycleState.FINISHED)
      const outcome = game.getOutcome()
      expect(outcome).not.toBeNull()
      expect(outcome?.type).toBe("DRAW")
      if (outcome?.type === "DRAW") {
        expect(outcome.reason).toBe("SIMULTANEOUS_LOSS")
      }
    })

    test("both players attempting empty library draw results in DRAW", () => {
      const { game, player1, player2 } = createStartedGame()

      // Both attempt to draw from empty library
      game.drawCards(player1.id, 1)
      game.drawCards(player2.id, 1)

      game.performStateBasedActions()

      // Game should be finished with a draw
      expect(game.getLifecycleState()).toBe(GameLifecycleState.FINISHED)
      const outcome = game.getOutcome()
      expect(outcome?.type).toBe("DRAW")
      if (outcome?.type === "DRAW") {
        expect(outcome.reason).toBe("SIMULTANEOUS_LOSS")
      }
    })

    test("one player life loss, one player empty library draw results in DRAW", () => {
      const { game, player1, player2 } = createStartedGame()

      // Player1 loses via life total
      player1.adjustLifeTotal(-20)

      // Player2 loses via empty library draw
      game.drawCards(player2.id, 1)

      game.performStateBasedActions()

      // Game should be finished with a draw
      expect(game.getLifecycleState()).toBe(GameLifecycleState.FINISHED)
      const outcome = game.getOutcome()
      expect(outcome?.type).toBe("DRAW")
      if (outcome?.type === "DRAW") {
        expect(outcome.reason).toBe("SIMULTANEOUS_LOSS")
      }
    })

    test("no winner is declared in a draw", () => {
      const { game, player1, player2 } = createStartedGame()

      player1.adjustLifeTotal(-20)
      player2.adjustLifeTotal(-20)

      game.performStateBasedActions()

      const outcome = game.getOutcome()
      expect(outcome?.type).toBe("DRAW")
      // Draw outcome should not have a winnerId field
      if (outcome?.type === "DRAW") {
        expect(outcome).not.toHaveProperty("winnerId")
      }
    })
  })

  describe("Action Rejection in FINISHED State", () => {
    test("casting spell in finished game fails with clear error", () => {
      const { game, player1, player2 } = createStartedGame()

      // End the game
      player1.adjustLifeTotal(-20)
      game.performStateBasedActions()

      expect(game.getLifecycleState()).toBe(GameLifecycleState.FINISHED)

      // Try to cast a spell
      expect(() => {
        game.apply({
          type: "CAST_SPELL",
          playerId: player2.id,
          cardId: "some-card",
          targets: [],
        })
      }).toThrow(/game.*finished|game.*over/i)
    })

    test("passing priority in finished game fails", () => {
      const { game, player1, player2 } = createStartedGame()

      player1.adjustLifeTotal(-20)
      game.performStateBasedActions()

      expect(() => {
        game.apply({
          type: "PASS_PRIORITY",
          playerId: player2.id,
        })
      }).toThrow(/game.*finished|game.*over/i)
    })

    test("playing land in finished game fails", () => {
      const { game, player1, player2 } = createStartedGame()

      player1.adjustLifeTotal(-20)
      game.performStateBasedActions()

      expect(() => {
        game.apply({
          type: "PLAY_LAND",
          playerId: player2.id,
          landId: "some-land",
        })
      }).toThrow(/game.*finished|game.*over/i)
    })

    test("declaring attackers in finished game fails", () => {
      const { game, player1, player2 } = createStartedGame()

      player1.adjustLifeTotal(-20)
      game.performStateBasedActions()

      expect(() => {
        game.apply({
          type: "DECLARE_ATTACKER",
          playerId: player2.id,
          creatureId: "some-creature",
        })
      }).toThrow(/game.*finished|game.*over/i)
    })

    test("advancing step in finished game fails", () => {
      const { game, player1, player2 } = createStartedGame()

      player1.adjustLifeTotal(-20)
      game.performStateBasedActions()

      expect(() => {
        game.apply({
          type: "ADVANCE_STEP",
          playerId: player2.id,
        })
      }).toThrow(/game.*finished|game.*over/i)
    })
  })

  describe("State Export in FINISHED State", () => {
    test("game state is queryable after game ends", () => {
      const { game, player1, player2 } = createStartedGame()

      // Add a creature to battlefield before ending
      const creature = createTestCreature(player1.id, "test-creature")
      addCreatureToBattlefield(game, player2.id, creature)

      // End the game
      player1.adjustLifeTotal(-20)
      game.performStateBasedActions()

      // Export should succeed
      const state = game.exportState()
      expect(state).toBeDefined()
      expect(state.gameId).toBe(game.id)
    })

    test("export includes finished status", () => {
      const { game, player1 } = createStartedGame()

      player1.adjustLifeTotal(-20)
      game.performStateBasedActions()

      const state = game.exportState()
      expect(state.lifecycleState).toBe(GameLifecycleState.FINISHED)
    })

    test("export includes outcome with winner information", () => {
      const { game, player1, player2 } = createStartedGame()

      player1.adjustLifeTotal(-20)
      game.performStateBasedActions()

      const state = game.exportState()
      expect(state.outcome).not.toBeNull()
      expect(state.outcome?.type).toBe("WIN")
      if (state.outcome?.type === "WIN") {
        expect(state.outcome.winnerId).toBe(player2.id)
        expect(state.outcome.reason).toBe("LIFE_TOTAL")
      }
    })

    test("export includes outcome with draw information", () => {
      const { game, player1, player2 } = createStartedGame()

      player1.adjustLifeTotal(-20)
      player2.adjustLifeTotal(-20)
      game.performStateBasedActions()

      const state = game.exportState()
      expect(state.outcome?.type).toBe("DRAW")
      if (state.outcome?.type === "DRAW") {
        expect(state.outcome.reason).toBe("SIMULTANEOUS_LOSS")
      }
    })

    test("export includes final board state", () => {
      const { game, player1, player2 } = createStartedGame()

      // Add creatures to battlefield
      const creature1 = createTestCreature(player2.id, "creature-1")
      const creature2 = createTestCreature(player2.id, "creature-2")
      addCreatureToBattlefield(game, player2.id, creature1)
      addCreatureToBattlefield(game, player2.id, creature2)

      // End game
      player1.adjustLifeTotal(-20)
      game.performStateBasedActions()

      // Creatures should be in export
      const state = game.exportState()
      const player2State = state.players[player2.id]
      expect(player2State.zones.battlefield.cards).toHaveLength(2)
    })

    test("export includes final life totals", () => {
      const { game, player1, player2 } = createStartedGame()

      player1.adjustLifeTotal(-15) // Now at 5 life
      player2.adjustLifeTotal(-10) // Now at 10 life
      player1.adjustLifeTotal(-5) // Now at 0 life, loses

      game.performStateBasedActions()

      const state = game.exportState()
      expect(state.players[player1.id].lifeTotal).toBe(0)
      expect(state.players[player2.id].lifeTotal).toBe(10)
    })
  })

  describe("SBA continues to work after implementing win/lose", () => {
    test("creatures with lethal damage still die during SBA", () => {
      const { game, player1 } = createStartedGame()

      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)

      // Mark lethal damage on the creature (accessing private method through any cast for test)
      // In production code, damage would be marked through combat or spells
      // biome-ignore lint/suspicious/noExplicitAny: Test helper needs private access
      ;(game as any).markDamageOnCreature(creature.instanceId, 2)

      // Trigger SBA
      game.performStateBasedActions()

      // Creature should be in graveyard
      const graveyard = game.getPlayerState(player1.id).graveyard
      expect(graveyard.cards).toHaveLength(1)
      expect(graveyard.cards[0].instanceId).toBe(creature.instanceId)
    })

    test("player loss is checked before creature destruction", () => {
      const { game, player1, player2 } = createStartedGame()

      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)

      // Mark lethal damage on creature AND reduce player life to 0
      // biome-ignore lint/suspicious/noExplicitAny: Test helper needs private access
      ;(game as any).markDamageOnCreature(creature.instanceId, 2)
      player1.adjustLifeTotal(-20)

      // Trigger SBA
      game.performStateBasedActions()

      // Game should end (player loss takes priority)
      expect(game.getLifecycleState()).toBe(GameLifecycleState.FINISHED)
      const outcome = game.getOutcome()
      expect(outcome?.type).toBe("WIN")
      if (outcome?.type === "WIN") {
        expect(outcome.winnerId).toBe(player2.id)
      }

      // Creature should still be destroyed (SBA processes all actions)
      const graveyard = game.getPlayerState(player1.id).graveyard
      expect(graveyard.cards).toHaveLength(1)
    })
  })
})
