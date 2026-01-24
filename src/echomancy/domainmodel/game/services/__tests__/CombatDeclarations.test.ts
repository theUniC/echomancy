import { describe, expect, test } from "vitest"
import type { StaticAbility } from "../../../cards/CardDefinition"
import { StaticAbilities } from "../../../cards/CardDefinition"
import type { CardInstance } from "../../../cards/CardInstance"
import { Step } from "../../Steps"
import { PermanentState } from "../../valueobjects/PermanentState"
import {
  type CombatValidationContext,
  validateDeclareAttacker,
  validateDeclareBlocker,
} from "../CombatDeclarations"

function createMockContext(
  overrides: Partial<CombatValidationContext> = {},
): CombatValidationContext {
  return {
    currentStep: Step.DECLARE_ATTACKERS,
    currentPlayerId: "player1",
    getOpponentOf: () => "player2",
    getBattlefieldCards: () => [],
    isCreature: () => true,
    hasStaticAbility: () => false,
    getCreatureState: () => undefined,
    ...overrides,
  }
}

function createMockCreature(
  instanceId: string,
  staticAbilities?: StaticAbility[],
): CardInstance {
  return {
    instanceId,
    ownerId: "player1",
    definition: {
      id: `${instanceId}-def`,
      name: "Test Creature",
      types: ["CREATURE"],
      subtypes: [],
      manaCost: "",
      text: "",
      power: 2,
      toughness: 2,
      staticAbilities,
    },
  }
}

function createPermanentState(
  overrides: Partial<{
    isTapped: boolean
    isAttacking: boolean
    hasAttackedThisTurn: boolean
    hasSummoningSickness: boolean
    blockingCreatureId: string | null
    blockedBy: string | null
  }> = {},
): PermanentState {
  const creature = createMockCreature("test")
  let state = PermanentState.forCreature(creature)
  // Default summoning sickness is true, so clear it unless specified
  if (!overrides.hasSummoningSickness) {
    state = state.withSummoningSickness(false)
  }
  if (overrides.isTapped) {
    state = state.withTapped(true)
  }
  if (overrides.isAttacking) {
    state = state.withAttacking(true)
  }
  if (overrides.hasAttackedThisTurn) {
    state = state.withHasAttackedThisTurn(true)
  }
  if (overrides.hasSummoningSickness) {
    state = state.withSummoningSickness(true)
  }
  if (overrides.blockingCreatureId) {
    state = state.withBlockingCreatureId(overrides.blockingCreatureId)
  }
  if (overrides.blockedBy) {
    state = state.withBlockedBy(overrides.blockedBy)
  }
  return state
}

