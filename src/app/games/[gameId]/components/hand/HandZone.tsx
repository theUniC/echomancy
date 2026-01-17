"use client"

/**
 * HandZone - Horizontal Row of Hand Cards
 *
 * Renders the viewing player's hand as a centered horizontal row.
 * Cards overlap with 60px overlap (120px visible width per card).
 *
 * Layout Algorithm:
 * 1. Calculate total width: (visibleWidth × count) + cardWidth
 * 2. Calculate start X: (canvasWidth - totalWidth) / 2
 * 3. Position each card: X = startX + (index × visibleWidth) + (cardWidth / 2)
 *
 * All cards are upright (rotation = 0).
 * Cards are centered at Y=925 (baseY).
 */

import { extend } from "@pixi/react"
import type * as PIXI from "pixi.js"
import { Container } from "pixi.js"
import type { CardSnapshot } from "@/echomancy/infrastructure/ui/GameSnapshot"
import { CardSprite } from "../battlefield/CardSprite"

// Register PixiJS classes with @pixi/react
extend({ Container })

const CARD_WIDTH = 180
const CARD_VISIBLE_WIDTH = 120 // 60px overlap means 120px visible
const CANVAS_WIDTH = 1920

type HandZoneProps = {
  cards: readonly CardSnapshot[]
  baseY: number
  renderer: PIXI.Renderer
}

export function HandZoneContent({ cards, baseY, renderer }: HandZoneProps) {
  // Handle empty hand
  if (cards.length === 0) {
    return null
  }

  // Calculate layout
  // Total width = visible width for all cards except the last one, plus full width for the last card
  const totalWidth =
    cards.length === 1
      ? CARD_WIDTH
      : CARD_VISIBLE_WIDTH * (cards.length - 1) + CARD_WIDTH
  const startX = (CANVAS_WIDTH - totalWidth) / 2

  return (
    <pixiContainer>
      {cards.map((card, index) => {
        const x = startX + index * CARD_VISIBLE_WIDTH + CARD_WIDTH / 2
        const y = baseY

        return (
          <CardSprite
            key={card.instanceId}
            card={card}
            x={x}
            y={y}
            isOpponent={false}
            renderer={renderer}
          />
        )
      })}
    </pixiContainer>
  )
}
