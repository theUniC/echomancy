/**
 * @vitest-environment jsdom
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { beforeEach, describe, expect, it, vi } from "vitest"
import DebugPage from "../page"

// Mock crypto.randomUUID
const mockUUID1 = "game-uuid-1"
const mockUUID2 = "player1-uuid"
const mockUUID3 = "player2-uuid"
let uuidCallCount = 0

const mockRandomUUID = vi.fn(() => {
  uuidCallCount++
  if (uuidCallCount === 1) return mockUUID1
  if (uuidCallCount === 2) return mockUUID2
  if (uuidCallCount === 3) return mockUUID3
  return `uuid-${uuidCallCount}`
})

// Mock crypto globally
Object.defineProperty(global, "crypto", {
  value: {
    randomUUID: mockRandomUUID,
  },
  writable: true,
})

// Mock fetch
global.fetch = vi.fn()

const mockGameState = {
  gameId: mockUUID1,
  currentTurnNumber: 1,
  currentPlayerId: mockUUID2,
  priorityPlayerId: mockUUID2,
  currentStep: "UNTAP",
  turnOrder: [mockUUID2, mockUUID3],
  stack: [],
  players: {
    [mockUUID2]: {
      lifeTotal: 20,
      manaPool: { W: 0, U: 0, B: 0, R: 0, G: 0, C: 0 },
      playedLandsThisTurn: 0,
      zones: {
        hand: { cards: [] },
        battlefield: { cards: [] },
        graveyard: { cards: [] },
      },
    },
    [mockUUID3]: {
      lifeTotal: 20,
      manaPool: { W: 0, U: 0, B: 0, R: 0, G: 0, C: 0 },
      playedLandsThisTurn: 0,
      zones: {
        hand: { cards: [] },
        battlefield: { cards: [] },
        graveyard: { cards: [] },
      },
    },
  },
  scheduledSteps: [],
}

const mockGameList = [
  {
    gameId: mockUUID1,
    status: "in_progress" as const,
    playerNames: ["Player 1", "Player 2"],
    turnNumber: 1,
    currentPhase: "UNTAP",
  },
  {
    gameId: "other-game-uuid-2",
    status: "not_started" as const,
    playerNames: ["Alice", "Bob"],
    turnNumber: null,
    currentPhase: null,
  },
]

// Helper to type JSON into textarea (avoids issues with user-event parsing braces)
const typeJSON = (element: Element, json: string) => {
  fireEvent.change(element, { target: { value: json } })
}

describe("DebugPage", () => {
  beforeEach(() => {
    uuidCallCount = 0
    vi.clearAllMocks()
  })

  describe("rendering", () => {
    it("displays the debug console title", () => {
      const mockFetch = fetch as ReturnType<typeof vi.fn>
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      render(<DebugPage />)

      expect(screen.getByText("Echomancy Debug Console")).toBeInTheDocument()
    })

    it("displays create game button initially", () => {
      const mockFetch = fetch as ReturnType<typeof vi.fn>
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      render(<DebugPage />)

      expect(
        screen.getByRole("button", { name: /create new game/i }),
      ).toBeInTheDocument()
    })

    it("disables action submission initially", () => {
      const mockFetch = fetch as ReturnType<typeof vi.fn>
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      render(<DebugPage />)

      const textarea = screen.getByPlaceholderText(/ADVANCE_STEP/i)
      const submitButton = screen.getByRole("button", {
        name: /submit action/i,
      })

      expect(textarea).toBeDisabled()
      expect(submitButton).toBeDisabled()
    })

    it("shows placeholder text in action input", () => {
      const mockFetch = fetch as ReturnType<typeof vi.fn>
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      render(<DebugPage />)

      expect(
        screen.getByPlaceholderText(
          '{"type": "ADVANCE_STEP", "playerId": "..."}',
        ),
      ).toBeInTheDocument()
    })
  })

  describe("listing games", () => {
    it("fetches and displays list of games on mount", async () => {
      const mockFetch = fetch as ReturnType<typeof vi.fn>
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameList }),
      } as Response)

      render(<DebugPage />)

      // Game IDs are truncated to first 8 chars in the UI
      await waitFor(() => {
        expect(screen.getByText(/game-uui/)).toBeInTheDocument()
      })

      expect(screen.getByText(/Player 1, Player 2/)).toBeInTheDocument()
      expect(screen.getByText(/Alice, Bob/)).toBeInTheDocument()
    })

    it("shows loading state while fetching games", () => {
      const mockFetch = fetch as ReturnType<typeof vi.fn>
      let resolveGames: (value: unknown) => void
      const gamesPromise = new Promise((resolve) => {
        resolveGames = resolve
      })
      mockFetch.mockReturnValueOnce(gamesPromise as Promise<Response>)

      render(<DebugPage />)

      expect(screen.getByText(/loading games/i)).toBeInTheDocument()

      resolveGames?.({
        ok: true,
        json: async () => ({ data: [] }),
      })
    })

    it("shows message when no games exist", async () => {
      const mockFetch = fetch as ReturnType<typeof vi.fn>
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      render(<DebugPage />)

      await waitFor(() => {
        expect(
          screen.getByText(/no games found. create a new game below./i),
        ).toBeInTheDocument()
      })
    })

    it("displays error when fetching games fails", async () => {
      const mockFetch = fetch as ReturnType<typeof vi.fn>
      mockFetch.mockResolvedValueOnce({
        ok: false,
        statusText: "Internal Server Error",
        json: async () => ({
          error: { code: "FETCH_ERROR", message: "Database unavailable" },
        }),
      } as Response)

      render(<DebugPage />)

      await waitFor(() => {
        expect(screen.getByRole("alert")).toBeInTheDocument()
      })

      expect(screen.getByText(/FETCH_GAMES_FAILED/i)).toBeInTheDocument()
      expect(screen.getByText(/Database unavailable/i)).toBeInTheDocument()
    })

    it("displays game status and metadata correctly", async () => {
      const mockFetch = fetch as ReturnType<typeof vi.fn>
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameList }),
      } as Response)

      render(<DebugPage />)

      // Wait for game list to load, then check status values are displayed
      await waitFor(() => {
        expect(screen.getByText(/in_progress/)).toBeInTheDocument()
      })

      // Check metadata values are visible (the labels are in <strong> tags)
      expect(screen.getByText(/not_started/)).toBeInTheDocument()
    })
  })

  describe("loading a game", () => {
    it("loads game state when clicking on a game", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      // Mock initial game list fetch
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameList }),
      } as Response)

      render(<DebugPage />)

      await waitFor(() => {
        expect(screen.getByText(/game-uui/)).toBeInTheDocument()
      })

      // Mock game state fetch
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameState }),
      } as Response)

      const gameButton = screen.getByRole("button", { name: /game-uui/i })
      await user.click(gameButton)

      await waitFor(() => {
        expect(screen.getByText(/"currentTurnNumber": 1/)).toBeInTheDocument()
      })
    })

    it("displays player IDs after loading game", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameList }),
      } as Response)

      render(<DebugPage />)

      await waitFor(() => {
        expect(screen.getByText(/game-uui/)).toBeInTheDocument()
      })

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameState }),
      } as Response)

      await user.click(screen.getByRole("button", { name: /game-uui/i }))

      await waitFor(() => {
        expect(screen.getByText(/Player 1 ID:/i)).toBeInTheDocument()
      })

      expect(screen.getByText(mockUUID2)).toBeInTheDocument()
      expect(screen.getByText(mockUUID3)).toBeInTheDocument()
    })

    it("highlights selected game", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameList }),
      } as Response)

      render(<DebugPage />)

      await waitFor(() => {
        expect(screen.getByText(/game-uui/)).toBeInTheDocument()
      })

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameState }),
      } as Response)

      const gameButton = screen.getByRole("button", { name: /game-uui/i })
      await user.click(gameButton)

      await waitFor(() => {
        expect(gameButton).toHaveStyle({ backgroundColor: "#e6f3ff" })
      })
    })

    it("displays error when loading game fails", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameList }),
      } as Response)

      render(<DebugPage />)

      await waitFor(() => {
        expect(screen.getByText(/game-uui/)).toBeInTheDocument()
      })

      mockFetch.mockResolvedValueOnce({
        ok: false,
        statusText: "Not Found",
        json: async () => ({
          error: { code: "GAME_NOT_FOUND", message: "Game not found" },
        }),
      } as Response)

      await user.click(screen.getByRole("button", { name: /game-uui/i }))

      await waitFor(() => {
        expect(screen.getByRole("alert")).toBeInTheDocument()
      })

      expect(screen.getByText(/LOAD_GAME_FAILED/i)).toBeInTheDocument()
      expect(screen.getByText(/Game not found/i)).toBeInTheDocument()
    })
  })

  describe("creating a game", () => {
    it("creates game with correct API sequence", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      // Mock initial game list fetch
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response) // GET /api/games

      render(<DebugPage />)

      await waitFor(() => {
        expect(
          screen.getByText(/no games found. create a new game below./i),
        ).toBeInTheDocument()
      })

      // Mock all API calls to succeed
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: { gameId: mockUUID1 } }),
      } as Response) // POST /api/games
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response) // POST /api/games/:id/players (player 1)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response) // POST /api/games/:id/players (player 2)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response) // POST /api/games/:id/start
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameState }),
      } as Response) // GET /api/games/:id/state
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [mockGameList[0]] }),
      } as Response) // GET /api/games (refresh after create)

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledTimes(7) // Initial fetch + 5 create calls + refresh
      })

      // Verify API call sequence (after initial fetch)
      expect(mockFetch).toHaveBeenNthCalledWith(2, "/api/games", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ gameId: mockUUID1 }),
      })

      expect(mockFetch).toHaveBeenNthCalledWith(
        3,
        `/api/games/${mockUUID1}/players`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            playerId: mockUUID2,
            playerName: "Player 1",
          }),
        },
      )

      expect(mockFetch).toHaveBeenNthCalledWith(
        4,
        `/api/games/${mockUUID1}/players`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            playerId: mockUUID3,
            playerName: "Player 2",
          }),
        },
      )

      expect(mockFetch).toHaveBeenNthCalledWith(
        5,
        `/api/games/${mockUUID1}/start`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ startingPlayerId: mockUUID2 }),
        },
      )

      expect(mockFetch).toHaveBeenNthCalledWith(
        6,
        `/api/games/${mockUUID1}/state`,
      )

      // Verify refresh call
      expect(mockFetch).toHaveBeenNthCalledWith(7, "/api/games")
    })

    it("displays game IDs after successful creation", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      // Mock initial game list fetch
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      // Mock successful responses
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameState }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [mockGameList[0]] }),
      } as Response)

      render(<DebugPage />)

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        expect(screen.getByText(/Game ID:/i)).toBeInTheDocument()
      })

      expect(screen.getByText(mockUUID1)).toBeInTheDocument()
      expect(screen.getByText(/Player 1 ID:/i)).toBeInTheDocument()
      expect(screen.getByText(mockUUID2)).toBeInTheDocument()
      expect(screen.getByText(/Player 2 ID:/i)).toBeInTheDocument()
      expect(screen.getByText(mockUUID3)).toBeInTheDocument()
    })

    it("displays game state after successful creation", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      // Mock initial game list fetch
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      // Mock successful responses
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameState }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [mockGameList[0]] }),
      } as Response)

      render(<DebugPage />)

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        expect(screen.getByText(/Current Game State/i)).toBeInTheDocument()
      })

      // Check that game state JSON is displayed
      expect(screen.getByText(/"currentTurnNumber": 1/)).toBeInTheDocument()
    })

    it("enables action input after game creation", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      // Mock initial game list fetch
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      // Mock successful responses
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameState }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [mockGameList[0]] }),
      } as Response)

      render(<DebugPage />)

      const textarea = screen.getByPlaceholderText(/ADVANCE_STEP/i)

      expect(textarea).toBeDisabled()

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        expect(textarea).not.toBeDisabled()
      })
    })

    it("displays error when game creation fails", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      // Mock initial game list fetch
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      mockFetch.mockResolvedValueOnce({
        ok: false,
        statusText: "Internal Server Error",
        json: async () => ({
          error: { code: "CREATE_FAILED", message: "Database error" },
        }),
      } as Response)

      render(<DebugPage />)

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        expect(screen.getByRole("alert")).toBeInTheDocument()
      })

      expect(screen.getByText(/CREATE_FAILED/i)).toBeInTheDocument()
      expect(screen.getByText(/Database error/i)).toBeInTheDocument()
    })

    it("displays loading state during game creation", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      // Mock initial game list fetch
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      // Create a promise we can control
      let resolveCreate: (value: unknown) => void
      const createPromise = new Promise((resolve) => {
        resolveCreate = resolve
      })

      render(<DebugPage />)

      await waitFor(() => {
        expect(
          screen.getByText(/no games found. create a new game below./i),
        ).toBeInTheDocument()
      })

      mockFetch.mockReturnValueOnce(createPromise as Promise<Response>)

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      expect(screen.getByRole("button", { name: /creating/i })).toBeDisabled()

      // Resolve the promise
      resolveCreate?.({
        ok: true,
        json: async () => ({ data: {} }),
      })
    })
  })

  describe("submitting actions", () => {
    const setupGameCreated = async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      // Mock initial game list fetch
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      // Mock successful game creation
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameState }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [mockGameList[0]] }),
      } as Response)

      render(<DebugPage />)

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        expect(screen.getByText(/Game ID:/i)).toBeInTheDocument()
      })

      return user
    }

    it("submits valid JSON action", async () => {
      const user = await setupGameCreated()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      const actionPayload = { type: "ADVANCE_STEP", playerId: mockUUID2 }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: { state: mockGameState } }),
      } as Response)

      const textarea = screen.getByPlaceholderText(/ADVANCE_STEP/i)
      const submitButton = screen.getByRole("button", {
        name: /submit action/i,
      })

      typeJSON(textarea, JSON.stringify(actionPayload))
      await user.click(submitButton)

      await waitFor(() => {
        expect(mockFetch).toHaveBeenLastCalledWith(
          `/api/games/${mockUUID1}/actions`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(actionPayload),
          },
        )
      })
    })

    it("updates game state after successful action", async () => {
      const user = await setupGameCreated()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      const updatedGameState = {
        ...mockGameState,
        currentStep: "UPKEEP",
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: { state: updatedGameState } }),
      } as Response)

      const textarea = screen.getByPlaceholderText(/ADVANCE_STEP/i)
      typeJSON(
        textarea,
        JSON.stringify({ type: "ADVANCE_STEP", playerId: mockUUID2 }),
      )
      await user.click(screen.getByRole("button", { name: /submit action/i }))

      await waitFor(() => {
        expect(screen.getByText(/"currentStep": "UPKEEP"/)).toBeInTheDocument()
      })
    })

    it("clears input after successful action", async () => {
      const user = await setupGameCreated()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: { state: mockGameState } }),
      } as Response)

      const textarea = screen.getByPlaceholderText(
        /ADVANCE_STEP/i,
      ) as HTMLTextAreaElement
      typeJSON(
        textarea,
        JSON.stringify({ type: "ADVANCE_STEP", playerId: mockUUID2 }),
      )
      await user.click(screen.getByRole("button", { name: /submit action/i }))

      await waitFor(() => {
        expect(textarea.value).toBe("")
      })
    })

    it("displays error for invalid JSON", async () => {
      const user = await setupGameCreated()

      const textarea = screen.getByPlaceholderText(/ADVANCE_STEP/i)
      typeJSON(textarea, "{invalid json}")
      await user.click(screen.getByRole("button", { name: /submit action/i }))

      await waitFor(() => {
        const alert = screen.getByRole("alert")
        expect(alert).toBeInTheDocument()
        expect(alert).toHaveTextContent(/Invalid JSON/)
      })
    })

    it("displays error when action fails", async () => {
      const user = await setupGameCreated()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      mockFetch.mockResolvedValueOnce({
        ok: false,
        statusText: "Unprocessable Entity",
        json: async () => ({
          error: {
            code: "INVALID_ACTION",
            message: "Cannot advance step",
          },
        }),
      } as Response)

      const textarea = screen.getByPlaceholderText(/ADVANCE_STEP/i)
      typeJSON(
        textarea,
        JSON.stringify({ type: "ADVANCE_STEP", playerId: mockUUID2 }),
      )
      await user.click(screen.getByRole("button", { name: /submit action/i }))

      await waitFor(() => {
        expect(screen.getByRole("alert")).toBeInTheDocument()
      })

      expect(screen.getByText(/ACTION_FAILED/i)).toBeInTheDocument()
      expect(screen.getByText(/Cannot advance step/i)).toBeInTheDocument()
    })

    it("preserves game state when action fails", async () => {
      const user = await setupGameCreated()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      // Capture initial state
      const initialStateText = screen.getByText(/"currentStep": "UNTAP"/)

      mockFetch.mockResolvedValueOnce({
        ok: false,
        statusText: "Unprocessable Entity",
        json: async () => ({
          error: {
            code: "INVALID_ACTION",
            message: "Cannot advance step",
          },
        }),
      } as Response)

      const textarea = screen.getByPlaceholderText(/ADVANCE_STEP/i)
      typeJSON(
        textarea,
        JSON.stringify({ type: "ADVANCE_STEP", playerId: mockUUID2 }),
      )
      await user.click(screen.getByRole("button", { name: /submit action/i }))

      await waitFor(() => {
        expect(screen.getByRole("alert")).toBeInTheDocument()
      })

      // State should still be visible
      expect(initialStateText).toBeInTheDocument()
    })

    it("disables submit button when input is empty", async () => {
      await setupGameCreated()

      const submitButton = screen.getByRole("button", {
        name: /submit action/i,
      })

      expect(submitButton).toBeDisabled()
    })

    it("enables submit button when input has text", async () => {
      const _user = await setupGameCreated()

      const textarea = screen.getByPlaceholderText(/ADVANCE_STEP/i)
      const submitButton = screen.getByRole("button", {
        name: /submit action/i,
      })

      typeJSON(textarea, '{"type": "ADVANCE_STEP"}')

      expect(submitButton).not.toBeDisabled()
    })
  })

  describe("error handling", () => {
    it("clears previous errors when creating new game", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      // Mock initial game list fetch
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      // First attempt fails
      mockFetch.mockResolvedValueOnce({
        ok: false,
        statusText: "Error",
        json: async () => ({
          error: { code: "ERROR", message: "First error" },
        }),
      } as Response)

      render(<DebugPage />)

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        expect(screen.getByText(/First error/i)).toBeInTheDocument()
      })

      // Second attempt succeeds
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: {} }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: mockGameState }),
      } as Response)
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [mockGameList[0]] }),
      } as Response)

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        expect(screen.queryByText(/First error/i)).not.toBeInTheDocument()
      })
    })

    it("displays error with alert role for accessibility", async () => {
      const user = userEvent.setup()
      const mockFetch = fetch as ReturnType<typeof vi.fn>

      // Mock initial game list fetch
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: [] }),
      } as Response)

      mockFetch.mockResolvedValueOnce({
        ok: false,
        statusText: "Error",
        json: async () => ({
          error: { code: "ERROR", message: "Game creation error" },
        }),
      } as Response)

      render(<DebugPage />)

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        const alert = screen.getByRole("alert")
        expect(alert).toBeInTheDocument()
        expect(alert).toHaveTextContent(/Game creation error/i)
      })
    })
  })
})
