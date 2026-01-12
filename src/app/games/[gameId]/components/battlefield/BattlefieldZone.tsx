"use client"

/**
 * BattlefieldZone - Horizontal Row of Cards
 *
 * Renders a player's battlefield as a centered horizontal row.
 *
 * Layout Algorithm:
 * 1. Calculate total width: (cardWidth × count) + (spacing × (count - 1))
 * 2. Calculate start X: (canvasWidth - totalWidth) / 2
 * 3. Position each card: X = startX + (index × (cardWidth + spacing))
 *
 * Each card is centered at its position with baseY as the vertical center.
 */

import { extend } from "@pixi/react"
import type * as PIXI from "pixi.js"
import { Container } from "pixi.js"
import type { CardSnapshot } from "@/echomancy/infrastructure/ui/GameSnapshot"
import { CardSprite } from "./CardSprite"

// Register PixiJS classes with @pixi/react
extend({ Container })

const CARD_WIDTH = 180
const CARD_SPACING = 20
const CANVAS_WIDTH = 1920

type BattlefieldZoneProps = {
  cards: readonly CardSnapshot[]
  baseY: number
  isOpponent: boolean
  renderer: PIXI.Renderer
}

export function BattlefieldZoneContent({
  cards,
  baseY,
  isOpponent,
  renderer,
}: BattlefieldZoneProps) {
  // Handle empty battlefield
  if (cards.length === 0) {
    return null
  }

  // Calculate layout
  const totalWidth =
    CARD_WIDTH * cards.length + CARD_SPACING * (cards.length - 1)
  const startX = (CANVAS_WIDTH - totalWidth) / 2

  return (
    <pixiContainer>
      {cards.map((card, index) => {
        const x = startX + index * (CARD_WIDTH + CARD_SPACING) + CARD_WIDTH / 2
        const y = baseY

        return (
          <CardSprite
            key={card.instanceId}
            card={card}
            x={x}
            y={y}
            isOpponent={isOpponent}
            renderer={renderer}
          />
        )
      })}
    </pixiContainer>
  )
}
