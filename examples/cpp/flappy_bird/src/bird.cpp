#include "bird.hpp"

#include <algorithm>
#include <cmath>

namespace flappy {

Bird::Bird(float startX, float startY)
    : x_(startX), y_(startY) {}

void Bird::update(float dt, bool jumpPressed) {
    if (jumpPressed) {
        jump();
    }

    // Gravity
    velocity_ += GRAVITY * dt * TARGET_FPS;
    jumpCooldown_ = std::max(jumpCooldown_ - dt, 0.0f);

    // Position
    y_ += velocity_ * dt;

    // Rotation (smooth towards velocity-driven target)
    float targetRotation = std::clamp(velocity_ * 3.0f, -45.0f, 45.0f);
    rotation_ += (targetRotation - rotation_) * ROTATION_SMOOTHING;

    // Animation
    animTime_ += dt;
    if (animTime_ >= ANIMATION_FRAME_DURATION) {
        animFrameIndex_ = (animFrameIndex_ + 1) % 3;
        animTime_ = 0.0f;
    }
}

void Bird::jump() {
    if (canJump()) {
        velocity_ = JUMP_STRENGTH * TARGET_FPS;
        jumpCooldown_ = JUMP_COOLDOWN;
    }
}

bool Bird::canJump() const {
    return jumpCooldown_ <= 0.0f;
}

Bird::Bounds Bird::getBounds() const {
    return { x_, y_, BIRD_WIDTH, BIRD_HEIGHT };
}

void Bird::draw(const goud::Context& ctx, goud_texture tex) const {
    float radians = rotation_ * (3.14159265358979323846f / 180.0f);
    goud_color white{ 1.0f, 1.0f, 1.0f, 1.0f };
    ctx.drawSprite(
        tex,
        x_ + BIRD_WIDTH / 2.0f,
        y_ + BIRD_HEIGHT / 2.0f,
        BIRD_WIDTH,
        BIRD_HEIGHT,
        radians,
        white
    );
}

void Bird::reset() {
    x_             = BIRD_START_X;
    y_             = BIRD_START_Y;
    velocity_      = 0.0f;
    rotation_      = 0.0f;
    jumpCooldown_  = 0.0f;
    animFrameIndex_ = 0;
    animTime_      = 0.0f;
}

}  // namespace flappy
