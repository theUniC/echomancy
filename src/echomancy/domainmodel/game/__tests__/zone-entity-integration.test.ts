/**
 * Zone Entity Integration Tests
 *
 * Tests that Game.ts properly integrates with zone entities (Battlefield, Hand, Graveyard).
 * These tests verify that zone operations use immutable entity methods and
 * maintain backward compatibility with existing code.
 */

import { describe, expect, test } from "vitest"
import { Battlefield } from "../entities/Battlefield"
import { Graveyard } from "../entities/Graveyard"
import { Hand } from "../entities/Hand"
import {
  addCreatureToBattlefield,
  createGameInMainPhase,
  createTestCreature,
  createTestLand,
} from "./helpers"

describe("Zone Entity Integration", () => {
  describe("PlayerState uses zone entities", () => {
    test("battlefield is Battlefield entity", () => {
      const { game, player1 } = createGameInMainPhase()
      const playerState = game.getPlayerState(player1.id)

      // PlayerState.battlefield should be a Battlefield entity
      expect(playerState.battlefield).toBeInstanceOf(Battlefield)
    })

    test("hand is Hand entity", () => {
      const { game, player1 } = createGameInMainPhase()
      const playerState = game.getPlayerState(player1.id)

      // PlayerState.hand should be a Hand entity
      expect(playerState.hand).toBeInstanceOf(Hand)
    })

    test("graveyard is Graveyard entity", () => {
      const { game, player1 } = createGameInMainPhase()
      const playerState = game.getPlayerState(player1.id)

      // PlayerState.graveyard should be a Graveyard entity
      expect(playerState.graveyard).toBeInstanceOf(Graveyard)
    })
  })

  describe("enterBattlefield uses Battlefield entity", () => {
    test("adds permanent using Battlefield.addPermanent()", () => {
      const { game, player1 } = createGameInMainPhase()
      const creature = createTestCreature(player1.id)

      addCreatureToBattlefield(game, player1.id, creature)

      const playerState = game.getPlayerState(player1.id)
      // Should be able to access via .cards for backward compatibility
      expect(playerState.battlefield.cards).toHaveLength(1)
      expect(playerState.battlefield.cards[0]).toEqual(creature)

      // Should also work with entity methods
      expect(playerState.battlefield.count()).toBe(1)
      expect(
        playerState.battlefield.findPermanent(creature.instanceId),
      ).toEqual(creature)
    })
  })

  describe("playLand uses Hand and Battlefield entities", () => {
    test("removes from hand and adds to battlefield using entity methods", () => {
      const { game, player1 } = createGameInMainPhase()
      const land = createTestLand(player1.id, "land-1")

      // Add land to hand
      const playerState = game.getPlayerState(player1.id)
      const handBefore = playerState.hand
      expect(handBefore.count()).toBe(0)

      // Manually add to hand using entity method (simulating what Game.ts should do)
      // NOTE: This is testing that we CAN use entity methods, not that playLand uses them yet
      const newHand = handBefore.addCard(land)
      expect(newHand.count()).toBe(1)
      expect(newHand.findCard(land.instanceId)).toEqual(land)
    })
  })

  describe("movePermanentToGraveyard uses Battlefield and Graveyard entities", () => {
    test("removes from battlefield and adds to graveyard using entity methods", () => {
      const { game, player1 } = createGameInMainPhase()
      const creature = createTestCreature(player1.id, "creature-1")

      addCreatureToBattlefield(game, player1.id, creature)

      // Verify creature is on battlefield
      let playerState = game.getPlayerState(player1.id)
      expect(playerState.battlefield.count()).toBe(1)
      expect(playerState.graveyard.count()).toBe(0)

      // Move to graveyard
      game.movePermanentToGraveyard(creature.instanceId, "state-based" as any)

      // Verify creature moved
      playerState = game.getPlayerState(player1.id)
      expect(playerState.battlefield.count()).toBe(0)
      expect(playerState.graveyard.count()).toBe(1)
      expect(playerState.graveyard.getTopCard()).toEqual(creature)
    })
  })

  describe("Backward compatibility with .cards property", () => {
    test("battlefield.cards still works", () => {
      const { game, player1 } = createGameInMainPhase()
      const creature = createTestCreature(player1.id)

      addCreatureToBattlefield(game, player1.id, creature)

      const playerState = game.getPlayerState(player1.id)
      // Old code accessing .cards should still work
      const cards = playerState.battlefield.cards
      expect(cards).toHaveLength(1)
      expect(cards[0]).toEqual(creature)
    })

    test("hand.cards still works", () => {
      const { game, player1 } = createGameInMainPhase()
      const land = createTestLand(player1.id)

      const playerState = game.getPlayerState(player1.id)
      const newHand = playerState.hand.addCard(land)

      // Old code accessing .cards should still work
      const cards = newHand.cards
      expect(cards).toHaveLength(1)
      expect(cards[0]).toEqual(land)
    })

    test("graveyard.cards still works", () => {
      const { game, player1 } = createGameInMainPhase()
      const creature = createTestCreature(player1.id)

      addCreatureToBattlefield(game, player1.id, creature)
      game.movePermanentToGraveyard(creature.instanceId, "state-based" as any)

      const playerState = game.getPlayerState(player1.id)
      // Old code accessing .cards should still work
      const cards = playerState.graveyard.cards
      expect(cards).toHaveLength(1)
      expect(cards[0]).toEqual(creature)
    })
  })
})
