export class Player {
    id: string;
    name: string;
    lifeTotal: number;

    constructor(id: string, name: string, lifeTotal: number = 20) {
        this.id = id;
        this.name = name;
        this.lifeTotal = lifeTotal;
    }

    adjustLifeTotal(amount: number) {
        this.lifeTotal += amount;
    }
}