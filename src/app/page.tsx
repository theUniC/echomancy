"use client"

import { useState } from "react"
import { Game } from "@/echomancy/domainmodel/game/Game"
import { Player } from "@/echomancy/domainmodel/game/Player"

export default function DebugConsole() {
  const [game, setGame] = useState<Game | null>(null)
  const [player1Id, setPlayer1Id] = useState<string>("")
  const [player2Id, setPlayer2Id] = useState<string>("")
  const [actionInput, setActionInput] = useState("")
  const [gameState, setGameState] = useState<string>("")
  const [error, setError] = useState<string>("")
  const [actionLog, setActionLog] = useState<string[]>([])

  const createNewGame = () => {
    try {
      const p1 = new Player("p1", "Player 1")
      const p2 = new Player("p2", "Player 2")

      const newGame = Game.create("debug-game")
      newGame.addPlayer(p1)
      newGame.addPlayer(p2)
      newGame.start(p1.id)

      setGame(newGame)
      setPlayer1Id(p1.id)
      setPlayer2Id(p2.id)
      setError("")
      setActionLog(["Game created. Player 1 goes first."])
      updateGameState(newGame)
    } catch (e) {
      setError(e instanceof Error ? e.message : "Unknown error")
    }
  }

  const updateGameState = (g: Game) => {
    const exported = g.exportState()
    setGameState(JSON.stringify(exported, null, 2))
  }

  const submitAction = () => {
    if (!game) {
      setError("No game created. Click 'New Game' first.")
      return
    }

    try {
      const action = JSON.parse(actionInput)
      game.apply(action)
      setError("")
      setActionLog((prev) => [...prev, `Applied: ${JSON.stringify(action)}`])
      updateGameState(game)
      setActionInput("")
    } catch (e) {
      setError(e instanceof Error ? e.message : "Unknown error")
    }
  }

  const exampleActions = [
    {
      label: "Advance Step",
      action: { type: "ADVANCE_STEP", playerId: "p1" },
    },
    {
      label: "Pass Priority",
      action: { type: "PASS_PRIORITY", playerId: "p1" },
    },
    {
      label: "End Turn",
      action: { type: "END_TURN", playerId: "p1" },
    },
  ]

  return (
    <div className="min-h-screen bg-zinc-900 text-zinc-100 p-8 font-mono">
      <h1 className="text-2xl font-bold mb-6">Echomancy Debug Console</h1>

      <div className="mb-6">
        <button
          type="button"
          onClick={createNewGame}
          className="bg-green-600 hover:bg-green-700 px-4 py-2 rounded font-semibold"
        >
          New Game
        </button>
        {game && (
          <span className="ml-4 text-zinc-400">
            Player 1: {player1Id} | Player 2: {player2Id}
          </span>
        )}
      </div>

      <div className="grid grid-cols-2 gap-8">
        <div>
          <h2 className="text-lg font-semibold mb-2">Action Input (JSON)</h2>
          <textarea
            value={actionInput}
            onChange={(e) => setActionInput(e.target.value)}
            placeholder='{"type": "ADVANCE_STEP", "playerId": "p1"}'
            className="w-full h-32 bg-zinc-800 border border-zinc-700 rounded p-3 text-sm"
          />
          <button
            type="button"
            onClick={submitAction}
            className="mt-2 bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded font-semibold"
          >
            Submit Action
          </button>

          {error && (
            <div className="mt-4 p-3 bg-red-900/50 border border-red-700 rounded text-red-200">
              <strong>Error:</strong> {error}
            </div>
          )}

          <div className="mt-6">
            <h3 className="text-sm font-semibold mb-2 text-zinc-400">
              Quick Actions (click to copy)
            </h3>
            <div className="flex flex-wrap gap-2">
              {exampleActions.map((ex) => (
                <button
                  type="button"
                  key={ex.label}
                  onClick={() => setActionInput(JSON.stringify(ex.action))}
                  className="text-xs bg-zinc-700 hover:bg-zinc-600 px-2 py-1 rounded"
                >
                  {ex.label}
                </button>
              ))}
            </div>
          </div>

          <div className="mt-6">
            <h3 className="text-sm font-semibold mb-2 text-zinc-400">
              Action Log
            </h3>
            <div className="bg-zinc-800 border border-zinc-700 rounded p-3 h-40 overflow-y-auto text-xs">
              {actionLog.map((log) => (
                <div key={log} className="text-zinc-300">
                  {log}
                </div>
              ))}
            </div>
          </div>
        </div>

        <div>
          <h2 className="text-lg font-semibold mb-2">Game State</h2>
          <pre className="bg-zinc-800 border border-zinc-700 rounded p-3 h-[600px] overflow-auto text-xs">
            {gameState || "No game created yet."}
          </pre>
        </div>
      </div>
    </div>
  )
}
