#include <catch2/catch_test_macros.hpp>
#include <goud/goud.hpp>

TEST_CASE("Error default construction", "[error]") {
    goud::Error err;
    REQUIRE(err.code() == SUCCESS);
    REQUIRE_FALSE(static_cast<bool>(err));
    REQUIRE(err.message().empty());
    REQUIRE(err.subsystem().empty());
    REQUIRE(err.operation().empty());
    REQUIRE(err.recoveryClass() == 0);
}

TEST_CASE("Error::last returns current error state", "[error]") {
    auto err = goud::Error::last();
    // After program init with no engine calls, last error should be SUCCESS
    REQUIRE(err.code() == SUCCESS);
}
