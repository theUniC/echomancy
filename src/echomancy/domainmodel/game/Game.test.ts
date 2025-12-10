import { expect, test } from 'vitest'
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
    
    expect(game.id).toBeTruthy();
    expect(game.id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
})