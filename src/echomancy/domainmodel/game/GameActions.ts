import type { Target } from "../targets/Target"

export type AdvanceStep = { type: "ADVANCE_STEP"; playerId: string }
export type EndTurn = { type: "END_TURN"; playerId: string }
export type PlayLand = { type: "PLAY_LAND"; playerId: string; cardId: string }
export type CastSpell = {
  type: "CAST_SPELL"
  playerId: string
  cardId: string
  targets: Target[]
}
export type PassPriority = { type: "PASS_PRIORITY"; playerId: string }
export type DeclareAttacker = {
  type: "DECLARE_ATTACKER"
  playerId: string
  creatureId: string
}
export type ActivateAbility = {
  type: "ACTIVATE_ABILITY"
  playerId: string
  permanentId: string
}
export type DeclareBlocker = {
  type: "DECLARE_BLOCKER"
  playerId: string
  blockerId: string
  attackerId: string
}
export type DrawCard = {
  type: "DRAW_CARD"
  playerId: string
  amount: number
}

export type Actions =
  | AdvanceStep
  | EndTurn
  | PlayLand
  | CastSpell
  | PassPriority
  | DeclareAttacker
  | DeclareBlocker
  | ActivateAbility
  | DrawCard

export type AllowedAction = Actions["type"]
