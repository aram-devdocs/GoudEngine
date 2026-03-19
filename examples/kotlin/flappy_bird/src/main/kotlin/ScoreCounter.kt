/**
 * Tracks the player score.
 */
class ScoreCounter {
    var value: Int = 0
        private set

    fun increment() {
        value++
    }

    fun reset() {
        value = 0
    }
}
