import { v4 as uuidv4 } from 'uuid';

export class Player {
    id: string;
    name: string;
    lifeTotal: number;

    constructor(name: string, lifeTotal: number = 20) {
        this.id = uuidv4();
        this.name = name;
        this.lifeTotal = lifeTotal;
    }

    adjustLifeTotal(amount: number) {
        this.lifeTotal += amount;
    }
}