import { Action } from "../Action"

export class AdvanceStep implements Action {
  constructor(public readonly playerId: string) {}
}
