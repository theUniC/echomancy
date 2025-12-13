import { expect, test } from 'vitest'
import { validate as isValidUUID } from 'uuid'
import { Game } from './Game'
import { Player } from './Player'

test('it can be instantiated', () => {
    const player1 = new Player("Player 1")
    const player2 = new Player("Player 2")
    const game = new Game("game-id", [player1, player2], player1.id, "UNTAP")

    expect(game).toBeInstanceOf(Game)
    expect(game.id).toBe("game-id")
    expect(game.players).toEqual([player1, player2])
    expect(game.currentPlayerId).toBe(player1.id)
    expect(game.currentStep).toBe("UNTAP")
})

test('it generates UUID for game when using start factory', () => {
    const player1 = new Player("Player 1")
    const player2 = new Player("Player 2")
    const game = Game.start([player1, player2], player1.id)

    expect(isValidUUID(game.id)).toBe(true)
})

test('it starts at UNTAP step when using start factory', () => {
    const player1 = new Player("Player 1")
    const player2 = new Player("Player 2")
    const game = Game.start([player1, player2], player1.id)

    expect(game.currentStep).toBe("UNTAP")
})

test('it sets the starting player correctly', () => {
    const player1 = new Player("Player 1")
    const player2 = new Player("Player 2")
    const game = Game.start([player1, player2], player2.id)

    expect(game.currentPlayerId).toBe(player2.id)
})

test('it stores player instances', () => {
    const player1 = new Player("Player 1")
    const player2 = new Player("Player 2")
    const game = Game.start([player1, player2], player1.id)

    expect(game.players[0]).toBe(player1)
    expect(game.players[1]).toBe(player2)
    expect(isValidUUID(game.players[0].id)).toBe(true)
    expect(isValidUUID(game.players[1].id)).toBe(true)
})