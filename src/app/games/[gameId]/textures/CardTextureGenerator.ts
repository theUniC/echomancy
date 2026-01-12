/**
 * CardTextureGenerator - Procedural Card Texture Generation
 *
 * Generates card frame textures using PixiJS Graphics API.
 * Creates professional TCG card frames with:
 * - Typed borders (creature/land/artifact/enchantment colors)
 * - Multi-section structure (name, art, type, text, P/T)
 * - Typography with drop shadows
 * - No external image assets required
 *
 * All dimensions and colors follow the Phase 1c spec.
 */

import * as PIXI from "pixi.js"
import type { CardSnapshot } from "@/echomancy/infrastructure/ui/GameSnapshot"

// Card dimensions (from spec)
const CARD_WIDTH = 180
const CARD_HEIGHT = 250
const CORNER_RADIUS = 12
const BORDER_WIDTH = 3
const PADDING = 8

// Section heights (from spec)
const NAME_BAR_HEIGHT = 36
const ART_AREA_HEIGHT = 120
const TYPE_LINE_HEIGHT = 28
const TEXT_BOX_HEIGHT = 46
const PT_BOX_HEIGHT = 20

// Color palette (from spec)
const BORDER_COLORS = {
  CREATURE: 0x4caf50, // Green
  LAND: 0x8d6e63, // Brown
  ARTIFACT: 0x90a4ae, // Gray
  ENCHANTMENT: 0xab47bc, // Purple
}

const SECTION_BACKGROUNDS = {
  NAME_BAR: 0x2c2c2c, // Dark gray
  ART_AREA: 0x1a1a1a, // Near-black
  TYPE_LINE: 0x3a3a3a, // Medium dark gray
  TEXT_BOX: 0x4a4a4a, // Medium gray
  PT_BOX: 0x2c2c2c, // Dark gray
}

const TEXT_COLORS = {
  CARD_NAME: 0xffffff, // White
  TYPE_TEXT: 0xe0e0e0, // Light gray
  RULES_TEXT: 0xcccccc, // Medium light gray
  PT: 0xffffff, // White
}

/**
 * Generates a card texture from a CardSnapshot.
 *
 * @param card - The card data to render
 * @param renderer - PixiJS renderer for texture generation
 * @returns A PixiJS Texture ready for use in Sprite components
 */
export function generateCardTexture(
  card: CardSnapshot,
  renderer: PIXI.Renderer,
): PIXI.Texture {
  const graphics = new PIXI.Graphics()

  // Determine border color from card type
  const borderColor = getBorderColor(card)

  // Draw card border (outer rounded rectangle)
  graphics
    .roundRect(0, 0, CARD_WIDTH, CARD_HEIGHT, CORNER_RADIUS)
    .fill({ color: 0x000000 }) // Will be covered by sections
    .stroke({ width: BORDER_WIDTH, color: borderColor })

  // Draw name bar section
  drawSection(
    graphics,
    0,
    0,
    CARD_WIDTH,
    NAME_BAR_HEIGHT,
    SECTION_BACKGROUNDS.NAME_BAR,
  )

  // Draw art area section
  drawSection(
    graphics,
    0,
    NAME_BAR_HEIGHT,
    CARD_WIDTH,
    ART_AREA_HEIGHT,
    SECTION_BACKGROUNDS.ART_AREA,
  )

  // Draw type line section
  drawSection(
    graphics,
    0,
    NAME_BAR_HEIGHT + ART_AREA_HEIGHT,
    CARD_WIDTH,
    TYPE_LINE_HEIGHT,
    SECTION_BACKGROUNDS.TYPE_LINE,
  )

  // Draw text box section
  drawSection(
    graphics,
    0,
    NAME_BAR_HEIGHT + ART_AREA_HEIGHT + TYPE_LINE_HEIGHT,
    CARD_WIDTH,
    TEXT_BOX_HEIGHT,
    SECTION_BACKGROUNDS.TEXT_BOX,
  )

  // Draw P/T box section (only for creatures)
  if (card.types.includes("CREATURE") && card.power !== null) {
    drawSection(
      graphics,
      0,
      NAME_BAR_HEIGHT + ART_AREA_HEIGHT + TYPE_LINE_HEIGHT + TEXT_BOX_HEIGHT,
      CARD_WIDTH,
      PT_BOX_HEIGHT,
      SECTION_BACKGROUNDS.PT_BOX,
    )
  }

  // Note: DropShadowFilter removed for MVP (not available in PixiJS v8 core)
  // Can be added later with @pixi/filter-drop-shadow package if needed

  // Create a container to hold graphics and text
  const container = new PIXI.Container()
  container.addChild(graphics)

  // Add card name text
  const nameText = createCardNameText(card.name)
  nameText.x = CARD_WIDTH / 2
  nameText.y = NAME_BAR_HEIGHT / 2
  container.addChild(nameText)

  // Add type line text
  const typeText = createTypeLineText(card)
  typeText.x = PADDING
  typeText.y = NAME_BAR_HEIGHT + ART_AREA_HEIGHT + TYPE_LINE_HEIGHT / 2
  container.addChild(typeText)

  // Add rules text (keywords)
  if (card.staticKeywords.length > 0) {
    const rulesText = createRulesText(card.staticKeywords)
    rulesText.x = PADDING
    rulesText.y = NAME_BAR_HEIGHT + ART_AREA_HEIGHT + TYPE_LINE_HEIGHT + 8
    container.addChild(rulesText)
  }

  // Add P/T text (creatures only)
  if (card.types.includes("CREATURE") && card.power !== null) {
    const ptText = createPTText(card.power, card.toughness ?? 0)
    ptText.x = CARD_WIDTH / 2
    ptText.y =
      NAME_BAR_HEIGHT +
      ART_AREA_HEIGHT +
      TYPE_LINE_HEIGHT +
      TEXT_BOX_HEIGHT +
      PT_BOX_HEIGHT / 2
    container.addChild(ptText)
  }

  // Generate texture from container
  return renderer.generateTexture(container)
}

