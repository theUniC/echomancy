/**
 * Hand Entity
 *
 * Immutable representation of a player's hand zone.
 * Wrapper around Zone with hand-specific operations.
 * All operations return new instances following the immutable pattern.
 *
 * @example
 * const hand = Hand.empty()
 * const withCard = hand.addCard(card)
 * const afterPlay = withCard.removeCard(card.instanceId)
 */

import type { CardInstance } from "../../cards/CardInstance"
import type { Zone } from "../../zones/Zone"

export class Hand {
  private readonly _cards: CardInstance[]

  private constructor(cards: CardInstance[]) {
    // Create defensive copy (not frozen for backward compatibility)
    this._cards = [...cards]
  }

  /**
   * Gets all cards in the hand.
   * Provided for backward compatibility with existing code that accesses zone.cards.
   */
  get cards(): CardInstance[] {
    return this._cards
  }

  /**
   * Creates an empty hand.
   */
  static empty(): Hand {
    return new Hand([])
  }

  /**
   * Creates a hand from an array of card instances.
   */
  static fromCards(cards: CardInstance[]): Hand {
    return new Hand(cards)
  }

  /**
   * Creates a hand from a Zone-like object.
   * Used for migration from existing Zone usage.
   */
  static fromZone(zone: Zone): Hand {
    return new Hand(zone.cards)
  }

  /**
   * Adds a card to the hand.
   * @returns A new Hand instance with the card added.
   */
  addCard(card: CardInstance): Hand {
    return new Hand([...this._cards, card])
  }

  /**
   * Removes a card from the hand by instanceId.
   * If the card doesn't exist, returns a new hand with the same cards.
   * @returns A new Hand instance with the card removed.
   */
  removeCard(instanceId: string): Hand {
    const filtered = this._cards.filter(
      (card) => card.instanceId !== instanceId,
    )
    return new Hand(filtered)
  }

  /**
   * Finds a card by its instanceId.
   * @returns The card if found, undefined otherwise.
   */
  findCard(instanceId: string): CardInstance | undefined {
    return this._cards.find((card) => card.instanceId === instanceId)
  }

  /**
   * Gets all cards in the hand.
   * @returns A defensive copy of all cards.
   */
  getAll(): CardInstance[] {
    return [...this._cards]
  }

  /**
   * Checks if the hand is empty.
   */
  isEmpty(): boolean {
    return this._cards.length === 0
  }

  /**
   * Gets the number of cards in the hand.
   */
  count(): number {
    return this._cards.length
  }
}
