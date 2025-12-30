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
  currentTurnNumber: 1,
  currentPlayerId: mockUUID2,
  priorityPlayerId: mockUUID2,
  currentStep: "UNTAP",
  stack: [],
  players: {
    [mockUUID2]: {
      lifeTotal: 20,
      manaPool: { W: 0, U: 0, B: 0, R: 0, G: 0, C: 0 },
      zones: {
        hand: { cards: [] },
        battlefield: { cards: [] },
        graveyard: { cards: [] },
      },
    },
    [mockUUID3]: {
      lifeTotal: 20,
      manaPool: { W: 0, U: 0, B: 0, R: 0, G: 0, C: 0 },
      zones: {
        hand: { cards: [] },
        battlefield: { cards: [] },
        graveyard: { cards: [] },
      },
    },
  },
}

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
      render(<DebugPage />)

      expect(screen.getByText("Echomancy Debug Console")).toBeInTheDocument()
    })

    it("displays create game button initially", () => {
      render(<DebugPage />)

      expect(
        screen.getByRole("button", { name: /create new game/i }),
      ).toBeInTheDocument()
    })

    it("disables action submission initially", () => {
      render(<DebugPage />)

      const textarea = screen.getByPlaceholderText(/ADVANCE_STEP/i)
      const submitButton = screen.getByRole("button", {
        name: /submit action/i,
      })

      expect(textarea).toBeDisabled()
      expect(submitButton).toBeDisabled()
    })

    it("shows placeholder text in action input", () => {
      render(<DebugPage />)

      expect(
        screen.getByPlaceholderText(
          '{"type": "ADVANCE_STEP", "playerId": "..."}',
        ),
      ).toBeInTheDocument()
    })
  })

  describe("creating a game", () => {
    it("creates game with correct API sequence", async () => {
      const user = userEvent.setup()
      const mockFetch = vi.mocked(fetch)

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

      render(<DebugPage />)

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledTimes(5)
      })

      // Verify API call sequence
      expect(mockFetch).toHaveBeenNthCalledWith(1, "/api/games", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ gameId: mockUUID1 }),
      })

      expect(mockFetch).toHaveBeenNthCalledWith(
        2,
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
        3,
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
        4,
        `/api/games/${mockUUID1}/start`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ startingPlayerId: mockUUID2 }),
        },
      )

      expect(mockFetch).toHaveBeenNthCalledWith(
        5,
        `/api/games/${mockUUID1}/state`,
      )
    })

    it("displays game IDs after successful creation", async () => {
      const user = userEvent.setup()
      const mockFetch = vi.mocked(fetch)

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
      const mockFetch = vi.mocked(fetch)

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
      const mockFetch = vi.mocked(fetch)

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
      const mockFetch = vi.mocked(fetch)

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
      const mockFetch = vi.mocked(fetch)

      // Create a promise we can control
      let resolveCreate: (value: unknown) => void
      const createPromise = new Promise((resolve) => {
        resolveCreate = resolve
      })

      mockFetch.mockReturnValueOnce(createPromise as Promise<Response>)

      render(<DebugPage />)

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
      const mockFetch = vi.mocked(fetch)

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

      render(<DebugPage />)

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        expect(screen.getByText(/Game ID:/i)).toBeInTheDocument()
      })

      return user
    }

    it("submits valid JSON action", async () => {
      const user = await setupGameCreated()
      const mockFetch = vi.mocked(fetch)

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
      const mockFetch = vi.mocked(fetch)

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
      const mockFetch = vi.mocked(fetch)

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
      const mockFetch = vi.mocked(fetch)

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
      const mockFetch = vi.mocked(fetch)

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
      const mockFetch = vi.mocked(fetch)

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

      await user.click(screen.getByRole("button", { name: /create new game/i }))

      await waitFor(() => {
        expect(screen.queryByText(/First error/i)).not.toBeInTheDocument()
      })
    })

    it("displays error with alert role for accessibility", async () => {
      const user = userEvent.setup()
      const mockFetch = vi.mocked(fetch)

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
