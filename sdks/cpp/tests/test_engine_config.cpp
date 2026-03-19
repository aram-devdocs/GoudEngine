#include <catch2/catch_test_macros.hpp>
#include <goud/goud.hpp>

TEST_CASE("EngineConfig::create succeeds", "[config]") {
    int status = -1;
    auto config = goud::EngineConfig::create(&status);
    REQUIRE(status == SUCCESS);
    REQUIRE(config.valid());
}

TEST_CASE("EngineConfig setters return SUCCESS", "[config]") {
    auto config = goud::EngineConfig::create();
    REQUIRE(config.setTitle("test") == SUCCESS);
    REQUIRE(config.setSize(640, 480) == SUCCESS);
    REQUIRE(config.setVsync(true) == SUCCESS);
    REQUIRE(config.setTargetFps(60) == SUCCESS);
    REQUIRE(config.setRenderBackend(0) == SUCCESS);
    REQUIRE(config.setWindowBackend(0) == SUCCESS);
}

TEST_CASE("EngineConfig move semantics", "[config]") {
    auto a = goud::EngineConfig::create();
    REQUIRE(a.valid());

    auto b = std::move(a);
    REQUIRE(b.valid());
    REQUIRE_FALSE(a.valid());
}

TEST_CASE("EngineConfig double-reset is safe", "[config]") {
    auto config = goud::EngineConfig::create();
    config.reset();
    REQUIRE_FALSE(config.valid());
    config.reset();  // Should not crash
    REQUIRE_FALSE(config.valid());
}

TEST_CASE("EngineConfig::createUnique", "[config]") {
    int status = -1;
    auto config = goud::EngineConfig::createUnique(&status);
    REQUIRE(status == SUCCESS);
    REQUIRE(config != nullptr);
    REQUIRE(config->valid());
}
