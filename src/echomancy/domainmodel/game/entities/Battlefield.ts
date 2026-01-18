/**
 * Battlefield Entity
 *
 * Immutable representation of the battlefield zone.
 * Wrapper around Zone with battlefield-specific operations.
 * All operations return new instances following the immutable pattern.
 *
 * @example
 * const battlefield = Battlefield.empty()
 * const withCreature = battlefield.addPermanent(creature)
 * const afterRemove = withCreature.removePermanent(creature.instanceId)
 */

import type { CardInstance } from "../../cards/CardInstance"
import type { Zone } from "../../zones/Zone"

export class Battlefield {
  private readonly _cards: CardInstance[]

  private constructor(cards: CardInstance[]) {
    // Create defensive copy (not frozen for backward compatibility)
    this._cards = [...cards]
  }

  /**
   * Gets all cards on the battlefield.
   * Provided for backward compatibility with existing code that accesses zone.cards.
   * @returns The array of cards.
   */
  get cards(): CardInstance[] {
    return this._cards
  }

  /**
   * Creates an empty battlefield.
   */
  static empty(): Battlefield {
    return new Battlefield([])
  }

  /**
   * Creates a battlefield from an array of card instances.
   */
  static fromCards(cards: CardInstance[]): Battlefield {
    return new Battlefield(cards)
  }

  /**
   * Creates a battlefield from a Zone-like object.
   * Used for migration from existing Zone usage.
   */
  static fromZone(zone: Zone): Battlefield {
    return new Battlefield(zone.cards)
  }

  /**
   * Adds a permanent to the battlefield.
   * @returns A new Battlefield instance with the permanent added.
   */
  addPermanent(permanent: CardInstance): Battlefield {
    return new Battlefield([...this.cards, permanent])
  }

  /**
   * Removes a permanent from the battlefield by instanceId.
   * If the permanent doesn't exist, returns a new battlefield with the same cards.
   * @returns A new Battlefield instance with the permanent removed.
   */
  removePermanent(instanceId: string): Battlefield {
    const filtered = this.cards.filter((card) => card.instanceId !== instanceId)
    return new Battlefield(filtered)
  }

  /**
   * Finds a permanent by its instanceId.
   * @returns The permanent if found, undefined otherwise.
   */
  findPermanent(instanceId: string): CardInstance | undefined {
    return this.cards.find((card) => card.instanceId === instanceId)
  }

  /**
   * Finds all permanents owned by a specific player.
   * @returns Array of permanents owned by the player.
   */
  findPermanentsByOwner(ownerId: string): CardInstance[] {
    return this.cards.filter((card) => card.ownerId === ownerId)
  }

  /**
   * Gets all permanents on the battlefield.
   * @returns A defensive copy of all permanents.
   */
  getAll(): CardInstance[] {
    return [...this.cards]
  }

  /**
   * Checks if the battlefield is empty.
   */
  isEmpty(): boolean {
    return this.cards.length === 0
  }

  /**
   * Gets the number of permanents on the battlefield.
   */
  count(): number {
    return this.cards.length
  }
}
