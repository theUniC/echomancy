/**
 * Specification Pattern Interface
 *
 * A specification encapsulates a business rule that can be evaluated
 * against a context. Specifications are composable and testable in isolation.
 *
 * @example
 * class CanPlayLand implements Specification<GameActionContext> {
 *   isSatisfiedBy(context: GameActionContext): boolean {
 *     // Check if player can play a land
 *   }
 * }
 */
export interface Specification<T> {
  isSatisfiedBy(context: T): boolean
}
