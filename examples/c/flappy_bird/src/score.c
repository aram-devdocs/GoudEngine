#include "score.h"
#include <stdio.h>
#include <string.h>

void score_draw(const Score *s, goud_context ctx, goud_texture digit_textures[10]) {
    char score_str[16];
    snprintf(score_str, sizeof(score_str), "%d", s->value);
    size_t len = strlen(score_str);

    float x_offset = SCREEN_WIDTH / 2.0f - 30.0f;
    float y_offset = 50.0f;
    goud_color white = { 1.0f, 1.0f, 1.0f, 1.0f };

    size_t i;
    for (i = 0; i < len; ++i) {
        int digit = score_str[i] - '0';
        float x = x_offset + (float)i * SCORE_DIGIT_SPACING + SCORE_DIGIT_WIDTH / 2.0f;
        float y = y_offset + SCORE_DIGIT_HEIGHT / 2.0f;
        goud_renderer_draw_sprite_color(
            ctx,
            digit_textures[digit],
            x, y,
            SCORE_DIGIT_WIDTH, SCORE_DIGIT_HEIGHT,
            0.0f,
            white
        );
    }
}
