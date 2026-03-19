#include "score.hpp"

#include <string>

namespace flappy {

void ScoreCounter::draw(const goud::Context& ctx, goud_texture digitTextures[10]) const {
    std::string scoreStr = std::to_string(score_);
    float xOffset = SCREEN_WIDTH / 2.0f - 30.0f;
    float yOffset = 50.0f;
    goud_color white{ 1.0f, 1.0f, 1.0f, 1.0f };

    for (std::size_t i = 0; i < scoreStr.size(); ++i) {
        int digit = scoreStr[i] - '0';
        float x = xOffset + static_cast<float>(i) * SCORE_DIGIT_SPACING + SCORE_DIGIT_WIDTH / 2.0f;
        float y = yOffset + SCORE_DIGIT_HEIGHT / 2.0f;
        ctx.drawSprite(
            digitTextures[digit],
            x, y,
            SCORE_DIGIT_WIDTH, SCORE_DIGIT_HEIGHT,
            0.0f,
            white
        );
    }
}

}  // namespace flappy