/**
 * Draws a rectangular section with background color.
 */
function drawSection(
  graphics: PIXI.Graphics,
  x: number,
  y: number,
  width: number,
  height: number,
  fillColor: number,
): void {
  graphics.rect(x, y, width, height).fill({ color: fillColor })
}

/**
 * Determines border color based on card types.
 * Prioritizes CREATURE > LAND > ARTIFACT > ENCHANTMENT.
 */
function getBorderColor(card: CardSnapshot): number {
  if (card.types.includes("CREATURE")) {
    return BORDER_COLORS.CREATURE
  }
  if (card.types.includes("LAND")) {
    return BORDER_COLORS.LAND
  }
  if (card.types.includes("ARTIFACT")) {
    return BORDER_COLORS.ARTIFACT
  }
  if (card.types.includes("ENCHANTMENT")) {
    return BORDER_COLORS.ENCHANTMENT
  }
  // Default to gray if no matching type
  return BORDER_COLORS.ARTIFACT
}

/**
 * Creates card name text with drop shadow.
 * Spec: 16px bold, white, centered.
 */
function createCardNameText(name: string): PIXI.Text {
  return new PIXI.Text({
    text: name,
    style: {
      fontFamily: "Inter, sans-serif",
      fontSize: 16,
      fontWeight: "bold",
      fill: TEXT_COLORS.CARD_NAME,
      align: "center",
      dropShadow: {
        alpha: 0.5,
        angle: Math.PI / 2,
        blur: 2,
        color: 0x000000,
        distance: 2,
      },
    },
    anchor: { x: 0.5, y: 0.5 },
  })
}

/**
 * Creates type line text.
 * Spec: 12px semi-bold italic, light gray, left-aligned.
 * Format: "Creature — Bear" or "Basic Land — Forest"
 */
function createTypeLineText(card: CardSnapshot): PIXI.Text {
  // Build type line string
  const typeString = card.types.join(" ")
  // For now, subtypes are empty in MVP, so just show types
  const typeLine =
    card.subtypes.length > 0
      ? `${typeString} — ${card.subtypes.join(" ")}`
      : typeString

  return new PIXI.Text({
    text: typeLine,
    style: {
      fontFamily: "Inter, sans-serif",
      fontSize: 12,
      fontWeight: "600", // Semi-bold
      fontStyle: "italic",
      fill: TEXT_COLORS.TYPE_TEXT,
      align: "left",
    },
    anchor: { x: 0, y: 0.5 },
  })
}

/**
 * Creates rules text for keywords/abilities.
 * Spec: 11px regular, medium light gray, left-aligned, word wrap enabled.
 */
function createRulesText(keywords: readonly string[]): PIXI.Text {
  const keywordString = keywords.join(", ")

  return new PIXI.Text({
    text: keywordString,
    style: {
      fontFamily: "Inter, sans-serif",
      fontSize: 11,
      fontWeight: "normal",
      fill: TEXT_COLORS.RULES_TEXT,
      align: "left",
      wordWrap: true,
      wordWrapWidth: CARD_WIDTH - PADDING * 2,
    },
    anchor: { x: 0, y: 0 },
  })
}

/**
 * Creates P/T text for creatures.
 * Spec: 16px extra-bold, white, centered, drop shadow.
 * Format: "{power}/{toughness}" (e.g., "2/2", "4/4")
 */
function createPTText(power: number, toughness: number): PIXI.Text {
  return new PIXI.Text({
    text: `${power}/${toughness}`,
    style: {
      fontFamily: "Inter, sans-serif",
      fontSize: 16,
      fontWeight: "800", // Extra-bold
      fill: TEXT_COLORS.PT,
      align: "center",
      dropShadow: {
        alpha: 0.5,
        angle: Math.PI / 2,
        blur: 2,
        color: 0x000000,
        distance: 2,
      },
    },
    anchor: { x: 0.5, y: 0.5 },
  })
}
