import type { Battlefield } from "./entities/Battlefield"
import type { Graveyard } from "./entities/Graveyard"
import type { Hand } from "./entities/Hand"
import type { Library } from "./entities/Library"

export type PlayerState = {
  hand: Hand
  battlefield: Battlefield
  graveyard: Graveyard
  library: Library
}
