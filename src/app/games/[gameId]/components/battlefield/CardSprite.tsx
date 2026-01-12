"use client"

/**
 * CardSprite - Individual Card Rendering
 *
 * Renders a single card as a PixiJS Sprite with:
 * - Texture from cache
 * - Rotation based on tapped state and player (yours vs opponent)
 * - Opacity reduction for tapped cards
 * - Center anchor for proper rotation
 *
 * Rotation values (in radians):
 * - Your untapped: 0
 * - Your tapped: Math.PI / 2 (90°)
 * - Opponent untapped: Math.PI (180°)
 * - Opponent tapped: 3 * Math.PI / 2 (270°)
 *
 * Alpha values:
 * - Untapped: 1.0
 * - Tapped: 0.85
 */

import { extend } from "@pixi/react"
import type * as PIXI from "pixi.js"
import { Sprite } from "pixi.js"
import type { CardSnapshot } from "@/echomancy/infrastructure/ui/GameSnapshot"
import { getTexture } from "../../textures/CardTextureCache"

// Register PixiJS classes with @pixi/react
extend({ Sprite })

type CardSpriteProps = {
  card: CardSnapshot
  x: number
  y: number
  isOpponent: boolean
  renderer: PIXI.Renderer
}

export function CardSprite({
  card,
  x,
  y,
  isOpponent,
  renderer,
}: CardSpriteProps) {
  // Get texture from cache (or generate if not cached)
  const texture = getTexture(card, renderer)

  // Calculate rotation based on tapped state and player
  const baseRotation = isOpponent ? Math.PI : 0 // 180° for opponent, 0° for you
  const tapRotation = card.tapped ? Math.PI / 2 : 0 // Add 90° if tapped
  const rotation = baseRotation + tapRotation

  // Calculate alpha based on tapped state
  const alpha = card.tapped ? 0.85 : 1.0

  return (
    <pixiSprite
      texture={texture}
      x={x}
      y={y}
      anchor={0.5} // Center anchor for rotation
      rotation={rotation}
      alpha={alpha}
    />
  )
}
