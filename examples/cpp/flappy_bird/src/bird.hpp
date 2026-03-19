#ifndef FLAPPY_BIRD_HPP
#define FLAPPY_BIRD_HPP

#include <goud/goud.hpp>
#include "constants.hpp"

namespace flappy {

/// The player-controlled bird: movement physics plus animation state.
class Bird {
public:
    Bird(float startX, float startY);

    /// Run one frame of physics: gravity, position, rotation, animation.
    void update(float dt, bool jumpPressed);

    /// Apply an upward impulse (if cooldown has elapsed).
    void jump();

    /// Whether the jump cooldown has elapsed.
    bool canJump() const;

    /// Axis-aligned bounding rectangle for collision testing.
    struct Bounds { float x, y, w, h; };
    Bounds getBounds() const;

    /// Draw the bird sprite at its current position and rotation.
    void draw(const goud::Context& ctx, goud_texture tex) const;

    /// Reset to starting position, velocity, and animation.
    void reset();

    // Current animation frame index (0, 1, 2).
    int animFrame() const { return animFrameIndex_; }

    float x() const { return x_; }
    float y() const { return y_; }

private:
    float x_;
    float y_;
    float velocity_       = 0.0f;
    float rotation_       = 0.0f;
    float jumpCooldown_   = 0.0f;

    // Animation
    int   animFrameIndex_ = 0;
    float animTime_       = 0.0f;
};

}  // namespace flappy

#endif
