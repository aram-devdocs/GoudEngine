#include <goud/goud.h>

int main(void) {
    goud_context context = goud_context_invalid();
    goud_entity entity = GOUD_INVALID_ENTITY_ID;
    int status = goud_context_init(&context);

    if (status != SUCCESS) {
        return status;
    }

    status = goud_entity_spawn(context, &entity);
    if (status != SUCCESS) {
        (void)goud_context_dispose(&context);
        return status;
    }

    return goud_context_dispose(&context);
}
