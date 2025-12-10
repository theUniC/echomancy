import { expect, test } from 'vitest'
import { Game } from './Game'

test('it can be instanciated', () => {
    expect(new Game("1", ["player1", "player2"])).toBeInstanceOf(Game)
})