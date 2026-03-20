/* Test engine config lifecycle in the C SDK. */

#include <assert.h>
#include <stdio.h>
#include <goud/goud.h>

int main(void) {
    goud_engine_config config = NULL;

    /* goud_engine_config_init succeeds */
    assert(goud_engine_config_init(&config) == SUCCESS);

    /* Config is valid after init */
    assert(goud_engine_config_valid(config) == true);

    /* Setters return SUCCESS on valid config */
    assert(goud_engine_config_set_title_utf8(config, "Test Window") == SUCCESS);
    assert(goud_engine_config_set_window_size(config, 800, 600) == SUCCESS);
    assert(goud_engine_config_set_vsync_enabled(config, true) == SUCCESS);
    assert(goud_engine_config_set_target_fps_value(config, 60) == SUCCESS);

    /* goud_engine_config_dispose succeeds */
    assert(goud_engine_config_dispose(&config) == SUCCESS);

    /* Config is invalid (NULL) after dispose */
    assert(config == NULL);
    assert(goud_engine_config_valid(config) == false);

    /* Double dispose is safe (returns SUCCESS for NULL config) */
    assert(goud_engine_config_dispose(&config) == SUCCESS);

    /* Init with NULL out pointer returns error */
    assert(goud_engine_config_init(NULL) == ERR_INVALID_STATE);

    /* Dispose with NULL pointer returns error */
    assert(goud_engine_config_dispose(NULL) == ERR_INVALID_STATE);

    /* Setters with NULL config return error */
    assert(goud_engine_config_set_title_utf8(NULL, "test") == ERR_INVALID_STATE);
    assert(goud_engine_config_set_window_size(NULL, 800, 600) == ERR_INVALID_STATE);
    assert(goud_engine_config_set_vsync_enabled(NULL, true) == ERR_INVALID_STATE);
    assert(goud_engine_config_set_target_fps_value(NULL, 60) == ERR_INVALID_STATE);

    printf("test_engine_config: all assertions passed\n");
    return 0;
}
