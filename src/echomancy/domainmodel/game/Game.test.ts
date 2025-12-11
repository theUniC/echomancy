import { expect, test } from 'vitest'
import { validate as isValidUUID } from 'uuid'
import { Game } from './Game'
import { Player } from '../player/Player'

test('it can be instanciated', () => {
    const player1 = new Player("Player 1");
    const player2 = new Player("Player 2");
    expect(new Game("game-id", [player1.id, player2.id])).toBeInstanceOf(Game)
})

test('it generates UUID for game when using start factory', () => {
    const player1 = new Player("Player 1");
    const player2 = new Player("Player 2");
    const game = Game.start([player1.id, player2.id]);

    expect(isValidUUID(game.id)).toBe(true);
})