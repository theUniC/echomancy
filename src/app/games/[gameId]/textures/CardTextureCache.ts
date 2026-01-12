/**
 * CardTextureCache - In-Memory Texture Caching
 *
 * Caches generated card textures to avoid redundant generation.
 * Key includes card state to ensure correct visuals for tapped/untapped.
 *
 * Cache key format: `{cardName}_{types}_{power}_{toughness}_{keywords}_{tapped}`
 *
 * This cache is session-scoped (cleared on page reload).
 * No eviction needed since card count is bounded by deck size.
 */

import type * as PIXI from "pixi.js"
import type { CardSnapshot } from "@/echomancy/infrastructure/ui/GameSnapshot"
import { generateCardTexture } from "./CardTextureGenerator"

/**
 * In-memory texture cache.
 * Maps cache keys to generated textures.
 */
const textureCache = new Map<string, PIXI.Texture>()

/**
 * Generates a cache key from card data.
 * Key includes all visual-affecting properties.
 *
 * @param card - The card snapshot
 * @returns A unique cache key string
 */
function getCacheKey(card: CardSnapshot): string {
  const types = card.types.join("_")
  const pt =
    card.power !== null && card.toughness !== null
      ? `${card.power}_${card.toughness}`
      : "none"
  const keywords =
    card.staticKeywords.length > 0 ? card.staticKeywords.join("_") : "none"
  const tapped = card.tapped === true ? "tapped" : "untapped"

  return `${card.name}_${types}_${pt}_${keywords}_${tapped}`
}

/**
 * Retrieves a texture from cache or generates it if not cached.
 *
 * @param card - The card snapshot to render
 * @param renderer - PixiJS renderer for texture generation
 * @returns A cached or newly generated texture
 */
export function getTexture(
  card: CardSnapshot,
  renderer: PIXI.Renderer,
): PIXI.Texture {
  const key = getCacheKey(card)

  // Check cache first
  const cached = textureCache.get(key)
  if (cached) {
    return cached
  }

  // Generate new texture
  const texture = generateCardTexture(card, renderer)

  // Store in cache
  textureCache.set(key, texture)

  return texture
}

/**
 * Clears all cached textures.
 * Useful for cleanup or testing.
 */
export function clearCache(): void {
  // Destroy all textures to free GPU memory
  for (const texture of textureCache.values()) {
    texture.destroy(true)
  }
  textureCache.clear()
}

/**
 * Gets current cache size (for debugging/monitoring).
 *
 * @returns Number of cached textures
 */
export function getCacheSize(): number {
  return textureCache.size
}
