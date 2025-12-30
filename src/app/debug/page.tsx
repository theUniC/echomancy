"use client"

import { useState } from "react"

type GameState = Record<string, unknown> | null
type ErrorState = { code: string; message: string } | null

export default function DebugPage() {
  const [gameId, setGameId] = useState<string | null>(null)
  const [player1Id, setPlayer1Id] = useState<string | null>(null)
  const [player2Id, setPlayer2Id] = useState<string | null>(null)
  const [gameState, setGameState] = useState<GameState>(null)
  const [error, setError] = useState<ErrorState>(null)
  const [actionInput, setActionInput] = useState("")
  const [isLoading, setIsLoading] = useState(false)

  const createGame = async () => {
    setIsLoading(true)
    setError(null)
    try {
      const newGameId = crypto.randomUUID()
      const newPlayer1Id = crypto.randomUUID()
      const newPlayer2Id = crypto.randomUUID()

      // Create game
      const createResponse = await fetch("/api/games", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ gameId: newGameId }),
      })
      if (!createResponse.ok) {
        const errorData = await createResponse.json()
        throw new Error(
          `Failed to create game: ${errorData.error?.message || createResponse.statusText}`,
        )
      }

      // Join player 1
      const player1Response = await fetch(`/api/games/${newGameId}/players`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          playerId: newPlayer1Id,
          playerName: "Player 1",
        }),
      })
      if (!player1Response.ok) {
        const errorData = await player1Response.json()
        throw new Error(
          `Failed to add Player 1: ${errorData.error?.message || player1Response.statusText}`,
        )
      }

      // Join player 2
      const player2Response = await fetch(`/api/games/${newGameId}/players`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          playerId: newPlayer2Id,
          playerName: "Player 2",
        }),
      })
      if (!player2Response.ok) {
        const errorData = await player2Response.json()
        throw new Error(
          `Failed to add Player 2: ${errorData.error?.message || player2Response.statusText}`,
        )
      }

      // Start game
      const startResponse = await fetch(`/api/games/${newGameId}/start`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ startingPlayerId: newPlayer1Id }),
      })
      if (!startResponse.ok) {
        const errorData = await startResponse.json()
        throw new Error(
          `Failed to start game: ${errorData.error?.message || startResponse.statusText}`,
        )
      }

      // Get initial state
      const stateResponse = await fetch(`/api/games/${newGameId}/state`)
      if (!stateResponse.ok) {
        const errorData = await stateResponse.json()
        throw new Error(
          `Failed to fetch state: ${errorData.error?.message || stateResponse.statusText}`,
        )
      }
      const stateData = await stateResponse.json()

      setGameId(newGameId)
      setPlayer1Id(newPlayer1Id)
      setPlayer2Id(newPlayer2Id)
      setGameState(stateData.data)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      setError({ code: "CREATE_FAILED", message: errorMessage })
    } finally {
      setIsLoading(false)
    }
  }

  const submitAction = async () => {
    if (!gameId) return

    setIsLoading(true)
    setError(null)
    try {
      // Parse JSON from textarea
      let parsedAction: unknown
      try {
        parsedAction = JSON.parse(actionInput)
      } catch (parseError) {
        throw new Error(
          `Invalid JSON: ${parseError instanceof Error ? parseError.message : String(parseError)}`,
        )
      }

      // POST to actions endpoint
      const actionResponse = await fetch(`/api/games/${gameId}/actions`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(parsedAction),
      })

      if (!actionResponse.ok) {
        const errorData = await actionResponse.json()
        throw new Error(errorData.error?.message || actionResponse.statusText)
      }

      const actionData = await actionResponse.json()

      // Update game state with response
      setGameState(actionData.data.state)
      setActionInput("") // Clear input on success
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      setError({ code: "ACTION_FAILED", message: errorMessage })
      // Don't clear existing state on error
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div
      style={{
        padding: "20px",
        fontFamily: "monospace",
        maxWidth: "1200px",
        margin: "0 auto",
      }}
    >
      <h1>Echomancy Debug Console</h1>

      {/* Create Game Section */}
      <section
        style={{
          marginBottom: "30px",
          padding: "15px",
          border: "1px solid #ccc",
          borderRadius: "5px",
        }}
      >
        <h2>1. Create New Game</h2>
        <button
          type="button"
          onClick={createGame}
          disabled={isLoading}
          style={{
            padding: "10px 20px",
            fontSize: "16px",
            cursor: isLoading ? "not-allowed" : "pointer",
            backgroundColor: isLoading ? "#ccc" : "#007bff",
            color: "white",
            border: "none",
            borderRadius: "5px",
          }}
        >
          {isLoading ? "Creating..." : "Create New Game"}
        </button>

        {gameId && (
          <div
            style={{
              marginTop: "15px",
              padding: "15px",
              background: "#f0f0f0",
              borderRadius: "5px",
              fontSize: "14px",
            }}
          >
            <div style={{ marginBottom: "8px" }}>
              <strong>Game ID:</strong>{" "}
              <code
                style={{
                  background: "#fff",
                  padding: "2px 6px",
                  borderRadius: "3px",
                }}
              >
                {gameId}
              </code>
            </div>
            <div style={{ marginBottom: "8px" }}>
              <strong>Player 1 ID:</strong>{" "}
              <code
                style={{
                  background: "#fff",
                  padding: "2px 6px",
                  borderRadius: "3px",
                }}
              >
                {player1Id}
              </code>
            </div>
            <div>
              <strong>Player 2 ID:</strong>{" "}
              <code
                style={{
                  background: "#fff",
                  padding: "2px 6px",
                  borderRadius: "3px",
                }}
              >
                {player2Id}
              </code>
            </div>
          </div>
        )}
      </section>

      {/* Action Input Section */}
      <section
        style={{
          marginBottom: "30px",
          padding: "15px",
          border: "1px solid #ccc",
          borderRadius: "5px",
        }}
      >
        <h2>2. Submit Action</h2>
        <textarea
          value={actionInput}
          onChange={(e) => setActionInput(e.target.value)}
          placeholder='{"type": "ADVANCE_STEP", "playerId": "..."}'
          style={{
            width: "100%",
            height: "120px",
            fontFamily: "monospace",
            fontSize: "14px",
            padding: "10px",
            border: "1px solid #ccc",
            borderRadius: "5px",
            resize: "vertical",
          }}
          disabled={!gameId}
        />
        <button
          type="button"
          onClick={submitAction}
          disabled={!gameId || isLoading || !actionInput.trim()}
          style={{
            marginTop: "10px",
            padding: "10px 20px",
            fontSize: "16px",
            cursor:
              !gameId || isLoading || !actionInput.trim()
                ? "not-allowed"
                : "pointer",
            backgroundColor:
              !gameId || isLoading || !actionInput.trim() ? "#ccc" : "#28a745",
            color: "white",
            border: "none",
            borderRadius: "5px",
          }}
        >
          {isLoading ? "Submitting..." : "Submit Action"}
        </button>
      </section>

      {/* Error Display */}
      {error && (
        <div
          role="alert"
          style={{
            marginBottom: "30px",
            padding: "15px",
            background: "#ffcccc",
            color: "#cc0000",
            borderRadius: "5px",
            border: "1px solid #cc0000",
          }}
        >
          <strong>Error:</strong> [{error.code}] {error.message}
        </div>
      )}

      {/* Game State Display */}
      {gameState && (
        <section
          style={{
            padding: "15px",
            border: "1px solid #ccc",
            borderRadius: "5px",
          }}
        >
          <h2>3. Current Game State</h2>
          <pre
            style={{
              background: "#f5f5f5",
              padding: "15px",
              overflow: "auto",
              borderRadius: "5px",
              fontSize: "12px",
              maxHeight: "600px",
            }}
          >
            {JSON.stringify(gameState, null, 2)}
          </pre>
        </section>
      )}
    </div>
  )
}
