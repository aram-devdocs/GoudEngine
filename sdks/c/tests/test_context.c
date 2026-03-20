/* Test context handle validation in the C SDK. */

#include <assert.h>
#include <stdio.h>
#include <goud/goud.h>

int main(void) {
    goud_context invalid;

    /* goud_context_invalid() returns the invalid sentinel */
    invalid = goud_context_invalid();

    /* goud_context_valid returns false for the invalid sentinel */
    assert(goud_context_valid(invalid) == false);

    /* goud_window_should_close_checked returns true for invalid context */
    assert(goud_window_should_close_checked(invalid) == true);

    /* goud_window_poll_events_checked returns 0 for invalid context */
    assert(goud_window_poll_events_checked(invalid) == 0.0f);

    /* goud_window_delta_time returns 0 for invalid context */
    assert(goud_window_delta_time(invalid) == 0.0f);

    /* goud_status_from_context returns error for invalid context */
    assert(goud_status_from_context(invalid) != SUCCESS);

    printf("test_context: all assertions passed\n");
    return 0;
}
