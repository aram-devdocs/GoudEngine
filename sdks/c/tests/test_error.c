/* Test error handling functions in the C SDK. */

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <goud/goud.h>

int main(void) {
    goud_error_info err;

    /* Default error info is zeroed after clear */
    goud_error_info_clear(&err);
    assert(err.code == SUCCESS);
    assert(err.recovery_class == 0);
    assert(err.message[0] == '\0');
    assert(err.subsystem[0] == '\0');
    assert(err.operation[0] == '\0');

    /* goud_last_error_code() returns SUCCESS initially (no error set) */
    assert(goud_last_error_code() == SUCCESS);

    /* goud_get_last_error() with NULL returns ERR_INVALID_STATE */
    assert(goud_get_last_error(NULL) == ERR_INVALID_STATE);

    /* goud_status_from_bool(true) returns SUCCESS */
    assert(goud_status_from_bool(true) == SUCCESS);

    /* goud_status_from_bool(false) returns non-zero */
    assert(goud_status_from_bool(false) != SUCCESS);

    /* goud_error_info_clear with NULL is safe (no crash) */
    goud_error_info_clear(NULL);

    printf("test_error: all assertions passed\n");
    return 0;
}
