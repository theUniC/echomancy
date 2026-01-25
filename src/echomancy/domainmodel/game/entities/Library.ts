/**
 * Library Entity
 *
 * Immutable representation of a player's library zone (deck).
 * Wrapper around an ordered collection of cards with library-specific operations.
 * All operations return new instances following the immutable pattern.
 *
 * Library is an ordered zone - cards are drawn from the top in sequence.
 *
 * @example
 * const library = Library.fromCards([card1, card2, card3])
 * const { card, newLibrary } = library.drawFromTop()
 * const topCards = library.peekTop(3)
 */

import type { CardInstance } from "../../cards/CardInstance"

export class Library {
  private readonly _cards: CardInstance[]

  private constructor(cards: CardInstance[]) {
    // Create defensive copy (not frozen for backward compatibility)
    this._cards = [...cards]
  }

  /**
   * Gets all cards in the library.
   * Provided for backward compatibility with existing code that accesses zone.cards.
   */
  get cards(): CardInstance[] {
    return [...this._cards]
  }

  /**
   * Creates an empty library.
   */
  static empty(): Library {
    return new Library([])
  }

  /**
   * Creates a library from an array of card instances.
   * Cards should be ordered from top (index 0) to bottom (index n-1).
   */
  static fromCards(cards: CardInstance[]): Library {
    return new Library(cards)
  }

  /**
   * Draws the top card from the library.
   * @returns An object containing the drawn card (or undefined if empty) and a new Library instance.
   */
  drawFromTop(): { card: CardInstance | undefined; newLibrary: Library } {
    if (this._cards.length === 0) {
      return { card: undefined, newLibrary: this }
    }

    const [card, ...rest] = this._cards
    return { card, newLibrary: new Library(rest) }
  }

  /**
   * Peeks at the top N cards without removing them.
   * @param n - Number of cards to peek at
   * @returns A defensive copy of the top N cards (or fewer if library is smaller)
   */
  peekTop(n: number): CardInstance[] {
    return [...this._cards.slice(0, n)]
  }

  /**
   * Checks if the library is empty.
   */
  isEmpty(): boolean {
    return this._cards.length === 0
  }

  /**
   * Gets the number of cards in the library.
   */
  count(): number {
    return this._cards.length
  }

  /**
   * Adds a card to the top of the library.
   * @returns A new Library instance with the card added to top.
   */
  addToTop(card: CardInstance): Library {
    return new Library([card, ...this._cards])
  }

  /**
   * Adds a card to the bottom of the library.
   * @returns A new Library instance with the card added to bottom.
   */
  addToBottom(card: CardInstance): Library {
    return new Library([...this._cards, card])
  }

  /**
   * Gets all cards in the library.
   * @returns A defensive copy of all cards.
   */
  getAll(): CardInstance[] {
    return [...this._cards]
  }

  /**
   * Shuffles the library using the Fisher-Yates algorithm.
   *
   * Returns a new Library instance with cards in randomized order.
   * The original library is unchanged (immutable pattern).
   *
   * @param seed - Optional seed for deterministic shuffling (for testing)
   * @returns A new Library instance with shuffled cards
   */
  shuffle(seed?: number): Library {
    // Create a copy of the cards array to shuffle
    const shuffledCards = [...this._cards]

    // Use seeded RNG if seed provided, otherwise use Math.random
    const rng = seed !== undefined ? createSeededRNG(seed) : Math.random

    // Fisher-Yates shuffle algorithm
    for (let i = shuffledCards.length - 1; i > 0; i--) {
      const j = Math.floor(rng() * (i + 1))
      // Swap elements at i and j
      ;[shuffledCards[i], shuffledCards[j]] = [
        shuffledCards[j],
        shuffledCards[i],
      ]
    }

    return new Library(shuffledCards)
  }
}

/**
 * Creates a seeded pseudo-random number generator.
 *
 * Uses a simple Linear Congruential Generator (LCG) algorithm.
 * Good enough for deterministic testing, not for cryptographic purposes.
 *
 * @param seed - The seed value
 * @returns A function that returns pseudo-random numbers in [0, 1)
 */
function createSeededRNG(seed: number): () => number {
  let state = seed
  return () => {
    // LCG parameters (same as glibc)
    state = (state * 1103515245 + 12345) & 0x7fffffff
    return state / 0x7fffffff
  }
}
