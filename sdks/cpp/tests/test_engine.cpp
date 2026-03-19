#include <catch2/catch_test_macros.hpp>
#include <goud/goud.hpp>

TEST_CASE("Engine default is invalid", "[engine]") {
    goud::Engine engine;
    REQUIRE_FALSE(engine.valid());
}

TEST_CASE("Engine::create with config", "[engine][gl_required]") {
    int status = -1;
    auto config = goud::EngineConfig::create(&status);
    REQUIRE(status == SUCCESS);
    config.setTitle("test_engine");
    config.setSize(64, 64);

    auto engine = goud::Engine::create(std::move(config), &status);
    REQUIRE(status == SUCCESS);
    REQUIRE(engine.valid());
    REQUIRE(engine.context().valid());
}

TEST_CASE("Engine::createShared", "[engine][gl_required]") {
    auto config = goud::EngineConfig::create();
    config.setTitle("test_shared_engine");
    config.setSize(64, 64);

    auto engine = goud::Engine::createShared(std::move(config));
    REQUIRE(engine != nullptr);
    REQUIRE(engine->valid());
    REQUIRE(engine->context().valid());
}
