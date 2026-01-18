/**
 * TheStack Entity
 *
 * Represents the game's stack zone where spells and abilities wait to resolve.
 * The stack follows LIFO (Last In, First Out) order - the most recently added
 * item resolves first.
 *
 * Named "TheStack" to avoid confusion with JavaScript's Stack data structure
 * and to emphasize it's "the" stack from MTG rules.
 *
 * @example
 * const stack = TheStack.empty()
 * const withSpell = stack.push(spell)
 * const { item, stack: newStack } = withSpell.pop()
 */

import type { StackItem } from "../StackTypes"

export class TheStack {
  private readonly _items: StackItem[]

  private constructor(items: StackItem[]) {
    // Create defensive copy (not frozen for backward compatibility)
    this._items = [...items]
  }

  /**
   * Gets all items on the stack.
   * Provided for backward compatibility with existing code.
   */
  get items(): StackItem[] {
    return this._items
  }

  /**
   * Creates an empty stack.
   */
  static empty(): TheStack {
    return new TheStack([])
  }

  /**
   * Creates a stack from an array of items.
   * Items should be ordered from bottom to top (oldest to newest).
   */
  static fromItems(items: StackItem[]): TheStack {
    return new TheStack(items)
  }

  /**
   * Pushes an item onto the stack (on top).
   * @returns A new TheStack instance with the item added.
   */
  push(item: StackItem): TheStack {
    return new TheStack([...this._items, item])
  }

  /**
   * Pops the top item from the stack.
   * @returns An object containing the popped item (or undefined if empty)
   *          and a new TheStack instance without that item.
   */
  pop(): { item: StackItem | undefined; stack: TheStack } {
    if (this._items.length === 0) {
      return { item: undefined, stack: TheStack.empty() }
    }
    const newItems = [...this._items]
    const item = newItems.pop()
    return { item, stack: new TheStack(newItems) }
  }

  /**
   * Peeks at the top item without removing it.
   * @returns The top item if stack is not empty, undefined otherwise.
   */
  peek(): StackItem | undefined {
    if (this._items.length === 0) {
      return undefined
    }
    return this._items[this._items.length - 1]
  }

  /**
   * Gets all items on the stack in order (bottom to top).
   * @returns A defensive copy of all items.
   */
  getAll(): StackItem[] {
    return [...this._items]
  }

  /**
   * Checks if the stack is empty.
   */
  isEmpty(): boolean {
    return this._items.length === 0
  }

  /**
   * Checks if the stack has any items.
   * Convenience method that reads better in conditionals.
   */
  hasItems(): boolean {
    return this._items.length > 0
  }

  /**
   * Gets the number of items on the stack.
   */
  count(): number {
    return this._items.length
  }

  /**
   * Clears all items from the stack.
   * @returns An empty TheStack instance.
   */
  clear(): TheStack {
    return TheStack.empty()
  }
}
