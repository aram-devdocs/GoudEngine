#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Single entry point for the game
 */
typedef struct GameSdk GameSdk;

struct GameSdk *game_new(uint32_t width, uint32_t height, const char *title);

void game_init(struct GameSdk *game);

void game_run(struct GameSdk *game);

void game_destroy(struct GameSdk *game);
