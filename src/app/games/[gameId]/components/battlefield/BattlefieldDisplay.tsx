"use client"

/**
 * BattlefieldDisplay - PixiJS Stage for Battlefield and Hand Rendering
 *
 * Main canvas component that renders both players' battlefields and the viewing player's hand.
 *
 * Canvas: 1920×1220 (extended to include hand zone)
 *
 * Zone positioning:
 * - Opponent battlefield: Y 60-380 (center at Y=220)
 * - Center gap: Y 380-460 (80px separation)
 * - Your battlefield: Y 460-780 (center at Y=550)
 * - Separator: Y 780-820 (40px separator)
 * - Hand zone: Y 820-1040 (cards at Y=925)
 *
 * Zone backgrounds:
 * - Opponent zone: #2E1F1F (dark red-brown)
 * - Your zone: #1E2A36 (dark blue-gray)
 * - Center gap: #0D1117 (near-black)
 * - Separator: #0D1117 (near-black)
 * - Hand zone: #0F1419 (dark)
 *
 * IMPORTANT: This uses dynamic import with ssr: false for Next.js compatibility.
 * PixiJS requires browser APIs not available during SSR.
 */

import { Application, extend } from "@pixi/react"
import type * as PIXI from "pixi.js"
import { Container, Graphics } from "pixi.js"
import { useCallback, useState } from "react"
import type { GameSnapshot } from "@/echomancy/infrastructure/ui/GameSnapshot"
import { HandZoneContent } from "../hand/HandZone"
import { BattlefieldZoneContent } from "./BattlefieldZone"

// Register PixiJS classes with @pixi/react
extend({ Container, Graphics })

const CANVAS_WIDTH = 1920
const CANVAS_HEIGHT = 1220 // Extended to include hand zone

// Zone positioning (from spec)
const OPPONENT_ZONE_Y = 220
const YOUR_ZONE_Y = 550
const HAND_ZONE_Y = 925 // Center of hand zone

// Zone background colors (from spec)
const OPPONENT_ZONE_BG = 0x2e1f1f // Dark red-brown
const YOUR_ZONE_BG = 0x1e2a36 // Dark blue-gray
const CENTER_GAP_BG = 0x0d1117 // Near-black
const SEPARATOR_BG = 0x0d1117 // Near-black (separator between battlefield and hand)
const HAND_ZONE_BG = 0x0f1419 // Dark (hand zone)

type BattlefieldDisplayProps = {
  snapshot: GameSnapshot
  onHandCardClick?: (cardId: string) => void
  playableCardIds?: readonly string[]
}

export function BattlefieldDisplay({
  snapshot,
  onHandCardClick,
  playableCardIds = [],
}: BattlefieldDisplayProps) {
  const { privatePlayerState, opponentStates } = snapshot

  // Get cards from snapshot
  const yourBattlefieldCards = privatePlayerState.battlefield
  const opponentCards = opponentStates[0]?.battlefield ?? []
  const yourHandCards = privatePlayerState.hand

  // Store renderer reference
  const [renderer, setRenderer] = useState<PIXI.Renderer | null>(null)

  // Draw zone backgrounds
  const drawBackground = useCallback((g: PIXI.Graphics) => {
    // Clear previous drawings
    g.clear()

    // Opponent zone background (Y 60-380)
    g.beginFill(OPPONENT_ZONE_BG)
    g.drawRect(0, 60, CANVAS_WIDTH, 320)
    g.endFill()

    // Center gap (Y 380-460)
    g.beginFill(CENTER_GAP_BG)
    g.drawRect(0, 380, CANVAS_WIDTH, 80)
    g.endFill()

    // Your battlefield zone background (Y 460-780)
    g.beginFill(YOUR_ZONE_BG)
    g.drawRect(0, 460, CANVAS_WIDTH, 320)
    g.endFill()

    // Separator between battlefield and hand (Y 780-820)
    g.beginFill(SEPARATOR_BG)
    g.drawRect(0, 780, CANVAS_WIDTH, 40)
    g.endFill()

    // Hand zone background (Y 820-1040)
    g.beginFill(HAND_ZONE_BG)
    g.drawRect(0, 820, CANVAS_WIDTH, 220)
    g.endFill()
  }, [])

  return (
    <Application
      width={CANVAS_WIDTH}
      height={CANVAS_HEIGHT}
      background={CENTER_GAP_BG}
      antialias={true}
      onInit={(app) => {
        // Store renderer when Application initializes
        setRenderer(app.renderer as PIXI.Renderer)
      }}
    >
      {/* Zone backgrounds */}
      <pixiGraphics draw={drawBackground} />

      {/* Only render zones when renderer is available */}
      {renderer && (
        <>
          {/* Opponent battlefield */}
          <BattlefieldZoneContent
            cards={opponentCards}
            baseY={OPPONENT_ZONE_Y}
            isOpponent={true}
            renderer={renderer}
          />

          {/* Your battlefield */}
          <BattlefieldZoneContent
            cards={yourBattlefieldCards}
            baseY={YOUR_ZONE_Y}
            isOpponent={false}
            renderer={renderer}
          />

          {/* Your hand */}
          <HandZoneContent
            cards={yourHandCards}
            baseY={HAND_ZONE_Y}
            renderer={renderer}
            onCardClick={onHandCardClick}
            playableCardIds={playableCardIds}
          />
        </>
      )}
    </Application>
  )
}
