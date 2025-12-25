import { describe, expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import { ZoneNames } from "../../zones/Zone"
import { GraveyardReason } from "../Game"
import {
  addCreatureToBattlefield,
  addSpellToHand,
  createGameInMainPhase,
  createTestCreature,
  resolveStack,
} from "./helpers"

/**
 * Tests for Permanent Types MVP
 *
 * This test suite validates that all core permanent types
 * (Creature, Land, Artifact, Enchantment, Planeswalker)
 * can enter the battlefield and coexist correctly.
 *
 * Scope:
 * - Identity and structure of permanent types
 * - Battlefield presence and zone transitions
 * - Type checking and targeting by type
 * - Coexistence of multiple permanent types
 *
 * Out of scope (explicitly NOT tested here):
 * - Auras and Equipment (attachment rules deferred)
 * - Damage rules for planeswalkers (handled later with combat)
 * - Loyalty abilities (placeholder only in MVP)
 */
describe("Permanent Types — MVP", () => {
  describe("Artifact permanents", () => {
    test("spell that creates an Artifact enters battlefield as Artifact permanent", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      // Create an artifact spell
      const artifactSpell: CardInstance = {
        instanceId: "test-artifact-1",
        definition: {
          id: "test-artifact-def",
          name: "Test Artifact",
          types: ["ARTIFACT"],
        },
        ownerId: player1.id,
      }

      addSpellToHand(game, player1.id, artifactSpell)

      // Cast the artifact spell
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: artifactSpell.instanceId,
        targets: [],
      })

      // Verify it's on the stack
      const stack = game.getStack()
      expect(stack).toHaveLength(1)

      // Resolve the stack
      resolveStack(game, player2.id, player1.id)

      // Verify artifact is on the battlefield
      const battlefield = game.getPlayerState(player1.id).battlefield.cards
      expect(battlefield).toHaveLength(1)
      expect(battlefield[0]?.instanceId).toBe("test-artifact-1")
      expect(battlefield[0]?.definition.types).toContain("ARTIFACT")
    })

    test("artifact permanent can have activated abilities", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      let abilityActivated = false

      // Create an artifact with an activated ability
      const artifactWithAbility: CardInstance = {
        instanceId: "artifact-with-ability",
        definition: {
          id: "artifact-def",
          name: "Artifact With Ability",
          types: ["ARTIFACT"],
          activatedAbility: {
            cost: {
              type: "TAP_SELF",
              validate: () => true,
              pay: (game) => {
                game.tapPermanent("artifact-with-ability")
              },
            },
            effect: {
              resolve: () => {
                abilityActivated = true
              },
            },
          },
        },
        ownerId: player1.id,
      }

      // Add artifact to battlefield
      game.enterBattlefield(artifactWithAbility, player1.id)

      // Activate the ability
      game.apply({
        type: "ACTIVATE_ABILITY",
        playerId: player1.id,
        permanentId: "artifact-with-ability",
      })

      // Resolve the stack
      resolveStack(game, player2.id, player1.id)

      // Verify the ability executed
      expect(abilityActivated).toBe(true)
    })
  })

  describe("Enchantment permanents", () => {
    test("spell that creates an Enchantment enters battlefield as Enchantment permanent", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      // Create an enchantment spell
      const enchantmentSpell: CardInstance = {
        instanceId: "test-enchantment-1",
        definition: {
          id: "test-enchantment-def",
          name: "Test Enchantment",
          types: ["ENCHANTMENT"],
        },
        ownerId: player1.id,
      }

      addSpellToHand(game, player1.id, enchantmentSpell)

      // Cast the enchantment spell
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: enchantmentSpell.instanceId,
        targets: [],
      })

      // Verify it's on the stack
      const stack = game.getStack()
      expect(stack).toHaveLength(1)

      // Resolve the stack
      resolveStack(game, player2.id, player1.id)

      // Verify enchantment is on the battlefield
      const battlefield = game.getPlayerState(player1.id).battlefield.cards
      expect(battlefield).toHaveLength(1)
      expect(battlefield[0]?.instanceId).toBe("test-enchantment-1")
      expect(battlefield[0]?.definition.types).toContain("ENCHANTMENT")
    })

    test("enchantment permanent can have triggered abilities", () => {
      const { game, player1 } = createGameInMainPhase()

      let triggerFired = false

      // Create an enchantment with a triggered ability
      const enchantmentWithTrigger: CardInstance = {
        instanceId: "enchantment-with-trigger",
        definition: {
          id: "enchantment-def",
          name: "Enchantment With Trigger",
          types: ["ENCHANTMENT"],
          triggers: [
            {
              eventType: "ZONE_CHANGED",
              condition: (_, event, source) =>
                event.card.instanceId === source.instanceId &&
                event.toZone === ZoneNames.BATTLEFIELD,
              effect: () => {
                triggerFired = true
              },
            },
          ],
        },
        ownerId: player1.id,
      }

      // Add enchantment to battlefield (should trigger)
      game.enterBattlefield(enchantmentWithTrigger, player1.id)

      // Verify the trigger fired
      expect(triggerFired).toBe(true)
    })
  })

  describe("Planeswalker permanents", () => {
    test("spell that creates a Planeswalker enters battlefield as Planeswalker permanent", () => {
      const { game, player1, player2 } = createGameInMainPhase()

      // Create a planeswalker spell
      const planeswalkerSpell: CardInstance = {
        instanceId: "test-planeswalker-1",
        definition: {
          id: "test-planeswalker-def",
          name: "Test Planeswalker",
          types: ["PLANESWALKER"],
        },
        ownerId: player1.id,
      }

      addSpellToHand(game, player1.id, planeswalkerSpell)

      // Cast the planeswalker spell
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: planeswalkerSpell.instanceId,
        targets: [],
      })

      // Verify it's on the stack
      const stack = game.getStack()
      expect(stack).toHaveLength(1)

      // Resolve the stack
      resolveStack(game, player2.id, player1.id)

      // Verify planeswalker is on the battlefield
      const battlefield = game.getPlayerState(player1.id).battlefield.cards
      expect(battlefield).toHaveLength(1)
      expect(battlefield[0]?.instanceId).toBe("test-planeswalker-1")
      expect(battlefield[0]?.definition.types).toContain("PLANESWALKER")
    })

    test("planeswalker can have ETB trigger", () => {
      const { game, player1 } = createGameInMainPhase()

      let etbFired = false

      // Create a planeswalker with ETB trigger
      const planeswalkerWithETB: CardInstance = {
        instanceId: "planeswalker-with-etb",
        definition: {
          id: "planeswalker-def",
          name: "Planeswalker With ETB",
          types: ["PLANESWALKER"],
          triggers: [
            {
              eventType: "ZONE_CHANGED",
              condition: (_, event, source) =>
                event.card.instanceId === source.instanceId &&
                event.toZone === ZoneNames.BATTLEFIELD,
              effect: () => {
                etbFired = true
              },
            },
          ],
        },
        ownerId: player1.id,
      }

      // Add planeswalker to battlefield
      game.enterBattlefield(planeswalkerWithETB, player1.id)

      // Verify ETB trigger fired
      expect(etbFired).toBe(true)
    })
  })

  describe("Creature permanents (existing behavior)", () => {
    test("creature behavior still works unchanged", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "test-creature-1")

      // Add creature to battlefield
      addCreatureToBattlefield(game, player1.id, creature)

      // Verify creature is on battlefield
      const battlefield = game.getPlayerState(player1.id).battlefield.cards
      expect(battlefield).toHaveLength(1)
      expect(battlefield[0]?.definition.types).toContain("CREATURE")

      // Verify creature state was initialized
      const creatureState = game.getCreatureState("test-creature-1")
      expect(creatureState.isTapped).toBe(false)
      expect(creatureState.isAttacking).toBe(false)
      expect(creatureState.hasAttackedThisTurn).toBe(false)
    })
  })

  describe("Land permanents (existing behavior)", () => {
    test("land behavior still works unchanged", () => {
      const { game, player1, dummyLandInstanceId } = createGameInMainPhase()

      // Play the dummy land
      game.apply({
        type: "PLAY_LAND",
        playerId: player1.id,
        cardId: dummyLandInstanceId,
      })

      // Verify land is on battlefield
      const battlefield = game.getPlayerState(player1.id).battlefield.cards
      expect(battlefield).toHaveLength(1)
      expect(battlefield[0]?.definition.types).toContain("LAND")

      // Verify it went directly to battlefield (not through stack)
      const stack = game.getStack()
      expect(stack).toHaveLength(0)
    })
  })

  describe("Multiple permanent types coexisting", () => {
    test("battlefield can contain multiple different permanent types simultaneously", () => {
      const { game, player1, player2, dummyLandInstanceId } =
        createGameInMainPhase()

      // Create one of each permanent type
      const creature = createTestCreature(player1.id, "creature-1")

      const artifact: CardInstance = {
        instanceId: "artifact-1",
        definition: {
          id: "artifact-def",
          name: "Test Artifact",
          types: ["ARTIFACT"],
        },
        ownerId: player1.id,
      }

      const enchantment: CardInstance = {
        instanceId: "enchantment-1",
        definition: {
          id: "enchantment-def",
          name: "Test Enchantment",
          types: ["ENCHANTMENT"],
        },
        ownerId: player1.id,
      }

      const planeswalker: CardInstance = {
        instanceId: "planeswalker-1",
        definition: {
          id: "planeswalker-def",
          name: "Test Planeswalker",
          types: ["PLANESWALKER"],
        },
        ownerId: player1.id,
      }

      // Add creature directly to battlefield
      addCreatureToBattlefield(game, player1.id, creature)

      // Play land
      game.apply({
        type: "PLAY_LAND",
        playerId: player1.id,
        cardId: dummyLandInstanceId,
      })

      // Cast artifact
      addSpellToHand(game, player1.id, artifact)
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: artifact.instanceId,
        targets: [],
      })
      resolveStack(game, player2.id, player1.id)

      // Cast enchantment
      addSpellToHand(game, player1.id, enchantment)
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: enchantment.instanceId,
        targets: [],
      })
      resolveStack(game, player2.id, player1.id)

      // Cast planeswalker
      addSpellToHand(game, player1.id, planeswalker)
      game.apply({
        type: "CAST_SPELL",
        playerId: player1.id,
        cardId: planeswalker.instanceId,
        targets: [],
      })
      resolveStack(game, player2.id, player1.id)

      // Verify all permanents coexist on battlefield
      const battlefield = game.getPlayerState(player1.id).battlefield.cards
      expect(battlefield).toHaveLength(5)

      const types = battlefield.flatMap((card) => card.definition.types)
      expect(types).toContain("CREATURE")
      expect(types).toContain("LAND")
      expect(types).toContain("ARTIFACT")
      expect(types).toContain("ENCHANTMENT")
      expect(types).toContain("PLANESWALKER")
    })

    test("artifact creature (multiple types on single card) works correctly", () => {
      const { game, player1 } = createGameInMainPhase()

      const artifactCreature: CardInstance = {
        instanceId: "artifact-creature-1",
        definition: {
          id: "artifact-creature-def",
          name: "Test Artifact Creature",
          types: ["ARTIFACT", "CREATURE"],
        },
        ownerId: player1.id,
      }

      // Add to battlefield
      game.enterBattlefield(artifactCreature, player1.id)

      // Verify it has both types
      const battlefield = game.getPlayerState(player1.id).battlefield.cards
      expect(battlefield).toHaveLength(1)
      expect(battlefield[0]?.definition.types).toContain("ARTIFACT")
      expect(battlefield[0]?.definition.types).toContain("CREATURE")

      // Verify creature state was initialized (because it's a creature)
      const creatureState = game.getCreatureState("artifact-creature-1")
      expect(creatureState).toBeDefined()
      expect(creatureState.isTapped).toBe(false)
    })
  })

  describe("Type-based targeting and filtering", () => {
    test("can distinguish between creature and non-creature permanents", () => {
      const { game, player1 } = createGameInMainPhase()

      const creature = createTestCreature(player1.id, "creature-1")
      const artifact: CardInstance = {
        instanceId: "artifact-1",
        definition: {
          id: "artifact-def",
          name: "Test Artifact",
          types: ["ARTIFACT"],
        },
        ownerId: player1.id,
      }

      // Add both to battlefield
      addCreatureToBattlefield(game, player1.id, creature)
      game.enterBattlefield(artifact, player1.id)

      // Get battlefield
      const battlefield = game.getPlayerState(player1.id).battlefield.cards

      // Filter for creatures only
      const creatures = battlefield.filter((card) =>
        card.definition.types.includes("CREATURE"),
      )
      expect(creatures).toHaveLength(1)
      expect(creatures[0]?.instanceId).toBe("creature-1")

      // Filter for non-creatures
      const nonCreatures = battlefield.filter(
        (card) => !card.definition.types.includes("CREATURE"),
      )
      expect(nonCreatures).toHaveLength(1)
      expect(nonCreatures[0]?.instanceId).toBe("artifact-1")
    })

    test("can filter permanents by specific type", () => {
      const { game, player1 } = createGameInMainPhase()

      const artifact1: CardInstance = {
        instanceId: "artifact-1",
        definition: {
          id: "artifact-def",
          name: "Test Artifact 1",
          types: ["ARTIFACT"],
        },
        ownerId: player1.id,
      }

      const artifact2: CardInstance = {
        instanceId: "artifact-2",
        definition: {
          id: "artifact-def",
          name: "Test Artifact 2",
          types: ["ARTIFACT"],
        },
        ownerId: player1.id,
      }

      const enchantment: CardInstance = {
        instanceId: "enchantment-1",
        definition: {
          id: "enchantment-def",
          name: "Test Enchantment",
          types: ["ENCHANTMENT"],
        },
        ownerId: player1.id,
      }

      const creature = createTestCreature(player1.id, "creature-1")

      // Add all to battlefield
      game.enterBattlefield(artifact1, player1.id)
      game.enterBattlefield(artifact2, player1.id)
      game.enterBattlefield(enchantment, player1.id)
      addCreatureToBattlefield(game, player1.id, creature)

      const battlefield = game.getPlayerState(player1.id).battlefield.cards

      // Filter for artifacts only
      const artifacts = battlefield.filter((card) =>
        card.definition.types.includes("ARTIFACT"),
      )
      expect(artifacts).toHaveLength(2)

      // Filter for enchantments only
      const enchantments = battlefield.filter((card) =>
        card.definition.types.includes("ENCHANTMENT"),
      )
      expect(enchantments).toHaveLength(1)

      // Filter for planeswalkers (should be none)
      const planeswalkers = battlefield.filter((card) =>
        card.definition.types.includes("PLANESWALKER"),
      )
      expect(planeswalkers).toHaveLength(0)
    })
  })

  describe("Zone transitions for all permanent types", () => {
    test("artifacts can be moved to graveyard", () => {
      const { game, player1 } = createGameInMainPhase()

      const artifact: CardInstance = {
        instanceId: "artifact-1",
        definition: {
          id: "artifact-def",
          name: "Test Artifact",
          types: ["ARTIFACT"],
        },
        ownerId: player1.id,
      }

      game.enterBattlefield(artifact, player1.id)

      // Verify it's on battlefield
      let battlefield = game.getPlayerState(player1.id).battlefield.cards
      expect(battlefield).toHaveLength(1)

      // Move to graveyard
      game.movePermanentToGraveyard("artifact-1", GraveyardReason.SACRIFICE)

      // Verify it's in graveyard
      const graveyard = game.getGraveyard(player1.id)
      expect(graveyard).toHaveLength(1)
      expect(graveyard[0]?.instanceId).toBe("artifact-1")

      // Verify it's not on battlefield
      battlefield = game.getPlayerState(player1.id).battlefield.cards
      expect(battlefield).toHaveLength(0)
    })

    test("enchantments can be moved to graveyard", () => {
      const { game, player1 } = createGameInMainPhase()

      const enchantment: CardInstance = {
        instanceId: "enchantment-1",
        definition: {
          id: "enchantment-def",
          name: "Test Enchantment",
          types: ["ENCHANTMENT"],
        },
        ownerId: player1.id,
      }

      game.enterBattlefield(enchantment, player1.id)

      // Move to graveyard
      game.movePermanentToGraveyard("enchantment-1", GraveyardReason.DESTROY)

      // Verify zone transition
      const graveyard = game.getGraveyard(player1.id)
      expect(graveyard).toHaveLength(1)
      expect(graveyard[0]?.instanceId).toBe("enchantment-1")
    })

    test("planeswalkers can be moved to graveyard", () => {
      const { game, player1 } = createGameInMainPhase()

      const planeswalker: CardInstance = {
        instanceId: "planeswalker-1",
        definition: {
          id: "planeswalker-def",
          name: "Test Planeswalker",
          types: ["PLANESWALKER"],
        },
        ownerId: player1.id,
      }

      game.enterBattlefield(planeswalker, player1.id)

      // Move to graveyard
      game.movePermanentToGraveyard(
        "planeswalker-1",
        GraveyardReason.STATE_BASED,
      )

      // Verify zone transition
      const graveyard = game.getGraveyard(player1.id)
      expect(graveyard).toHaveLength(1)
      expect(graveyard[0]?.instanceId).toBe("planeswalker-1")
    })
  })
})
