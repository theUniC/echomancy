/**
 * GameStateExporter Service
 *
 * Extracts state export/serialization logic from Game.ts.
 * Converts internal game state to the GameStateExport format.
 */

import type { CardInstance } from "../../cards/CardInstance"
import type { Zone } from "../../zones/Zone"
import type {
  CardInstanceExport,
  CreatureStateExport,
  GameStateExport,
  ManaPoolExport,
  PlayerStateExport,
  StackItemExport,
  ZoneExport,
} from "../GameStateExport"
import type { Player } from "../Player"
import type { PlayerState } from "../PlayerState"
import type { StackItem } from "../StackTypes"
import type { GameSteps } from "../Steps"
import type { CreatureState as CreatureStateVO } from "../valueobjects/CreatureState"
import type { ManaPool } from "../valueobjects/ManaPool"

/**
 * Read-only context interface for exporting game state.
 * Game implements this interface to provide export access.
 */
export type ExportableGameContext = {
  readonly id: string
  readonly currentPlayerId: string
  readonly currentStep: GameSteps
  readonly priorityPlayerId: string | null
  readonly turnNumber: number
  readonly playedLands: number
  readonly turnOrder: readonly string[]
  readonly scheduledSteps: readonly GameSteps[]
  readonly resumeStepAfterScheduled: GameSteps | undefined

  getPlayer(playerId: string): Player | undefined
  getPlayerState(playerId: string): PlayerState
  getManaPool(playerId: string): ManaPool
  getCreatureState(instanceId: string): CreatureStateVO | undefined
  getStackItems(): readonly StackItem[]
  isPlaneswalker(card: CardInstance): boolean
  findCardOnBattlefields(instanceId: string): CardInstance | undefined
}

/**
 * Exports the complete game state to a serializable format.
 */
export function exportGameState(ctx: ExportableGameContext): GameStateExport {
  const playersExport: Record<string, PlayerStateExport> = {}

  for (const playerId of ctx.turnOrder) {
    const player = ctx.getPlayer(playerId)
    if (!player) {
      throw new Error(`Player not found: ${playerId}`)
    }

    const playerState = ctx.getPlayerState(playerId)
    const manaPool = ctx.getManaPool(playerId)

    playersExport[playerId] = {
      lifeTotal: player.lifeTotal,
      manaPool: exportManaPool(manaPool),
      playedLandsThisTurn:
        playerId === ctx.currentPlayerId ? ctx.playedLands : 0,
      zones: {
        hand: exportZone(ctx, playerState.hand, playerId),
        battlefield: exportZone(ctx, playerState.battlefield, playerId),
        graveyard: exportZone(ctx, playerState.graveyard, playerId),
      },
    }
  }

  return {
    gameId: ctx.id,
    currentTurnNumber: ctx.turnNumber,
    currentPlayerId: ctx.currentPlayerId,
    currentStep: ctx.currentStep,
    priorityPlayerId: ctx.priorityPlayerId,
    turnOrder: [...ctx.turnOrder],
    players: playersExport,
    stack: ctx.getStackItems().map((item) => exportStackItem(ctx, item)),
    scheduledSteps: [...ctx.scheduledSteps],
    resumeStepAfterScheduled: ctx.resumeStepAfterScheduled,
  }
}

function exportManaPool(manaPool: ManaPool): ManaPoolExport {
  return manaPool.toSnapshot()
}

function exportZone(
  ctx: ExportableGameContext,
  zone: Zone,
  controllerId: string,
): ZoneExport {
  return {
    cards: zone.cards.map((card) =>
      exportCardInstance(ctx, card, controllerId),
    ),
  }
}

function exportCardInstance(
  ctx: ExportableGameContext,
  card: CardInstance,
  controllerId: string,
): CardInstanceExport {
  const def = card.definition
  const exported: CardInstanceExport = {
    instanceId: card.instanceId,
    ownerId: card.ownerId,
    controllerId: controllerId,
    cardDefinitionId: def.id,
    types: def.types,
  }

  if (def.staticAbilities && def.staticAbilities.length > 0) {
    exported.staticAbilities = def.staticAbilities
  }

  if (def.power !== undefined) {
    exported.power = def.power
  }
  if (def.toughness !== undefined) {
    exported.toughness = def.toughness
  }

  const creatureState = ctx.getCreatureState(card.instanceId)
  if (creatureState) {
    exported.creatureState = exportCreatureState(creatureState)
  }

  if (ctx.isPlaneswalker(card)) {
    exported.planeswalkerState = {}
  }

  return exported
}

function exportCreatureState(state: CreatureStateVO): CreatureStateExport {
  const snapshot = state.toExport()

  return {
    isTapped: snapshot.isTapped,
    isAttacking: snapshot.isAttacking,
    hasAttackedThisTurn: snapshot.hasAttackedThisTurn,
    hasSummoningSickness: snapshot.hasSummoningSickness,
    power: state.getCurrentPower(),
    toughness: state.getCurrentToughness(),
    counters: snapshot.counters,
    damageMarkedThisTurn: snapshot.damageMarkedThisTurn,
    blockingCreatureId: snapshot.blockingCreatureId,
    blockedBy: snapshot.blockedBy,
  }
}

function exportStackItem(
  ctx: ExportableGameContext,
  item: StackItem,
): StackItemExport {
  if (item.kind === "SPELL") {
    return {
      kind: "SPELL",
      sourceCardInstanceId: item.card.instanceId,
      sourceCardDefinitionId: item.card.definition.id,
      controllerId: item.controllerId,
      targets: item.targets.map((t) => t.playerId),
    }
  }

  const sourceCard = ctx.findCardOnBattlefields(item.sourceId)

  return {
    kind: "ACTIVATED_ABILITY",
    sourceCardInstanceId: item.sourceId,
    sourceCardDefinitionId: sourceCard?.definition.id ?? "unknown",
    controllerId: item.controllerId,
    targets: item.targets.map((t) => t.playerId),
  }
}
