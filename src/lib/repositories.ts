/**
 * Shared singleton instances for API routes.
 *
 * This module provides shared repository instances that persist across
 * API route invocations. In production, these would be replaced with
 * database-backed implementations.
 *
 * NOTE: Uses globalThis pattern to ensure true singleton across Next.js
 * route handlers, which may run in separate module contexts.
 */
import { InMemoryGameRepository } from "@/echomancy/infrastructure/persistence/InMemoryGameRepository"

const globalForRepo = globalThis as unknown as {
  gameRepository: InMemoryGameRepository | undefined
}

/**
 * Shared game repository instance for API routes.
 *
 * This in-memory implementation is suitable for development and testing.
 * Data persists only for the lifetime of the server process.
 */
export const gameRepository =
  globalForRepo.gameRepository ?? new InMemoryGameRepository()

if (process.env.NODE_ENV !== "production") {
  globalForRepo.gameRepository = gameRepository
}
