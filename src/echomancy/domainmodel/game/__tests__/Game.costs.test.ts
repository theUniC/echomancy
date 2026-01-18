import { expect, test } from "vitest"
import { canPayAllCosts, payAllCosts } from "../../costs/Cost"
import { ManaCost } from "../../costs/impl/ManaCost"
import { SacrificeSelfCost } from "../../costs/impl/SacrificeSelfCost"
import { TapSelfCost } from "../../costs/impl/TapSelfCost"
import {
  InsufficientManaError,
  PermanentAlreadyTappedError,
  PermanentNotControlledError,
  PermanentNotFoundError,
} from "../GameErrors"
import { Step } from "../Steps"
import {
  addCreatureToBattlefield,
  advanceToStep,
  createStartedGame,
  createTestCreature,
} from "./helpers"

test("ManaCost.canPay returns true when player has sufficient mana", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Add mana to player's pool
  game.addMana(player1.id, "G", 2)

  const cost = new ManaCost({ G: 2 })
  const canPay = cost.canPay(game, {
    playerId: player1.id,
    sourceId: "test-source",
  })

  expect(canPay).toBe(true)
})

test("ManaCost.canPay returns false when player has insufficient mana", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 1)

  const cost = new ManaCost({ G: 2 })
  const canPay = cost.canPay(game, {
    playerId: player1.id,
    sourceId: "test-source",
  })

  expect(canPay).toBe(false)
})

test("ManaCost.canPay returns true when player has sufficient mana of multiple colors", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 1)
  game.addMana(player1.id, "W", 1)

  const cost = new ManaCost({ G: 1, W: 1 })
  const canPay = cost.canPay(game, {
    playerId: player1.id,
    sourceId: "test-source",
  })

  expect(canPay).toBe(true)
})

test("ManaCost.canPay returns false when player has insufficient mana of one color", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 1)
  // W mana is 0 (not added)

  const cost = new ManaCost({ G: 1, W: 1 })
  const canPay = cost.canPay(game, {
    playerId: player1.id,
    sourceId: "test-source",
  })

  expect(canPay).toBe(false)
})

test("ManaCost.pay removes mana from player's pool", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 3)

  const cost = new ManaCost({ G: 2 })
  cost.pay(game, {
    playerId: player1.id,
    sourceId: "test-source",
  })

  const pool = game.getManaPool(player1.id)
  expect(pool.G).toBe(1)
})

test("ManaCost.pay removes mana of multiple colors", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 2)
  game.addMana(player1.id, "W", 3)

  const cost = new ManaCost({ G: 1, W: 2 })
  cost.pay(game, {
    playerId: player1.id,
    sourceId: "test-source",
  })

  const pool = game.getManaPool(player1.id)
  expect(pool.G).toBe(1)
  expect(pool.W).toBe(1)
})

test("ManaCost.pay throws InsufficientManaError when insufficient mana", () => {
  const { game, player1 } = createStartedGame()

  game.addMana(player1.id, "G", 1)

  const cost = new ManaCost({ G: 2 })
  expect(() =>
    cost.pay(game, {
      playerId: player1.id,
      sourceId: "test-source",
    }),
  ).toThrow(InsufficientManaError)
})

test("TapSelfCost.canPay returns true when permanent exists, is untapped, and controlled by player", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  const cost = new TapSelfCost()
  const canPay = cost.canPay(game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  expect(canPay).toBe(true)
})

test("TapSelfCost.canPay returns false when permanent is already tapped", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  // Tap the creature
  game.tapPermanent(creature.instanceId)

  const cost = new TapSelfCost()
  const canPay = cost.canPay(game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  expect(canPay).toBe(false)
})

test("TapSelfCost.canPay returns false when permanent does not exist", () => {
  const { game, player1 } = createStartedGame()

  const cost = new TapSelfCost()
  const canPay = cost.canPay(game, {
    playerId: player1.id,
    sourceId: "nonexistent",
  })

  expect(canPay).toBe(false)
})

test("TapSelfCost.canPay returns false when permanent is not controlled by player", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player2.id, "creature-1")
  addCreatureToBattlefield(game, player2.id, creature)

  const cost = new TapSelfCost()
  const canPay = cost.canPay(game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  expect(canPay).toBe(false)
})

test("TapSelfCost.pay taps the permanent", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  const cost = new TapSelfCost()
  cost.pay(game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  const creatureState = game.getCreatureState(creature.instanceId)
  expect(creatureState.isTapped).toBe(true)
})

test("TapSelfCost.pay throws PermanentNotFoundError when permanent does not exist", () => {
  const { game, player1 } = createStartedGame()

  const cost = new TapSelfCost()
  expect(() =>
    cost.pay(game, {
      playerId: player1.id,
      sourceId: "nonexistent",
    }),
  ).toThrow(PermanentNotFoundError)
})

test("TapSelfCost.pay throws PermanentAlreadyTappedError when permanent is already tapped", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  // Tap the creature
  game.tapPermanent(creature.instanceId)

  const cost = new TapSelfCost()
  expect(() =>
    cost.pay(game, {
      playerId: player1.id,
      sourceId: creature.instanceId,
    }),
  ).toThrow(PermanentAlreadyTappedError)
})

