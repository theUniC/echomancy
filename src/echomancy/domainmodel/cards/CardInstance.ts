import type { CardDefinition } from "./CardDefinition"

export type CardInstance = {
  instanceId: string
  definition: CardDefinition
  ownerId: string
}
