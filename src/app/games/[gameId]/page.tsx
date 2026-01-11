"use client"

import { use, useEffect, useState } from "react"
import type { GameStateExport } from "@/echomancy/domainmodel/game/GameStateExport"
import {
  type CardRegistry,
  createGameSnapshot,
  type GameSnapshot,
} from "@/echomancy/infrastructure/ui/GameSnapshot"

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

  return <div>Game snapshot loaded successfully for Player 1</div>
}
