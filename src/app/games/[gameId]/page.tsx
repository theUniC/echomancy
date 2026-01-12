"use client"

import dynamic from "next/dynamic"
import { use, useEffect, useState } from "react"
import type { GameStateExport } from "@/echomancy/domainmodel/game/GameStateExport"
import {
  type CardRegistry,
  createGameSnapshot,
  type GameSnapshot,
} from "@/echomancy/infrastructure/ui/GameSnapshot"
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

  useEffect(() => {
    const fetchGameState = async () => {
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
      } catch {
        setError({
          code: "NETWORK_ERROR",
          message: "Failed to load game",
        })
      } finally {
        setIsLoading(false)
      }
    }

    fetchGameState()
  }, [gameId])

  if (isLoading) {
    return <div>Loading game...</div>
  }

  if (error) {
    return <div role="alert">Error: {error.message}</div>
  }

  if (!snapshot) {
    return <div>No game snapshot available</div>
  }

  return <GameInfo snapshot={snapshot} />
}

type GameInfoProps = {
  snapshot: GameSnapshot
}

function GameInfo({ snapshot }: GameInfoProps) {
  const { publicGameState, privatePlayerState, opponentStates } = snapshot

  // Format phase and step for display
  const phaseStepDisplay = formatPhaseAndStep(
    publicGameState.currentPhase,
    publicGameState.currentStep,
  )

  // Get opponent life total (handle null case)
  const opponentLife = opponentStates[0]?.lifeTotal ?? null

  return (
    <div>
      <div>
        Turn {publicGameState.turnNumber} - {phaseStepDisplay}
      </div>

      <div>Your Life: {privatePlayerState.lifeTotal}</div>
      {opponentLife !== null && <div>Opponent Life: {opponentLife}</div>}

      {/* Battlefield Display */}
      <div style={{ marginTop: "20px" }}>
        <BattlefieldDisplay snapshot={snapshot} />
      </div>
    </div>
  )
}