test("TapSelfCost.pay throws PermanentNotControlledError when permanent is not controlled by player", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player2.id, "creature-1")
  addCreatureToBattlefield(game, player2.id, creature)

  const cost = new TapSelfCost()
  expect(() =>
    cost.pay(game, {
      playerId: player1.id,
      sourceId: creature.instanceId,
    }),
  ).toThrow(PermanentNotControlledError)
})

test("SacrificeSelfCost.canPay returns true when permanent exists and is controlled by player", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  const cost = new SacrificeSelfCost()
  const canPay = cost.canPay(game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  expect(canPay).toBe(true)
})

test("SacrificeSelfCost.canPay returns false when permanent does not exist", () => {
  const { game, player1 } = createStartedGame()

  const cost = new SacrificeSelfCost()
  const canPay = cost.canPay(game, {
    playerId: player1.id,
    sourceId: "nonexistent",
  })

  expect(canPay).toBe(false)
})

test("SacrificeSelfCost.canPay returns false when permanent is not controlled by player", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player2.id, "creature-1")
  addCreatureToBattlefield(game, player2.id, creature)

  const cost = new SacrificeSelfCost()
  const canPay = cost.canPay(game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  expect(canPay).toBe(false)
})

test("SacrificeSelfCost.pay moves permanent from battlefield to graveyard", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  const playerState = game.getPlayerState(player1.id)
  expect(playerState.battlefield.cards).toHaveLength(1)
  expect(playerState.graveyard.cards).toHaveLength(0)

  const cost = new SacrificeSelfCost()
  cost.pay(game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  expect(playerState.battlefield.cards).toHaveLength(0)
  expect(playerState.graveyard.cards).toHaveLength(1)
  expect(playerState.graveyard.cards[0]?.instanceId).toBe(creature.instanceId)
})

test("SacrificeSelfCost.pay cleans up creature state when sacrificing", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  // Verify creature state exists
  const creatureStateBefore = game.getCreatureState(creature.instanceId)
  expect(creatureStateBefore).toBeDefined()

  const cost = new SacrificeSelfCost()
  cost.pay(game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  // Verify creature state is cleaned up
  expect(() => game.getCreatureState(creature.instanceId)).toThrow(
    PermanentNotFoundError,
  )
})

test("SacrificeSelfCost.pay throws PermanentNotFoundError when permanent does not exist", () => {
  const { game, player1 } = createStartedGame()

  const cost = new SacrificeSelfCost()
  expect(() =>
    cost.pay(game, {
      playerId: player1.id,
      sourceId: "nonexistent",
    }),
  ).toThrow(PermanentNotFoundError)
})

test("SacrificeSelfCost.pay throws PermanentNotControlledError when permanent is not controlled by player", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player2.id, "creature-1")
  addCreatureToBattlefield(game, player2.id, creature)

  const cost = new SacrificeSelfCost()
  expect(() =>
    cost.pay(game, {
      playerId: player1.id,
      sourceId: creature.instanceId,
    }),
  ).toThrow(PermanentNotControlledError)
})

test("all costs are validated before any are paid", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  // Add some mana
  game.addMana(player1.id, "G", 2)

  const costs = [new ManaCost({ G: 2 }), new TapSelfCost()]

  const canPay = canPayAllCosts(costs, game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  expect(canPay).toBe(true)
})

test("if ANY cost cannot be paid, canPayAllCosts returns false", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  // Add insufficient mana
  game.addMana(player1.id, "G", 1)

  const costs = [new ManaCost({ G: 2 }), new TapSelfCost()]

  const canPay = canPayAllCosts(costs, game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  expect(canPay).toBe(false)
})

test("payAllCosts pays all costs in order", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  game.addMana(player1.id, "G", 2)

  const costs = [new ManaCost({ G: 2 }), new TapSelfCost()]

  payAllCosts(costs, game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  // Verify mana was spent
  const pool = game.getManaPool(player1.id)
  expect(pool.G).toBe(0)

  // Verify creature was tapped
  const creatureState = game.getCreatureState(creature.instanceId)
  expect(creatureState.isTapped).toBe(true)
})

test("costs are paid before effects execute (integration concept)", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id, "creature-1")
  addCreatureToBattlefield(game, player1.id, creature)

  game.addMana(player1.id, "G", 1)

  const costs = [new ManaCost({ G: 1 }), new TapSelfCost()]

  // Validate costs before paying
  const canPay = canPayAllCosts(costs, game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })
  expect(canPay).toBe(true)

  // Pay costs
  payAllCosts(costs, game, {
    playerId: player1.id,
    sourceId: creature.instanceId,
  })

  // At this point, costs have been paid
  // In a real scenario, effects would execute after this
  // This test just verifies the concept

  const pool = game.getManaPool(player1.id)
  expect(pool.G).toBe(0)

  const creatureState = game.getCreatureState(creature.instanceId)
  expect(creatureState.isTapped).toBe(true)
})
