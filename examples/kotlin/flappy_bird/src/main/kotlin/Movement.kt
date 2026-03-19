/**
 * Simple 2D movement helpers shared across game objects.
 */
object Movement {
    /**
     * Move an x position leftward at the given speed, scaled by dt and TARGET_FPS.
     */
    fun scrollLeft(x: Float, speed: Float, dt: Float): Float {
        return x - speed * GameConstants.TARGET_FPS * dt
    }
}
