/**
 * Cost System - Domain model for ability and spell costs
 *
 * Exports:
 * - Cost interface and utilities
 * - Cost implementations (ManaCost, TapSelfCost, SacrificeSelfCost)
 */

export type { Cost, CostContext } from "./Cost"
export { canPayAllCosts, payAllCosts } from "./Cost"
export { ManaCost, SacrificeSelfCost, TapSelfCost } from "./impl/CostImpls"
