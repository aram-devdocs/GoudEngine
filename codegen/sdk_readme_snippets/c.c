#include <goud/goud.h>

int main(void) {
    goud_engine_config* config = goud_engine_config_create();
    goud_engine_config_set_title(config, "My Game");
    goud_engine_config_set_size(config, 800, 600);

    goud_context ctx = goud_engine_config_build(config);
    // game loop here
    goud_context_destroy(ctx);
    return 0;
}
