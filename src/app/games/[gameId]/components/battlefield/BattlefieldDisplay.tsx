"use client"

/**
 * BattlefieldDisplay - PixiJS Stage for Battlefield Rendering
 *
 * Main canvas component that renders both players' battlefields.
 *
 * Canvas: 1920×1080 (standard HD reference size)
 *
 * Zone positioning:
 * - Opponent battlefield: Y 60-380 (center at Y=220)
 * - Center gap: Y 380-460 (80px separation)
 * - Your battlefield: Y 460-780 (center at Y=550)
 *
 * Zone backgrounds:
 * - Opponent zone: #2E1F1F (dark red-brown)
 * - Your zone: #1E2A36 (dark blue-gray)
 * - Center gap: #0D1117 (near-black)
 *
 * IMPORTANT: This uses dynamic import with ssr: false for Next.js compatibility.
 * PixiJS requires browser APIs not available during SSR.
 */

import { Application, extend } from "@pixi/react"
import type * as PIXI from "pixi.js"
import { Container, Graphics } from "pixi.js"
import { useCallback, useState } from "react"
import type { GameSnapshot } from "@/echomancy/infrastructure/ui/GameSnapshot"
import { BattlefieldZoneContent } from "./BattlefieldZone"

// Register PixiJS classes with @pixi/react
extend({ Container, Graphics })

const CANVAS_WIDTH = 1920
const CANVAS_HEIGHT = 1080

// Zone positioning (from spec)
const OPPONENT_ZONE_Y = 220
const YOUR_ZONE_Y = 550

// Zone background colors (from spec)
const OPPONENT_ZONE_BG = 0x2e1f1f // Dark red-brown
const YOUR_ZONE_BG = 0x1e2a36 // Dark blue-gray
const CENTER_GAP_BG = 0x0d1117 // Near-black

type BattlefieldDisplayProps = {
  snapshot: GameSnapshot
}

export function BattlefieldDisplay({ snapshot }: BattlefieldDisplayProps) {
  const { privatePlayerState, opponentStates } = snapshot

  // Get cards from snapshot
  const yourCards = privatePlayerState.battlefield
  const opponentCards = opponentStates[0]?.battlefield ?? []

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

    // Your zone background (Y 460-780)
    g.beginFill(YOUR_ZONE_BG)
    g.drawRect(0, 460, CANVAS_WIDTH, 320)
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
            cards={yourCards}
            baseY={YOUR_ZONE_Y}
            isOpponent={false}
            renderer={renderer}
          />
        </>
      )}
    </Application>
  )
}
