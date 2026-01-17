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
import { Graphics, Sprite } from "pixi.js"
import type { CardSnapshot } from "@/echomancy/infrastructure/ui/GameSnapshot"
import { getTexture } from "../../textures/CardTextureCache"

// Register PixiJS classes with @pixi/react
extend({ Sprite, Graphics })

type CardSpriteProps = {
  card: CardSnapshot
  x: number
  y: number
  isOpponent: boolean
  renderer: PIXI.Renderer
  onClick?: (cardId: string) => void
  isPlayable?: boolean
}

export function CardSprite({
  card,
  x,
  y,
  isOpponent,
  renderer,
  onClick,
  isPlayable = false,
}: CardSpriteProps) {
  // Get texture from cache (or generate if not cached)
  const texture = getTexture(card, renderer)

  // Calculate rotation based on tapped state and player
  const baseRotation = isOpponent ? Math.PI : 0 // 180° for opponent, 0° for you
  const tapRotation = card.tapped ? Math.PI / 2 : 0 // Add 90° if tapped
  const rotation = baseRotation + tapRotation

  // Calculate alpha based on tapped state
  const alpha = card.tapped ? 0.85 : 1.0

  // Handle click event
  const handleClick = () => {
    if (onClick) {
      onClick(card.instanceId)
    }
  }

  // Draw green border for playable cards
  const drawPlayableBorder = (g: PIXI.Graphics) => {
    g.clear()

    if (isPlayable) {
      // Card dimensions
      const cardWidth = 180
      const cardHeight = 252

      // Draw green border (4px thick)
      g.lineStyle(4, 0x00ff00, 1) // Green color with full opacity
      g.drawRoundedRect(
        -cardWidth / 2,
        -cardHeight / 2,
        cardWidth,
        cardHeight,
        12, // Corner radius to match card
      )
    }
  }

  return (
    <pixiContainer x={x} y={y} rotation={rotation}>
      {/* Card sprite */}
      <pixiSprite
        texture={texture}
        anchor={0.5} // Center anchor for rotation
        alpha={alpha}
        eventMode={onClick ? "static" : "auto"} // Enable interactivity if onClick provided
        cursor={onClick ? "pointer" : "default"}
        onclick={handleClick}
      />

      {/* Playable border */}
      {isPlayable && <pixiGraphics draw={drawPlayableBorder} />}
    </pixiContainer>
  )
}
