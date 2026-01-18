import type { Battlefield } from "./entities/Battlefield"
import type { Graveyard } from "./entities/Graveyard"
import type { Hand } from "./entities/Hand"

export type PlayerState = {
  hand: Hand
  battlefield: Battlefield
  graveyard: Graveyard
}
