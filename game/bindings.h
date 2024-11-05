#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Option_Renderer2D Option_Renderer2D;

/**
 * # Window
 *
 * An abstraction layer for creating a GLFW window.
 *
 * ## Example
 * ```
 * let mut window = Window::new(1280, 720, "Window Title");
 * window.init_gl();
 *
 * while !window.should_close() {
 *     window.update();
 * }
 * ```
 */
typedef struct Window Window;

typedef struct WindowBuilder WindowBuilder;

/**
 * Single entry point for the game
 */
typedef struct GameSdk {
  struct Window window;
  struct Option_Renderer2D renderer_2d;
  float elapsed_time;
} GameSdk;

struct GameSdk *game_new(uint32_t width, uint32_t height, const char *title);

void game_init(struct GameSdk *game);

void game_run(struct GameSdk *game);

void game_destroy(struct GameSdk *game);

/**
 * Creates a new game instance.
 */
struct GameSdk new(struct WindowBuilder data);
