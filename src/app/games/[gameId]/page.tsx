"use client"

import dynamic from "next/dynamic"
import { use, useCallback, useEffect, useState } from "react"
import type { GameStateExport } from "@/echomancy/domainmodel/game/GameStateExport"
import {
  type CardRegistry,
  createGameSnapshot,
  type GameSnapshot,
} from "@/echomancy/infrastructure/ui/GameSnapshot"
import { GraveyardCount } from "./components/graveyard/GraveyardCount"
import { OpponentHandCount } from "./components/hand/OpponentHandCount"
import { formatPhaseAndStep } from "./formatters"

// Dynamic import of BattlefieldDisplay with ssr: false for PixiJS compatibility
const BattlefieldDisplay = dynamic(
  () =>
    import("./components/battlefield/BattlefieldDisplay").then(
      (mod) => mod.BattlefieldDisplay,
    ),
  { ssr: false },
)

type GamePageProps = {
  params: Promise<{ gameId: string }>
}

type ErrorState = { code: string; message: string } | null

// Simple in-memory CardRegistry that returns the cardDefinitionId as the name
const simpleCardRegistry: CardRegistry = {
  getCardName(cardDefinitionId: string): string {
    return cardDefinitionId
  },
}

export default function GamePage(props: GamePageProps) {
  const params = use(props.params)
  const { gameId } = params

  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<ErrorState>(null)
  const [snapshot, setSnapshot] = useState<GameSnapshot | null>(null)
  const [actionError, setActionError] = useState<ErrorState>(null)
  const [playableCardIds, setPlayableCardIds] = useState<readonly string[]>([])

  const fetchGameState = useCallback(async () => {
    setIsLoading(true)
    setError(null)

    try {
      const response = await fetch(`/api/games/${gameId}/state`)

      if (!response.ok) {
        if (response.status === 404) {
          setError({ code: "NOT_FOUND", message: "Game not found" })
          return
        }
        const errorData = await response.json()
        setError({
          code: "FETCH_FAILED",
          message: errorData.error?.message || "Failed to load game",
        })
        return
      }

      const data = await response.json()
      const state: GameStateExport = data.data

      // Create GameSnapshot for Player 1
      if (!state.turnOrder || state.turnOrder.length === 0) {
        setError({ code: "NO_PLAYERS", message: "No players in game" })
        return
      }

      const player1Id = state.turnOrder[0]
      const gameSnapshot = createGameSnapshot(
        state,
        player1Id,
        simpleCardRegistry,
      )
      setSnapshot(gameSnapshot)

      // Fetch allowed actions for the player
      const allowedActionsResponse = await fetch(
        `/api/games/${gameId}/allowed-actions?playerId=${player1Id}`,
      )

      if (allowedActionsResponse.ok) {
        const allowedActionsData = await allowedActionsResponse.json()
        setPlayableCardIds(allowedActionsData.data?.playableLands ?? [])
      } else {
        // If allowed actions fails, continue with empty playable lands
        setPlayableCardIds([])
      }
    } catch {
      setError({
        code: "NETWORK_ERROR",
        message: "Failed to load game",
      })
    } finally {
      setIsLoading(false)
    }
  }, [gameId])

  useEffect(() => {
    fetchGameState()
  }, [fetchGameState])

  if (isLoading) {
    return <div>Loading game...</div>
  }

  if (error) {
    return <div role="alert">Error: {error.message}</div>
  }

  if (!snapshot) {
    return <div>No game snapshot available</div>
  }

  return (
    <GameInfo
      snapshot={snapshot}
      gameId={gameId}
      actionError={actionError}
      setActionError={setActionError}
      refreshGameState={fetchGameState}
      playableCardIds={playableCardIds}
    />
  )
}

type GameInfoProps = {
  snapshot: GameSnapshot
  gameId: string
  actionError: ErrorState
  setActionError: (error: ErrorState) => void
  refreshGameState: () => Promise<void>
  playableCardIds: readonly string[]
}

function GameInfo({
  snapshot,
  gameId,
  actionError,
  setActionError,
  refreshGameState,
  playableCardIds,
}: GameInfoProps) {
  const { publicGameState, privatePlayerState, opponentStates } = snapshot

  // Format phase and step for display
  const phaseStepDisplay = formatPhaseAndStep(
    publicGameState.currentPhase,
    publicGameState.currentStep,
  )

  // Get opponent data (handle null case)
  const opponentLife = opponentStates[0]?.lifeTotal ?? null
  const opponentHandSize = opponentStates[0]?.handSize ?? 0
  const opponentGraveyardSize = opponentStates[0]?.graveyard.length ?? 0

  // Handle card click from hand - play land action
  const handleCardClick = useCallback(
    async (cardInstanceId: string) => {
      setActionError(null)

      try {
        const response = await fetch(`/api/games/${gameId}/actions`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            type: "PLAY_LAND",
            playerId: snapshot.viewerPlayerId,
            cardId: cardInstanceId,
          }),
        })

        if (!response.ok) {
          const errorData = await response.json()
          setActionError({
            code: errorData.error?.code || "ACTION_FAILED",
            message: errorData.error?.message || "Failed to play land",
          })
          return
        }

        // Refresh game state after successful action
        await refreshGameState()
      } catch {
        setActionError({
          code: "NETWORK_ERROR",
          message: "Failed to play land",
        })
      }
    },
    [gameId, snapshot.viewerPlayerId, setActionError, refreshGameState],
  )

  return (
    <div>
      {/* Action Error Display */}
      {actionError && (
        <div
          role="alert"
          style={{
            padding: "12px",
            marginBottom: "16px",
            backgroundColor: "#fee",
            border: "1px solid #c33",
            borderRadius: "4px",
            color: "#c33",
          }}
        >
          <strong>Error:</strong> {actionError.message}
          <button
            type="button"
            onClick={() => setActionError(null)}
            style={{
              marginLeft: "12px",
              padding: "4px 8px",
              cursor: "pointer",
            }}
          >
            Dismiss
          </button>
        </div>
      )}

      <div>
        Turn {publicGameState.turnNumber} - {phaseStepDisplay}
      </div>

      <div>Your Life: {privatePlayerState.lifeTotal}</div>
      {opponentLife !== null && <div>Opponent Life: {opponentLife}</div>}

      {/* Opponent Hand Count */}
      <OpponentHandCount count={opponentHandSize} />

      {/* Graveyard Counts */}
      <GraveyardCount
        count={privatePlayerState.graveyard.length}
        label="Your Graveyard"
      />
      <GraveyardCount
        count={opponentGraveyardSize}
        label="Opponent Graveyard"
      />

      {/* Battlefield and Hand Display */}
      <div style={{ marginTop: "20px" }}>
        <BattlefieldDisplay
          snapshot={snapshot}
          onHandCardClick={handleCardClick}
          playableCardIds={playableCardIds}
        />
      </div>
    </div>
  )
}