describe("CombatManager Service", () => {
  describe("validateDeclareAttacker", () => {
    test("throws if not DECLARE_ATTACKERS step", () => {
      const ctx = createMockContext({ currentStep: Step.FIRST_MAIN })

      expect(() =>
        validateDeclareAttacker(ctx, "player1", "creature-1"),
      ).toThrow()
    })

    test("throws if player is not current player", () => {
      const ctx = createMockContext({ currentPlayerId: "player1" })

      expect(() =>
        validateDeclareAttacker(ctx, "player2", "creature-1"),
      ).toThrow()
    })

    test("throws if creature not found on battlefield", () => {
      const ctx = createMockContext({
        getBattlefieldCards: () => [],
      })

      expect(() =>
        validateDeclareAttacker(ctx, "player1", "creature-1"),
      ).toThrow(/not found/)
    })

    test("throws if card is not a creature", () => {
      const creature = createMockCreature("creature-1")
      const ctx = createMockContext({
        getBattlefieldCards: () => [creature],
        isCreature: () => false,
      })

      expect(() =>
        validateDeclareAttacker(ctx, "player1", "creature-1"),
      ).toThrow(/not found/)
    })

    test("throws if creature has summoning sickness without Haste", () => {
      const creature = createMockCreature("creature-1")
      const creatureState = createPermanentState({ hasSummoningSickness: true })
      const ctx = createMockContext({
        getBattlefieldCards: () => [creature],
        getCreatureState: () => creatureState,
      })

      expect(() =>
        validateDeclareAttacker(ctx, "player1", "creature-1"),
      ).toThrow(/summoning sickness/)
    })

    test("allows attack if creature has Haste despite summoning sickness", () => {
      const creature = createMockCreature("creature-1", [StaticAbilities.HASTE])
      const creatureState = createPermanentState({ hasSummoningSickness: true })
      const ctx = createMockContext({
        getBattlefieldCards: () => [creature],
        getCreatureState: () => creatureState,
        hasStaticAbility: (_card, ability) => ability === StaticAbilities.HASTE,
      })

      const result = validateDeclareAttacker(ctx, "player1", "creature-1")

      expect(result.newCreatureState.creatureState?.isAttacking).toBe(true)
    })

    test("throws if creature is tapped", () => {
      const creature = createMockCreature("creature-1")
      const creatureState = createPermanentState({ isTapped: true })
      const ctx = createMockContext({
        getBattlefieldCards: () => [creature],
        getCreatureState: () => creatureState,
      })

      expect(() =>
        validateDeclareAttacker(ctx, "player1", "creature-1"),
      ).toThrow(/tapped/i)
    })

    test("throws if creature already attacked this turn", () => {
      const creature = createMockCreature("creature-1")
      const creatureState = createPermanentState({ hasAttackedThisTurn: true })
      const ctx = createMockContext({
        getBattlefieldCards: () => [creature],
        getCreatureState: () => creatureState,
      })

      expect(() =>
        validateDeclareAttacker(ctx, "player1", "creature-1"),
      ).toThrow(/already attacked/)
    })

    test("returns valid result with creature tapped for normal attacker", () => {
      const creature = createMockCreature("creature-1")
      const creatureState = createPermanentState()
      const ctx = createMockContext({
        getBattlefieldCards: () => [creature],
        getCreatureState: () => creatureState,
      })

      const result = validateDeclareAttacker(ctx, "player1", "creature-1")

      expect(result.creature).toBe(creature)
      expect(result.newCreatureState.creatureState?.isAttacking).toBe(true)
      expect(result.newCreatureState.creatureState?.hasAttackedThisTurn).toBe(
        true,
      )
      expect(result.newCreatureState.isTapped).toBe(true)
    })

    test("returns valid result without tapping for creature with Vigilance", () => {
      const creature = createMockCreature("creature-1", [
        StaticAbilities.VIGILANCE,
      ])
      const creatureState = createPermanentState()
      const ctx = createMockContext({
        getBattlefieldCards: () => [creature],
        getCreatureState: () => creatureState,
        hasStaticAbility: (_card, ability) =>
          ability === StaticAbilities.VIGILANCE,
      })

      const result = validateDeclareAttacker(ctx, "player1", "creature-1")

      expect(result.newCreatureState.creatureState?.isAttacking).toBe(true)
      expect(result.newCreatureState.isTapped).toBe(false)
    })
  })

  describe("validateDeclareBlocker", () => {
    test("throws if not DECLARE_BLOCKERS step", () => {
      const ctx = createMockContext({ currentStep: Step.FIRST_MAIN })

      expect(() =>
        validateDeclareBlocker(ctx, "player2", "blocker-1", "attacker-1"),
      ).toThrow()
    })

    test("throws if player is not defending player", () => {
      const ctx = createMockContext({
        currentStep: Step.DECLARE_BLOCKERS,
        currentPlayerId: "player1",
        getOpponentOf: () => "player2",
      })

      // player1 (attacking) trying to declare blockers should fail
      expect(() =>
        validateDeclareBlocker(ctx, "player1", "blocker-1", "attacker-1"),
      ).toThrow()
    })

    test("throws if blocker not found on battlefield", () => {
      const ctx = createMockContext({
        currentStep: Step.DECLARE_BLOCKERS,
        getBattlefieldCards: () => [],
      })

      expect(() =>
        validateDeclareBlocker(ctx, "player2", "blocker-1", "attacker-1"),
      ).toThrow(/not found/)
    })

    test("throws if blocker is tapped", () => {
      const blocker = createMockCreature("blocker-1")
      blocker.ownerId = "player2"
      const blockerState = createPermanentState({ isTapped: true })
      const attacker = createMockCreature("attacker-1")
      const attackerState = createPermanentState({ isAttacking: true })

      const ctx = createMockContext({
        currentStep: Step.DECLARE_BLOCKERS,
        currentPlayerId: "player1",
        getOpponentOf: () => "player2",
        getBattlefieldCards: (playerId) =>
          playerId === "player2" ? [blocker] : [attacker],
        getCreatureState: (id) =>
          id === "blocker-1" ? blockerState : attackerState,
      })

      expect(() =>
        validateDeclareBlocker(ctx, "player2", "blocker-1", "attacker-1"),
      ).toThrow(/tapped/i)
    })

    test("throws if blocker is already blocking", () => {
      const blocker = createMockCreature("blocker-1")
      blocker.ownerId = "player2"
      const blockerState = createPermanentState({
        blockingCreatureId: "other-attacker",
      })
      const attacker = createMockCreature("attacker-1")
      const attackerState = createPermanentState({ isAttacking: true })

      const ctx = createMockContext({
        currentStep: Step.DECLARE_BLOCKERS,
        currentPlayerId: "player1",
        getOpponentOf: () => "player2",
        getBattlefieldCards: (playerId) =>
          playerId === "player2" ? [blocker] : [attacker],
        getCreatureState: (id) =>
          id === "blocker-1" ? blockerState : attackerState,
      })

      expect(() =>
        validateDeclareBlocker(ctx, "player2", "blocker-1", "attacker-1"),
      ).toThrow(/already blocking/)
    })

    test("throws if attacker is not attacking", () => {
      const blocker = createMockCreature("blocker-1")
      blocker.ownerId = "player2"
      const blockerState = createPermanentState()
      const attacker = createMockCreature("attacker-1")
      const attackerState = createPermanentState() // Not attacking

      const ctx = createMockContext({
        currentStep: Step.DECLARE_BLOCKERS,
        currentPlayerId: "player1",
        getOpponentOf: () => "player2",
        getBattlefieldCards: (playerId) =>
          playerId === "player2" ? [blocker] : [attacker],
        getCreatureState: (id) =>
          id === "blocker-1" ? blockerState : attackerState,
      })

      expect(() =>
        validateDeclareBlocker(ctx, "player2", "blocker-1", "attacker-1"),
      ).toThrow(/not attacking/i)
    })

    test("throws if attacker is already blocked", () => {
      const blocker = createMockCreature("blocker-1")
      blocker.ownerId = "player2"
      const blockerState = createPermanentState()
      const attacker = createMockCreature("attacker-1")
      const attackerState = createPermanentState({
        isAttacking: true,
        blockedBy: "other-blocker",
      })

      const ctx = createMockContext({
        currentStep: Step.DECLARE_BLOCKERS,
        currentPlayerId: "player1",
        getOpponentOf: () => "player2",
        getBattlefieldCards: (playerId) =>
          playerId === "player2" ? [blocker] : [attacker],
        getCreatureState: (id) =>
          id === "blocker-1" ? blockerState : attackerState,
      })

      expect(() =>
        validateDeclareBlocker(ctx, "player2", "blocker-1", "attacker-1"),
      ).toThrow(/already blocked/)
    })

    test("throws if attacker has Flying and blocker lacks Flying or Reach", () => {
      const blocker = createMockCreature("blocker-1")
      blocker.ownerId = "player2"
      const blockerState = createPermanentState()
      const attacker = createMockCreature("attacker-1", [
        StaticAbilities.FLYING,
      ])
      const attackerState = createPermanentState({ isAttacking: true })

      const ctx = createMockContext({
        currentStep: Step.DECLARE_BLOCKERS,
        currentPlayerId: "player1",
        getOpponentOf: () => "player2",
        getBattlefieldCards: (playerId) =>
          playerId === "player2" ? [blocker] : [attacker],
        getCreatureState: (id) =>
          id === "blocker-1" ? blockerState : attackerState,
        hasStaticAbility: (card, ability) =>
          card.instanceId === "attacker-1" &&
          ability === StaticAbilities.FLYING,
      })

      expect(() =>
        validateDeclareBlocker(ctx, "player2", "blocker-1", "attacker-1"),
      ).toThrow(/flying/i)
    })

    test("allows blocker with Reach to block flyer", () => {
      const blocker = createMockCreature("blocker-1", [StaticAbilities.REACH])
      blocker.ownerId = "player2"
      const blockerState = createPermanentState()
      const attacker = createMockCreature("attacker-1", [
        StaticAbilities.FLYING,
      ])
      const attackerState = createPermanentState({ isAttacking: true })

      const ctx = createMockContext({
        currentStep: Step.DECLARE_BLOCKERS,
        currentPlayerId: "player1",
        getOpponentOf: () => "player2",
        getBattlefieldCards: (playerId) =>
          playerId === "player2" ? [blocker] : [attacker],
        getCreatureState: (id) =>
          id === "blocker-1" ? blockerState : attackerState,
        hasStaticAbility: (card, ability) => {
          if (
            card.instanceId === "attacker-1" &&
            ability === StaticAbilities.FLYING
          )
            return true
          if (
            card.instanceId === "blocker-1" &&
            ability === StaticAbilities.REACH
          )
            return true
          return false
        },
      })

      const result = validateDeclareBlocker(
        ctx,
        "player2",
        "blocker-1",
        "attacker-1",
      )

      expect(result.newBlockerState.creatureState?.blockingCreatureId).toBe(
        "attacker-1",
      )
      expect(result.newAttackerState.creatureState?.blockedBy).toBe("blocker-1")
    })

    test("returns valid result with blocking relationship established", () => {
      const blocker = createMockCreature("blocker-1")
      blocker.ownerId = "player2"
      const blockerState = createPermanentState()
      const attacker = createMockCreature("attacker-1")
      const attackerState = createPermanentState({ isAttacking: true })

      const ctx = createMockContext({
        currentStep: Step.DECLARE_BLOCKERS,
        currentPlayerId: "player1",
        getOpponentOf: () => "player2",
        getBattlefieldCards: (playerId) =>
          playerId === "player2" ? [blocker] : [attacker],
        getCreatureState: (id) =>
          id === "blocker-1" ? blockerState : attackerState,
      })

      const result = validateDeclareBlocker(
        ctx,
        "player2",
        "blocker-1",
        "attacker-1",
      )

      expect(result.blocker).toBe(blocker)
      expect(result.attacker).toBe(attacker)
      expect(result.newBlockerState.creatureState?.blockingCreatureId).toBe(
        "attacker-1",
      )
      expect(result.newAttackerState.creatureState?.blockedBy).toBe("blocker-1")
    })
  })
})
