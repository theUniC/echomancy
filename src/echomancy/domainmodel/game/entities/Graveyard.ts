/**
 * Graveyard Entity
 *
 * Immutable representation of the graveyard zone.
 * Wrapper around Zone with graveyard-specific operations.
 * All operations return new instances following the immutable pattern.
 *
 * Graveyard is ordered - cards are stored from bottom to top, with the most
 * recently added card on top (last element in the array).
 *
 * @example
 * const graveyard = Graveyard.empty()
 * const withCard = graveyard.addCard(creature)
 * const topCard = withCard.getTopCard() // Most recently added
 */

import type { CardInstance } from "../../cards/CardInstance"
import type { Zone } from "../../zones/Zone"

export class Graveyard {
  private readonly _cards: CardInstance[]

  private constructor(cards: CardInstance[]) {
    // Create defensive copy (not frozen for backward compatibility)
    this._cards = [...cards]
  }

  /**
   * Gets all cards in the graveyard.
   * Provided for backward compatibility with existing code that accesses zone.cards.
   */
  get cards(): CardInstance[] {
    return this._cards
  }

  /**
   * Creates an empty graveyard.
   */
  static empty(): Graveyard {
    return new Graveyard([])
  }

  /**
   * Creates a graveyard from an array of card instances.
   * Cards should be ordered from bottom to top (earliest to most recent).
   */
  static fromCards(cards: CardInstance[]): Graveyard {
    return new Graveyard(cards)
  }

  /**
   * Creates a graveyard from a Zone-like object.
   * Used for migration from existing Zone usage.
   */
  static fromZone(zone: Zone): Graveyard {
    return new Graveyard(zone.cards)
  }

  /**
   * Adds a card to the graveyard (on top).
   * @returns A new Graveyard instance with the card added.
   */
  addCard(card: CardInstance): Graveyard {
    return new Graveyard([...this._cards, card])
  }

  /**
   * Gets the top card of the graveyard (most recently added).
   * @returns The top card if graveyard is not empty, undefined otherwise.
   */
  getTopCard(): CardInstance | undefined {
    if (this._cards.length === 0) {
      return undefined
    }
    return this._cards[this._cards.length - 1]
  }

  /**
   * Gets all cards in the graveyard in order (bottom to top).
   * @returns A defensive copy of all cards.
   */
  getAll(): CardInstance[] {
    return [...this._cards]
  }

  /**
   * Checks if the graveyard is empty.
   */
  isEmpty(): boolean {
    return this._cards.length === 0
  }

  /**
   * Gets the number of cards in the graveyard.
   */
  count(): number {
    return this._cards.length
  }
}
