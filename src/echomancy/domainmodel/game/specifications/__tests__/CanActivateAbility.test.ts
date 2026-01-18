import { describe, expect, test } from "bun:test"
import {
  addCreatureToBattlefield,
  createGameInMainPhase,
  createTestCreature,
} from "../../__tests__/helpers"
import { CanActivateAbility } from "../CanActivateAbility"

describe("CanActivateAbility Specification", () => {
  test("returns false when player has no permanents with abilities", () => {
    const { game, player1 } = createGameInMainPhase()
    const spec = new CanActivateAbility()

    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(false)
  })

  test("returns true when player has permanent with tap ability", () => {
    const { game, player1 } = createGameInMainPhase()

    // Create a creature with a tap ability
    const creature = {
      ...createTestCreature(player1.id),
      definition: {
        ...createTestCreature(player1.id).definition,
        activatedAbility: {
          cost: { type: "TAP" as const },
          effect: { type: "DRAW_CARD" as const, count: 1 },
        },
      },
    }
    addCreatureToBattlefield(game, player1.id, creature)

    const spec = new CanActivateAbility()
    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(true)
  })

  test("returns false when permanent with ability is tapped", () => {
    const { game, player1 } = createGameInMainPhase()

    const creature = {
      ...createTestCreature(player1.id),
      definition: {
        ...createTestCreature(player1.id).definition,
        activatedAbility: {
          cost: { type: "TAP" as const },
          effect: { type: "DRAW_CARD" as const, count: 1 },
        },
      },
    }
    addCreatureToBattlefield(game, player1.id, creature)

    // Tap the creature
    game.apply({
      type: "ACTIVATE_ABILITY",
      playerId: player1.id,
      permanentId: creature.instanceId,
    })

    const spec = new CanActivateAbility()
    expect(spec.isSatisfiedBy({ game, playerId: player1.id })).toBe(false)
  })
})
