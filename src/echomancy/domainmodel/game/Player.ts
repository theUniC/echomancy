const DEFAULT_LIFE_TOTAL = 20

export class Player {
  id: string
  name: string
  lifeTotal: number

  constructor(
    id: string,
    name: string,
    lifeTotal: number = DEFAULT_LIFE_TOTAL,
  ) {
    this.id = id
    this.name = name
    this.lifeTotal = lifeTotal
  }

  adjustLifeTotal(amount: number) {
    this.lifeTotal += amount
  }
}
