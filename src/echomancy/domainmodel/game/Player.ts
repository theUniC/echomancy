import { v4 as uuidv4 } from "uuid"

const DEFAULT_LIFE_TOTAL = 20

export class Player {
  id: string
  name: string
  lifeTotal: number

  constructor(name: string, lifeTotal: number = DEFAULT_LIFE_TOTAL) {
    this.id = uuidv4()
    this.name = name
    this.lifeTotal = lifeTotal
  }

  adjustLifeTotal(amount: number) {
    this.lifeTotal += amount
  }
}
