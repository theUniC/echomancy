/**
 * Cost System - Domain model for ability and spell costs
 *
 * Exports:
 * - Cost interface and utilities
 * - Cost implementations (ManaCost, TapSelfCost, SacrificeSelfCost)
 */

export type { Cost, CostContext } from "./Cost"
export { canPayAllCosts, payAllCosts } from "./Cost"
export { ManaCost } from "./impl/ManaCost"
export { SacrificeSelfCost } from "./impl/SacrificeSelfCost"
export { TapSelfCost } from "./impl/TapSelfCost"
