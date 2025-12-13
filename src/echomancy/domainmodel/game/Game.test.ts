import { expect, test } from 'vitest'
import { validate as isValidUUID } from 'uuid'
import { Game } from './Game'
import { Player } from './Player'
import { AdvanceStep } from './actions/AdvanceStep'

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

test('it can apply AdvanceStep action', () => {
    const player1 = new Player("Player 1")
    const player2 = new Player("Player 2")
    const game = Game.start([player1, player2], player1.id)

    game.apply(new AdvanceStep(player1.id))

    expect(game.currentStep).toBe("UPKEEP")
})

test('it throws error when non-current player tries to advance step', () => {
    const player1 = new Player("Player 1")
    const player2 = new Player("Player 2")
    const game = Game.start([player1, player2], player1.id)

    expect(() => {
        game.apply(new AdvanceStep(player2.id))
    }).toThrow('Only the current player can advance the step')
})

test('it advances to next player when completing a turn', () => {
    const player1 = new Player("Player 1")
    const player2 = new Player("Player 2")
    const game = Game.start([player1, player2], player1.id)

    // Advance through all steps to complete player1's turn
    const steps = ["UNTAP", "UPKEEP", "DRAW", "MAIN1", "BEGINING_OF_COMBAT",
        "DECLARE_ATTACKERS", "DECLARE_BLOCKERS", "COMBAT_DAMAGE",
        "END_OF_COMBAT", "SECOND_MAIN", "END_STEP", "CLEANUP"]

    for (let i = 0; i < steps.length; i++) {
        game.apply(new AdvanceStep(game.currentPlayerId))
    }

    expect(game.currentStep).toBe("UNTAP")
    expect(game.currentPlayerId).toBe(player2.id)
})

test('it validates starting player is in player list', () => {
    const player1 = new Player("Player 1")
    const player2 = new Player("Player 2")

    expect(() => {
        Game.start([player1, player2], "invalid-id")
    }).toThrow('Starting player must be in player list')
})

test('it requires at least 2 players', () => {
    const player1 = new Player("Player 1")

    expect(() => {
        Game.start([player1], player1.id)
    }).toThrow('Game requires at least 2 players')
})