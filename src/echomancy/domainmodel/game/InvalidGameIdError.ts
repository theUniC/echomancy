export class InvalidGameIdError extends Error {
    constructor(invalidId: string) {
        super(`The provided game ID "${invalidId}" is not a valid UUID.`)
    }
}