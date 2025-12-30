/**
 * Shared singleton instances for API routes.
 *
 * This module provides shared repository instances that persist across
 * API route invocations. In production, these would be replaced with
 * database-backed implementations.
 */
import { InMemoryGameRepository } from "@/echomancy/infrastructure/persistence/InMemoryGameRepository"

/**
 * Shared game repository instance for API routes.
 *
 * This in-memory implementation is suitable for development and testing.
 * Data persists only for the lifetime of the server process.
 */
export const gameRepository = new InMemoryGameRepository()
